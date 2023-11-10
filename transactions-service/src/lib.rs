use std::net::TcpListener;

use actix_web::{dev::Server, get, web, App, HttpResponse, HttpServer, Responder};
use couchbase::{Cluster, CreatePrimaryQueryIndexOptions, GetAllQueryIndexOptions, QueryOptions};
use futures::stream::StreamExt;
use serde_json::Value;

pub mod configuration;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/transaction")]
async fn transactions(cluster: web::Data<Cluster>) -> impl Responder {
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


    let cluster = web::Data::new(cluster);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(cluster.clone())
            .service(transactions)
            .service(hello)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
