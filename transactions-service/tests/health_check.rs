use std::net::TcpListener;

#[actix_web::test]
async fn servers_is_working() {
    // Given
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // When
    let response = client
        .get(&format!("{}/", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Then
    assert!(response.status().is_success());
}

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();

    let server = transactions_service::run(listener)
        .await
        .expect("Server initialization failed.");

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
