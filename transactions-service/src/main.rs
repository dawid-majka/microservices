use std::net::TcpListener;

use transactions_service::{
    configuration::get_configuration,
    run,
    telemetry::{get_subscriber, init_subscriber},
    CouchbaseConnection,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber(
        "transactions-service".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_data = CouchbaseConnection::new(&configuration);

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    let server = run(listener, connection_data).await?;
    server.await
}
