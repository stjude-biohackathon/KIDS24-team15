// A simple task runner CLI for executing tasks defined in JSON or YAML files.

use clap::{Arg, Command};  // Import clap for command-line interface
use std::fs;               // Import fs for file operations
use serde::{Deserialize, Serialize};  // Import serde for JSON parsing
/// Represents a task to be executed.
#[derive(Debug, Deserialize, Serialize)]
struct Task {
    /// The name of the task.	
    name: String,
   /// The name of the task.
    command: String,
}

fn main() {
    let matches = Command::new("crankshaft")
        .version("1.0")
        .author("Your Name")
        .about("A simple task runner CLI")
        .subcommand(
            Command::new("run")
                .about("Runs a task")
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .value_name("FILE")
                        .help("Task definition file")
                        .required(true),
                )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let task_file = matches.get_one::<String>("file").unwrap();  // Correctly fetching the argument value
        println!("Running task defined in file: {}", task_file);

        // Reading file content
        match fs::read_to_string(task_file) {
            Ok(content) => println!("File content:\n{}", content),  // Print file content if successful
            Err(e) => println!("Error reading file: {}", e),       // Error handling for file read errors
        }
    }
}

