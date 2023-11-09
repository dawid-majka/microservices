use std::{net::TcpListener, sync::Arc};

use actix_web::{dev::Server, get, web, App, HttpResponse, HttpServer, Responder};
use couchbase::{Cluster, CreatePrimaryQueryIndexOptions, GetAllQueryIndexOptions, QueryOptions};
use futures::stream::StreamExt;
use serde_json::Value;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/transaction")]
async fn transactions(cluster: web::Data<Arc<Cluster>>) -> impl Responder {
    let result = cluster
        .query("SELECT * FROM `transactions`", QueryOptions::default())
        .await;

    let mut response_rows = vec![];

    match result {
        Ok(mut data) => {
            let mut rows = data.rows::<Value>();
            while let Some(row) = rows.next().await {
                match row {
                    Ok(ok_row) => response_rows.push(ok_row),
                    Err(e) => println!("Error in row: {}", e),
                }
            }
        }
        Err(e) => println!("Query error: {}", e),
    }

    println!("Returned rows: {:?}", response_rows);

    HttpResponse::Ok().json(response_rows)
}

pub async fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let cluster = Cluster::connect("couchbase://localhost:8091", "Administrator", "password");

    //TODO: Should be done when creating db
    manage_db_indexing(&cluster).await;

    let cluster = Arc::new(cluster);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cluster.clone()))
            .service(transactions)
            .service(hello)
    })
    .listen(listener)?
    .run();

    Ok(server)
}

async fn manage_db_indexing(cluster: &Cluster) {
    let index_manager = cluster.query_indexes();

    match index_manager
        .get_all_indexes("transactions", GetAllQueryIndexOptions::default())
        .await
    {
        Ok(results) => {
            if !results.into_iter().any(|index| index.is_primary()) {
                match index_manager
                    .create_primary_index(
                        "transactions",
                        CreatePrimaryQueryIndexOptions::default().index_name("transactions_id"),
                    )
                    .await
                {
                    Ok(_result) => {
                        println!("primary index created");
                    }
                    Err(e) => println!("got error! {}", e),
                }
            }
        }

        Err(e) => {
            println!("got error! {}", e)
        }
    };
}
