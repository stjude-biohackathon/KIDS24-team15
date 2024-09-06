use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command as ProcessCommand;
use std::path::PathBuf;
use futures_util::FutureExt; 
use wdl_analysis::{Analyzer}; 

#[derive(Debug, Deserialize, Serialize)]
struct Task {
    name: String,
    command: String,
}

#[tokio::main]
async fn main() {
    let matches = Command::new("crankshaft")
        .version("1.0")
        .about("CLI for JSON, YAML, and WDL files")
        .subcommand(
            Command::new("run")
                .about("Runs a task")
                .arg(
                    Arg::new("file")
                        .help("Task file (JSON, YAML, or WDL)")
                        .required(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let task_file = matches.get_one::<String>("file").unwrap();

        if task_file.ends_with(".wdl") {
            if let Err(e) = analyze_wdl(PathBuf::from(task_file)).await {
                eprintln!("Error analyzing WDL file: {}", e);
            }
        } else {
            run_simple_task(task_file);
        }
    }
}

fn run_simple_task(task_file: &str) {
    if let Ok(content) = fs::read_to_string(task_file) {
        let task: Result<Task, Box<dyn std::error::Error>> = if task_file.ends_with(".yaml") || task_file.ends_with(".yml") {
            serde_yaml::from_str(&content).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        } else {
            serde_json::from_str(&content).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        };

        if let Ok(task) = task {
            let _ = ProcessCommand::new("sh").arg("-c").arg(&task.command).status();
        }
    }
}

async fn analyze_wdl(wdl_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let progress_handler = |_, _, _, _| async {}.boxed_local(); 

    let mut analyzer = Analyzer::new(progress_handler);

    analyzer.add_documents(vec![wdl_path]).await?;

    let results = analyzer.analyze(()).await?;

    for result in results {
        for diagnostic in result.diagnostics() {
            println!("{}: {}", diagnostic.severity(), diagnostic.message()); 
        }
    }

    Ok(())
}

