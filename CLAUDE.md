# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a biohackathon project (KIDS24-team15) that implements a task execution system for running WDL (Workflow Description Language) workflows. The system consists of three main Rust workspace members and a Vue-based dashboard.

## Workspace Structure

This is a Cargo workspace with three main crates:

- **crankshaft**: Main CLI application with two binaries (`crankshaft` and `sprocket`)
- **tes**: Task Execution Service client library for communicating with TES servers
- **wdl-runtime**: WDL evaluation runtime for parsing and executing WDL workflows

Additionally:
- **tes-dashboard**: Vue 3 + Vite + Vuetify dashboard for monitoring TES job status

## Build and Run Commands

### Rust Components

```bash
# Build all workspace members
cargo build

# Build release version
cargo build --release

# Run tests for all workspace members
cargo test

# Run tests for a specific crate
cargo test -p crankshaft
cargo test -p tes
cargo test -p wdl-runtime

# Run a specific test by name
cargo test test_name

# Run the crankshaft binary (simple task runner)
cargo run --bin crankshaft -- <path_to_task_file.json>

# Run the sprocket binary (WDL task runner)
cargo run --bin sprocket -- run --task <task_name> --inputs <inputs.json> <path_to_wdl_file>

# Example sprocket command
cargo run --bin sprocket -- run --task MyTask --inputs demo.json demo.wdl
```

### TypeScript Dashboard

```bash
# Navigate to dashboard directory
cd tes-dashboard

# Install dependencies
npm install

# Run development server
npm run dev
# or
vite

# Build for production
npm run build

# Run tests
npm test
```

## Key Architecture Concepts

### Execution Engine (Crankshaft)

The execution engine follows a backend-based architecture where tasks are submitted to configurable execution backends:

- **Task Model**: Tasks are built using a builder pattern and contain inputs, outputs, resources, and executions
- **Backend Abstraction**: The `Runner` service accepts any backend implementing the `Backend` trait
- **Three Backend Types**:
  - `Docker`: Executes tasks in Docker containers locally
  - `TES`: Submits tasks to remote Task Execution Service endpoints
  - `Generic`: Executes arbitrary shell commands (e.g., LSF/bsub submission)

### Configuration

Backends are configured via TOML files (typically `~/.crankshaft/config.toml`):

```toml
[[backends]]
name = "my-backend"
kind = "Generic"  # or "Docker" or "TES"
submit = "bsub ~{script}"
default-cpu = 1
default-ram = 1
```

The config module at `crankshaft/src/engine/config.rs` handles loading these configurations.

### WDL Runtime

The `wdl-runtime` crate provides:
- **Runtime**: Main evaluation context for WDL documents
- **TaskEvaluator**: Evaluates individual WDL tasks
- **Value**: Type system for WDL values
- Uses upstream `wdl-analysis`, `wdl-ast`, and `wdl-grammar` crates from a forked branch

### TES Client

The `tes` crate provides a simple HTTP client for TES servers with:
- Automatic retry logic with exponential backoff
- Task creation and retrieval
- Health check capabilities

## Development Patterns

### Linting Standards

Crankshaft uses strict linting enabled in `crankshaft/Cargo.toml`:
- `missing_docs = "warn"` - All public items must have documentation
- `missing_docs_in_private_items = "warn"` - Private items should be documented
- Code should follow rust-2018-idioms, rust-2021-compatibility, and rust-2024-compatibility

### Builder Pattern

Tasks and their components (Input, Output, Execution, Resources) use the builder pattern extensively. See `crankshaft/src/engine/task/builder.rs` and related builder modules.

### Async Runtime

The project uses Tokio for async execution. Most engine operations are async and should be run within a Tokio runtime context.

## Dashboard Integration

The TES dashboard requires a `.env.development.local` file in `tes-dashboard/` with:
```
VITE_APP_API='<TES_API_URL>'
VITE_APP_AUTH_HEADER='<AUTH_HEADER>'
```

Uses:
- Vue 3 with Composition API
- Vuetify for UI components
- PrimeVue for charts
- Pinia for state management (boilerplate included)
- Vue Router (boilerplate included)

## Dependencies

Key external dependencies:
- **WDL parsing**: Custom fork at `https://github.com/peterhuene/wdl` (branch: `hackathon`)
- **Docker**: `bollard` for Docker API interactions
- **HTTP**: `reqwest` with middleware for retries
- **CLI**: `clap` with derive macros
- **Serialization**: `serde` with JSON/YAML support
