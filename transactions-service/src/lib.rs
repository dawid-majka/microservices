use std::{net::TcpListener, sync::Arc};

use actix_web::{dev::Server, get, web, App, HttpResponse, HttpServer, Responder};
use configuration::Settings;
use couchbase::{Cluster, QueryOptions};
use futures::stream::StreamExt;
use tracing::Instrument;
use tracing_actix_web::TracingLogger;

use crate::model::{CouchbaseTransactionWrapper, Transaction};

pub mod configuration;
pub mod model;
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

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
}

#[tracing::instrument(
    name = "Getting transactions for /transactions request",
    skip(connection_data)
)]
#[get("/transactions")]
async fn transactions(connection_data: web::Data<CouchbaseConnection>) -> impl Responder {
    let query_span = tracing::info_span!("Fetching transactions from couchbase");

    let query = format!(
        "SELECT * FROM `{}`.`{}`.`{}`",
        connection_data.bucket_name, connection_data.scope_name, connection_data.collection_name
    );

    let result = connection_data
        .cluster
        .query(query, QueryOptions::default())
        .instrument(query_span)
        .await;

    let mut response_rows: Vec<Transaction> = vec![];

    match result {
        Ok(mut data) => {
            let mut rows = data.rows::<CouchbaseTransactionWrapper>();

            while let Some(row) = rows.next().await {
                match row {
                    Ok(wrapper) => {
                        for (_, transaction) in wrapper.inner {
                            response_rows.push(transaction);
                        }
                    }
                    Err(e) => tracing::error!("Error in row: {}", e),
                }
            }
        }
        Err(e) => tracing::error!("Query error: {}", e),
    }

    HttpResponse::Ok().json(response_rows)
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
            .service(hello)
            .app_data(connection_data.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
