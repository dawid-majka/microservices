use std::net::TcpListener;

use transactions_service::{configuration::get_configuration, run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    let server = run(listener).await?;
    server.await
}
