use actix_web::{get, web, HttpResponse, Responder};
use couchbase::QueryOptions;
use futures::StreamExt;
use tracing::Instrument;

use crate::{
    model::{CouchbaseTransactionWrapper, Transaction},
    CouchbaseConnection,
};

#[tracing::instrument(
    name = "Getting transactions by type for /transactions/ request",
    skip(connection_data)
)]
#[get("/transactions/{type}")]
async fn transactions_by_type(
    connection_data: web::Data<CouchbaseConnection>,
    path: web::Path<String>,
) -> impl Responder {
    let transaction_type = path.into_inner();
    let query_span =
        tracing::info_span!("Fetching {} transactions from couchbase", transaction_type);

    let query = format!(
        "SELECT * FROM `{}`.`{}`.`{}`",
        connection_data.bucket_name, connection_data.scope_name, transaction_type
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

#[tracing::instrument(
    name = "Getting transactions for /transactions/ request",
    skip(connection_data)
)]
#[get("/transactions")]
async fn transactions(connection_data: web::Data<CouchbaseConnection>) -> impl Responder {
    let query_span = tracing::info_span!("Fetching transactions from couchbase");

    let query = format!(
        "SELECT * FROM `{}`.`{}`.`bet` \
        UNION ALL \
        SELECT * FROM `{}`.`{}`.`trade`\
        UNION ALL \
        SELECT * FROM `{}`.`{}`.`deposit` \
        UNION ALL \
        SELECT * FROM `{}`.`{}`.`withdrawal`",
        connection_data.bucket_name,
        connection_data.scope_name,
        connection_data.bucket_name,
        connection_data.scope_name,
        connection_data.bucket_name,
        connection_data.scope_name,
        connection_data.bucket_name,
        connection_data.scope_name
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
