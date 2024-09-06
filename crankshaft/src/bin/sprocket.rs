//! A testing implementation for a `sprocket run` command.

use anyhow::{anyhow, bail, Context, Result};
use clap::{Arg, Command};
use codespan_reporting::{
    files::SimpleFile,
    term::{
        emit,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use colored::Colorize;
use crankshaft::engine::{
    task::{
        input::{self, Contents},
        Execution, Input,
    },
    Engine, Task,
};
use std::{borrow::Cow, collections::HashMap, fs, io::IsTerminal, path::PathBuf};
use tempfile::tempdir;
use wdl_analysis::{AnalysisResult, Analyzer};
use wdl_ast::{AstToken, Diagnostic, Severity, SyntaxNode};
use wdl_runtime::{Runtime, TaskEvaluator, Value};

/// Emits the given diagnostics to the output stream.
///
/// The use of color is determined by the presence of a terminal.
///
/// In the future, we might want the color choice to be a CLI argument.
fn emit_diagnostics(path: &str, source: &str, diagnostics: &[Diagnostic]) -> Result<()> {
    let file = SimpleFile::new(path, source);
    let mut stream = StandardStream::stdout(if std::io::stdout().is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });
    for diagnostic in diagnostics.iter() {
        emit(
            &mut stream,
            &Config::default(),
            &file,
            &diagnostic.to_codespan(),
        )
        .context("failed to emit diagnostic")?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = inner_main().await {
        eprintln!(
            "{error}: {e:?}",
            error = if std::io::stderr().is_terminal() {
                "error".red().bold()
            } else {
                "error".normal()
            }
        );
        std::process::exit(1);
    }
}

/// An inner main that returns result.
///
/// This exists so we can do custom error handling instead of returning `Result` from `main`.
async fn inner_main() -> Result<()> {
    let matches = Command::new("sprocket")
        .version("1.0")
        .about("Runs a WDL task")
        .subcommand(
            Command::new("run")
                .about("Runs a WDL task")
                .arg(
                    Arg::new("PATH")
                        .help("The path to the WDL file defining the task to run")
                        .required(true),
                )
                .arg(
                    Arg::new("TASK")
                        .long("task")
                        .help("The name of the task to run")
                        .required(true),
                )
                .arg(
                    Arg::new("INPUTS")
                        .long("inputs")
                        .help("The inputs JSON file"),
                ),
        )
        .arg_required_else_help(true)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let task_file = matches.get_one::<String>("PATH").unwrap();
        let task_name = matches.get_one::<String>("TASK").unwrap();
        let inputs_file = matches.get_one::<String>("INPUTS");
        let result = analyze_wdl(PathBuf::from(task_file)).await?;

        let document = result
            .parse_result()
            .document()
            .expect("should have a parsed document");

        match document.ast() {
            wdl_ast::Ast::Unsupported => {
                panic!("should not have parsed an unsupported document without error")
            }
            wdl_ast::Ast::V1(ast) => {
                let task = ast
                    .tasks()
                    .find(|t| t.name().as_str() == task_name)
                    .ok_or_else(|| {
                        anyhow!("document does not contain a task named `{task_name}`")
                    })?;
                let mut runtime = Runtime::new(result.scope());
                let evaluator = TaskEvaluator::new(task);

                let inputs = if let Some(inputs_file) = inputs_file {
                    read_inputs(&mut runtime, inputs_file)?
                } else {
                    Default::default()
                };

                match evaluator.evaluate(&mut runtime, &inputs, "/tmp") {
                    Ok(evaluated) => {
                        let container = match evaluated
                            .requirements()
                            .get("container")
                            .or_else(|| evaluated.requirements().get("docker"))
                        {
                            Some(container) => container.unwrap_string(&runtime),
                            None => {
                                bail!("task `{task_name}` is missing a `container` requirement");
                            }
                        };

                        let input = Input::builder()
                            .contents(Contents::Literal(evaluated.command().to_string()))
                            .path("/exec/command")
                            .r#type(input::Type::File)
                            .try_build()
                            .unwrap();

                        let mut engine = Engine::default();
                        let task = Task::builder()
                            .name(task_name)
                            .extend_inputs([input])
                            .extend_executions([Execution::builder()
                                .image(container)
                                .args(["bash", "-C", "/exec/command"])
                                .stdout("stdout.txt")
                                .stderr("stderr.txt")
                                .try_build()
                                .context("failed to build execution definition")?])
                            .try_build()
                            .context("failed to build task definition")?;

                        let receivers = (0..1)
                            .map(|_| engine.submit("docker", task.clone()).callback)
                            .collect::<Vec<_>>();

                        engine.run().await;

                        for rx in receivers {
                            let reply = rx.await.expect("failed to receive reply");
                            let exec_result =
                                &reply.executions.expect("should have execution result")[0];
                            if exec_result.status != 0 {
                                bail!(
                                    "task failed with exit code {status}:\n{stderr}",
                                    status = exec_result.status,
                                    stderr = exec_result.stderr
                                );
                            }

                            let dir = tempdir().context("failed to create temp directory")?;
                            let stdout = dir.path().join("stdout");
                            fs::write(&stdout, &exec_result.stdout).with_context(|| {
                                format!(
                                    "failed to write stdout to `{stdout}`",
                                    stdout = stdout.display()
                                )
                            })?;

                            let stderr = dir.path().join("stderr");
                            fs::write(&stderr, &exec_result.stderr).with_context(|| {
                                format!(
                                    "failed to write stderr to `{stderr}`",
                                    stderr = stderr.display()
                                )
                            })?;

                            match evaluated.outputs(&mut runtime, stdout, stderr) {
                                Ok(outputs) => {
                                    for (name, value) in outputs {
                                        println!(
                                            "Output `{name}`:\n{value}",
                                            name = name.as_ref().as_str(),
                                            value = value.display(&runtime)
                                        );
                                    }
                                }
                                Err(diagnostic) => {
                                    emit_diagnostics(
                                        task_file,
                                        &result
                                            .parse_result()
                                            .root()
                                            .map(|n| {
                                                SyntaxNode::new_root(n.clone()).text().to_string()
                                            })
                                            .unwrap_or(String::new()),
                                        &[diagnostic],
                                    )?;

                                    bail!("aborting due to evaluation error");
                                }
                            }
                        }
                    }
                    Err(diagnostic) => {
                        emit_diagnostics(
                            task_file,
                            &result
                                .parse_result()
                                .root()
                                .map(|n| SyntaxNode::new_root(n.clone()).text().to_string())
                                .unwrap_or(String::new()),
                            &[diagnostic],
                        )?;

                        bail!("aborting due to evaluation error");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Reads task inputs from a given JSON file.
fn read_inputs(runtime: &mut Runtime<'_>, inputs_file: &str) -> Result<HashMap<String, Value>> {
    let contents = &fs::read_to_string(inputs_file)
        .with_context(|| format!("failed to read inputs file `{inputs_file}`"))?;
    let inputs: serde_json::Value = serde_json::from_str(contents)
        .with_context(|| format!("failed to deserialize JSON inputs file `{inputs_file}`"))?;
    let object = inputs
        .as_object()
        .with_context(|| format!("inputs file `{inputs_file}` is not a JSON object"))?;

    let mut inputs = HashMap::new();
    for (name, value) in object.iter() {
        let value = match value {
            serde_json::Value::Bool(v) => (*v).into(),
            serde_json::Value::Number(v) => v
                .as_i64()
                .with_context(|| {
                    format!("input value `{name}` cannot be represented as a 64-bit signed integer")
                })?
                .into(),
            serde_json::Value::String(s) => runtime.new_string(s),
            _ => bail!("input value `{name}` has an unsupported type"),
        };

        inputs.insert(name.clone(), value);
    }

    Ok(inputs)
}

/// Analyzes the given WDL document.
async fn analyze_wdl(wdl_path: PathBuf) -> Result<AnalysisResult> {
    let analyzer = Analyzer::new(|_: (), _, _, _| async {});
    analyzer.add_documents(vec![wdl_path.clone()]).await?;
    let mut results = analyzer.analyze(()).await?;

    let mut result_index = None;
    let mut error_count = 0;
    let cwd = std::env::current_dir().ok();
    for (index, result) in results.iter().enumerate() {
        let path = result.uri().to_file_path().ok();

        // Attempt to strip the CWD from the result path
        let path = match (&cwd, &path) {
            // Use the id itself if there is no path
            (_, None) => result.uri().as_str().into(),
            // Use just the path if there's no CWD
            (None, Some(path)) => path.to_string_lossy(),
            // Strip the CWD from the path
            (Some(cwd), Some(path)) => path.strip_prefix(cwd).unwrap_or(path).to_string_lossy(),
        };

        if path == wdl_path.to_string_lossy() {
            result_index = Some(index);
        }

        let diagnostics: Cow<'_, [Diagnostic]> = match result.parse_result().error() {
            Some(e) => vec![Diagnostic::error(format!("failed to read `{path}`: {e:#}"))].into(),
            None => result.diagnostics().into(),
        };

        if !diagnostics.is_empty() {
            emit_diagnostics(
                &path,
                &result
                    .parse_result()
                    .root()
                    .map(|n| SyntaxNode::new_root(n.clone()).text().to_string())
                    .unwrap_or(String::new()),
                &diagnostics,
            )?;

            error_count += diagnostics
                .iter()
                .filter(|d| d.severity() == Severity::Error)
                .count();
        }
    }

    if error_count > 0 {
        bail!(
            "aborting due to previous {error_count} error{s}",
            s = if error_count == 1 { "" } else { "s" }
        );
    }

    Ok(results.swap_remove(result_index.expect("should have seen result for requested file")))
}
