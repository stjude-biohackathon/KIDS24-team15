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

    let config = crankshaft::engine::config::Config::new("configs/lsf.toml")
        .expect("Load from example config")
        .backends[0]
        .clone();

    let backend =
        crankshaft::engine::service::runner::backend::generic::GenericBackend::from_config(config)
            .expect("Get backend from config");

    let generic_runner =
        crankshaft::engine::service::runner::backend::generic::GenericRunner::new(backend);
    let runner = crankshaft::engine::service::runner::Runner::new(generic_runner);

    let mut engine = Engine::with_runner(runner);

    let task = Task::builder()
        .name("my-example-task")
        .description("a longer description")
        .extend_executions(vec![Execution::builder()
            .working_directory(".")
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
