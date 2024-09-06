//! An example for runner a task using the TES backend service.

use crankshaft::engine::service::runner::backend::tes::TesBackend;
use crankshaft::engine::task::Execution;
use crankshaft::engine::Engine;
use crankshaft::engine::Task;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::EnvFilter;

/// The environment variable name for the token.
const TOKEN_ENV_NAME: &str = "TOKEN";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let token = std::env::var(TOKEN_ENV_NAME).unwrap();

    let url = std::env::args().nth(1).expect("no url provided");
    let mut engine = Engine::new_with_backend("tes", TesBackend::new(url, Some(token)));

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
        .map(|_| engine.submit("tes", task.clone()).callback)
        .collect::<Vec<_>>();

    engine.run().await;

    for rx in receivers {
        info!(runner = "TES", reply = ?rx.await.unwrap());
    }
}
