use std::{net::TcpListener, sync::Arc};

use actix_web::{dev::Server, web, App, HttpServer};
use configuration::Settings;
use couchbase::Cluster;
use routes::{
    health_check::hello,
    transactions::{transactions, transactions_by_type},
};
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod model;
pub mod routes;
pub mod telemetry;

#[derive(Debug, Clone)]
pub struct CouchbaseConnection {
    pub cluster: Arc<Cluster>,
    pub bucket_name: String,
    pub scope_name: String,
    pub collection_name: String,
}

impl CouchbaseConnection {
    pub fn new(configuration: &Settings) -> Self {
        let cluster = Cluster::connect(
            &configuration.database.connection_string(),
            &configuration.database.username,
            &configuration.database.password,
        );

        CouchbaseConnection {
            cluster: Arc::new(cluster),
            bucket_name: configuration.database.bucket_name.clone(),
            scope_name: configuration.database.scope_name.clone(),
            collection_name: configuration.database.collection_name.clone(),
        }
    }

    pub fn test_connection(configuration: &Settings) -> Self {
        let cluster = Cluster::connect(
            &configuration.database.connection_string(),
            &configuration.database.username,
            &configuration.database.password,
        );

        CouchbaseConnection {
            cluster: Arc::new(cluster),
            bucket_name: configuration.database.test_bucket_name.clone(),
            scope_name: configuration.database.test_scope_name.clone(),
            collection_name: configuration.database.test_collection_name.clone(),
        }
    }
}

pub async fn run(
    listener: TcpListener,
    connection_data: CouchbaseConnection,
) -> Result<Server, std::io::Error> {
    let connection_data = web::Data::new(connection_data);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(transactions)
            .service(transactions_by_type)
            .service(hello)
            .app_data(connection_data.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
