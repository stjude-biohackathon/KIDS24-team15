//! An example for runner that uses multiple backends at the same time.

use crankshaft::engine::config::Config;
use crankshaft::engine::service::runner::backend::config::BackendType;
use crankshaft::engine::service::runner::backend::generic::GenericBackend;
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

    let token = std::env::var(TOKEN_ENV_NAME).ok();

    let url = std::env::args().nth(1).expect("no url provided");
    let config = Config::new(std::env::args().nth(2).expect("no config provided"))
        .expect("could not load from config file")
        .backends
        .into_iter()
        .find(|backend| matches!(backend.kind, BackendType::Generic(_)))
        .expect("at least one generic backend config to be present in the config");

    let mut engine = Engine::empty()
        .with_docker(false)
        .expect("docker daemon to be alive and reachable")
        .with_backend("tes", TesBackend::new(url, token))
        .with_backend(
            "lsf",
            GenericBackend::try_from(config)
                .expect("parsing the backend configuration")
                .to_runner(),
        );

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

    let mut receivers = Vec::new();
    let runners = engine.runners().map(|s| s.to_owned()).collect::<Vec<_>>();

    for runner in &runners {
        if runner == "lsf" {
            continue;
        }

        info!("creating jobs within {runner}");

        for _ in 0..10 {
            receivers.push(engine.submit(runner, task.clone()).callback);
        }
    }

    engine.run().await;

    for rx in receivers {
        info!(reply = ?rx.await.unwrap());
    }
}
