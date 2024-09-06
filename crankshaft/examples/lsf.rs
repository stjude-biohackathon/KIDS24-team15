//! An example for runner a task using the generic LSF backend service.

use crankshaft::engine::config::Config;
use crankshaft::engine::service::runner::backend::config::BackendType;
use crankshaft::engine::service::runner::backend::generic::GenericBackend;
use crankshaft::engine::task::Execution;
use crankshaft::engine::Engine;
use crankshaft::engine::Task;
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

    let config = Config::new(std::env::args().nth(1).expect("no config provided"))
        .expect("could not load from config file")
        .backends
        .into_iter()
        .find(|backend| matches!(backend.kind, BackendType::Generic(_)))
        .expect("at least one generic backend config to be present in the config");

    let backend = GenericBackend::try_from(config).expect("parsing the backend configuration");
    let mut engine = Engine::empty().with_backend("generic", backend.to_runner());

    let task = Task::builder()
        .name("my-example-task")
        .description("a longer description")
        .extend_executions(vec![Execution::builder()
            .working_directory(".")
            .image("ubuntu")
            .args(&[String::from("echo"), String::from("'hello world from LSF'")])
            .stdout("stdout.txt")
            .stderr("stderr.txt")
            .try_build()
            .unwrap()])
        .try_build()
        .unwrap();

    let receivers = (0..10000)
        .map(|_| engine.submit("generic", task.clone()).callback)
        .collect::<Vec<_>>();

    engine.run().await;

    for rx in receivers {
        info!(runner = "LSF", reply = ?rx.await.unwrap());
    }
}
