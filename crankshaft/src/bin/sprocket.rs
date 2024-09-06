//! A testing implementation for a `sprocket run` command.

use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;
use wdl_analysis::Analyzer;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("sprocket")
        .version("1.0")
        .about("Runs a WDL task")
        .subcommand(
            Command::new("run").about("Runs a WDL task").arg(
                Arg::new("PATH")
                    .help("The path to the WDL file defining the task to run")
                    .required(true),
            ),
        )
        .arg_required_else_help(true)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let task_file = matches.get_one::<String>("PATH").unwrap();
        analyze_wdl(PathBuf::from(task_file)).await?;
    }

    Ok(())
}

/// Analyzes the given WDL document.
async fn analyze_wdl(wdl_path: PathBuf) -> Result<()> {
    let analyzer = Analyzer::new(|_: (), _, _, _| async {});
    analyzer.add_documents(vec![wdl_path]).await?;
    let results = analyzer.analyze(()).await?;

    for result in results {
        for diagnostic in result.diagnostics() {
            println!("{:?}: {}", diagnostic.severity(), diagnostic.message());
        }
    }

    Ok(())
}
