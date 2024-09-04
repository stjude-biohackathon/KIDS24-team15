//! An example for runner a task using the Docker backend service.

use crankshaft::engine::service::runner::backend::Docker;
use crankshaft::engine::task::Execution;
use crankshaft::engine::Task;

#[tokio::main]
async fn main() {
    let mut docker = Docker::try_new().unwrap();

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

    let result = docker.submit(task).await.unwrap();
    println!("Exit code: {:?}", &result)
}
