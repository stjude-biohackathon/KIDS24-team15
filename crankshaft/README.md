# Crankshaft - A Simple Task Runner CLI

Crankshaft is a command-line interface (CLI) tool designed to execute tasks defined in JSON or YAML files. It allows users to automate commands via the system shell, making it easy to run predefined scripts or repetitive tasks.

## Features

- Supports task definitions in both JSON and YAML formats.
- Executes commands directly through the system's shell.
- Compatible with Windows and Unix-like systems( Linux, macOS).

## Requirements

- Rust installed on your system. Install Rust from [rust-lang.org](https://www.rust-lang.org/).
- A compatible shell available.
  
## Setup

### 1. Clone the Repository

```
git clone <https://github.com/stjude-biohackathon/KIDS24-team15.git>
cd KIDS24-team15
```
### 2. Build the Project

Compile the project using Cargo, Rust's package manager:

```
cargo build

```
### How to Use

### 1. Create a Task File

- Define your task in a JSON or YAML file. Each task file should include a name and a command to execute.

- Example JSON file (example_task.json)

{
    "name": "Sample Task",
    "command": "echo Hello, json file input!"
}

- Example YAML file (example_task.yaml)

name: Sample Task
command: echo Hello, yaml file!

### 2 Run a Task

- Use the following command to run a task defined in your JSON or YAML file:

```
cargo run --bin crankshaft -- <path_to_task_file>
```

- Example Command

```
cargo run --bin crankshaft -- ./example_task.json

```
### 3 Expected Output

The command specified in the task file will be executed via the system shell.

For the examples provided, the expected output should be "Hello, Json file input!". when using json file.

### Troubleshooting
- No Output: Verify that the command in your task file works independently in your terminal.
- JSON/YAML Errors: Ensure that your task files are correctly formatted with valid JSON or YAML syntax.
- Command Execution Errors: Check that the shell command is valid and your system environment is set up correctly.

### References

- 1. Rust programming language: <https://www.rust-lang.org/>
- 2. Rust Libraries Used:
	- Clap (Command-line Argument Parser) <https://github.com/clap-rs/clap> 
	- Serde (Serialization and Deserialization) <https://github.com/serde-rs/serde>
- 3. Workflow Description Language (WDL): <https://github.com/openwdl/wdl>

### Contributors

- Peter Huene <https://github.com/peterhuene> 
- Clay McLeod <dhttps://github.com/claymcleod>
- John McGuigan <https://github.com/jrm5100 >
- Andrew Frantz <https://github.com/a-frantz>
- Braden Everson <https://github.com/BradenEverson>
- Jared Andrews <https://github.com/j-andrews7>
- Michael Gattas <https://github.com/michaelgattas>
- Suchitra Chavan <https://github.com/schavan023>
