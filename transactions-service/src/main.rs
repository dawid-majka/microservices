use std::sync::Arc;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use couchbase::{Cluster, CreatePrimaryQueryIndexOptions, GetAllQueryIndexOptions, QueryOptions};
use futures::stream::StreamExt;
use serde_json::Value;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cluster = Cluster::connect("couchbase://localhost:8091", "Administrator", "password");

    let index_manager = cluster.query_indexes();

    let mut has_primary = false;

    // TODO:
    match index_manager
        .get_all_indexes("transactions", GetAllQueryIndexOptions::default())
        .await
    {
        Ok(result) => {
            for index in result {
                if index.is_primary() {
                    has_primary = true;
                    break;
                }
            }
            println!("Have primary? {}", has_primary);
        }
        Err(e) => println!("got error! {}", e),
    }

    if !has_primary {
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

    let cluster = Arc::new(cluster);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cluster.clone()))
            .service(transactions)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
