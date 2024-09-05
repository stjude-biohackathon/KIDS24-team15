#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "X-Pinggy-No-Screen",
        header::HeaderValue::from_static("value"),
    );
    let client = TesHttpClient::new(
        "https://rnhuq-40-124-106-161.a.free.pinggy.link/ga4gh/tes/v1/",
        headers,
    )
    .unwrap();

    let foo = client.service_info().await.unwrap();

    let task = TesTask {
        name: Some("Hello World".to_string()),
        description: Some("Hello World, inspired by Funnel's most basic example".to_string()),
        executors: vec![TesExecutor {
            image: "alpine".to_string(),
            command: vec!["echo".to_string(), "TESK says: Hello World".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let id = client.create_task(task).await.unwrap();
    let res_task = client.get_task(id).await.unwrap();

    dbg!(res_task);
}
