//! An example for runner a task using the Docker backend service.
use std::io::Write;

use crankshaft::engine::task::input;
use crankshaft::engine::task::Execution;
use crankshaft::engine::task::Input;
use crankshaft::engine::Engine;
use crankshaft::engine::Task;
use tempfile::NamedTempFile;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let mut engine = Engine::default();

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Hello, world from an input").unwrap();

    // Get the path to the temp file
    let temp_path = temp_file.path().to_path_buf();
    let input = Input::builder()
        .contents(temp_path)
        .path("/volA/test_input.txt")
        .r#type(input::Type::File)
        .try_build()
        .unwrap();

    let task = Task::builder()
        .name("my-example-task")
        .description("a longer description")
        .extend_inputs(vec![input])
        .extend_executions(vec![
            Execution::builder()
                .image("ubuntu")
                .args(&[
                    String::from("bash"),
                    String::from("-c"),
                    String::from("ls /volA"),
                ])
                .try_build()
                .unwrap(),
            Execution::builder()
                .image("ubuntu")
                .args(&[String::from("cat"), String::from("/volA/test_input.txt")])
                .try_build()
                .unwrap(),
        ])
        .extend_volumes(vec!["/volA".to_string(), "/volB".to_string()])
        .try_build()
        .unwrap();

    let receivers = (0..10)
        .map(|_| engine.submit("docker", task.clone()).callback)
        .collect::<Vec<_>>();

    engine.run().await;

    for rx in receivers {
        info!(runner = "Docker", reply = ?rx.await.unwrap());
    }
}
