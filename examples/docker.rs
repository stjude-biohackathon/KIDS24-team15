//! An example for runner a task using the Docker backend service.

use crankshaft::engine::service::runner::backend::Docker;
use crankshaft::engine::task::Execution;
use crankshaft::engine::Task;
use crankshaft::engine::task::resources::Resources;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let mut docker = Docker::try_new().unwrap();

    let task = Task::builder()
        .name("my-example-task")
        .unwrap()
        .description("a longer description")
        .unwrap()
        .resources(Resources::builder()
            .disk_gb(1.0)
            .ram_gb(0.1)
            .cpu_cores(4)
            .build())
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

    docker.submit(task).await;
}
