use std::net::TcpListener;

use transactions_service::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let server = run(listener).await?;
    server.await
}
