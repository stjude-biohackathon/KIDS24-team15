//! A command to evaluate WDL.

use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command as ProcessCommand;

#[derive(Debug, Deserialize, Serialize)]
struct Task {
    name: String,
    command: String,
}

fn main() {
    let matches = Command::new("crankshaft")
        .version("1.0")
        .about("A simple task runner CLI using JSON and YAML")
        .arg(
            Arg::new("file")
                .help("Task definition file (JSON or YAML)")
                .required(true),
        )
        .get_matches();

    if let Some(task_file) = matches.get_one::<String>("file") {
        if let Ok(content) = fs::read_to_string(task_file) {
            let task: Result<Task, Box<dyn std::error::Error>> = if task_file.ends_with(".yaml") || task_file.ends_with(".yml") {
                serde_yaml::from_str(&content).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            } else if task_file.ends_with(".json") {
                serde_json::from_str(&content).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            } else {
                Err("Unsupported file format".into())
            };

            if let Ok(task) = task {
                let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
                let arg = if cfg!(target_os = "windows") { "/C" } else { "-c" };

		let _ = ProcessCommand::new(shell).arg(arg).arg(&task.command).status();
            }
        }
    }
}
