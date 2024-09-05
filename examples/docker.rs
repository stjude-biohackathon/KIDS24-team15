//! An example for runner a task using the Docker backend service.

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

    let mut engine = Engine::default();

    let task = Task::builder()
        .name("my-example-task")
        .unwrap()
        .description("a longer description")
        .unwrap()
        .extend_executors(vec![Execution::builder()
            .image("ubuntu")
            .args(&[String::from("echo"), String::from("'hello, world!'")])
            .stdout("stdout.txt")
            .stderr("stderr.txt")
            .try_build()
            .unwrap()])
        .unwrap()
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
