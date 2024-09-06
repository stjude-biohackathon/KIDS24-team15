//! An example for runner a task using multiple backend services.

use crankshaft::engine::task::Execution;
use crankshaft::engine::Engine;
use crankshaft::engine::Task;
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

    let mut engine = Engine::with_default_tes();

    let task = Task::builder()
        .name("my-example-task")
        .description("a longer description")
        .extend_executions(vec![Execution::builder()
            .image("ubuntu")
            .args(&[String::from("echo"), String::from("'hello, world!'")])
            .stdout("stdout.txt")
            .stderr("stderr.txt")
            .try_build()
            .unwrap()])
        .try_build()
        .unwrap();

    let receivers = (0..10)
        .map(|_| engine.submit(task.clone()).callback)
        .collect::<Vec<_>>();

    engine.run().await;

    for rx in receivers {
        println!("Reply: {:?}", rx.await.unwrap());
    }
}