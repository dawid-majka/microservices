use std::{net::TcpListener, time::Duration};

use couchbase::{
    Collection, CollectionSpec, CreateCollectionOptions, CreatePrimaryQueryIndexOptions,
    CreateScopeOptions, DropScopeOptions, GetAllQueryIndexOptions, Scope, UpsertOptions,
};
use once_cell::sync::Lazy;
use rand::Rng;
use tokio::time::sleep;
use transactions_service::{
    configuration::get_configuration,
    model::Transaction,
    telemetry::{get_subscriber, init_subscriber},
    CouchbaseConnection,
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub connection_data: CouchbaseConnection,
}

#[actix_web::test]
async fn servers_is_working() {
    // Given
    let app_data = spawn_app("test".to_string()).await;
    let client = reqwest::Client::new();

    // When
    let response = client
        .get(&format!("{}/", &app_data.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Then
    assert!(response.status().is_success());
}

#[actix_web::test]
async fn get_transactions_returns_empty_json_when_no_rows() {
    // Given
    let client = reqwest::Client::new();
    let app_data = spawn_app("test".to_string()).await;
    let mut con = app_data.connection_data;
    create_scope(&con).await;
    con.collection_name = "withdrawal".to_string();
    create_collection(&con).await;
    manage_db_indexing(&con).await;

    // When
    let response = client
        .get(&format!(
            "{}/transactions/{}",
            &app_data.address, con.collection_name
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Then
    assert_eq!(200, response.status().as_u16());

    drop_scope(&con).await
}

#[actix_web::test]
async fn get_transactions_by_type_returns_a_transactions_from_db() {
    // Given
    let mut rng = rand::thread_rng();
    let test_id: u32 = rng.gen();
    let scope_name = format!("{}test", test_id);

    let app_data = spawn_app(scope_name.clone()).await;
    let mut con = app_data.connection_data;

    let client = reqwest::Client::new();

    create_scope(&con).await;
    con.collection_name = "withdrawal".to_string();
    let collection = create_collection(&con).await;
    sleep(Duration::from_secs(5)).await;

    manage_db_indexing(&con).await;

    let transaction: Transaction = serde_json::from_str(r#"{"id":1447241290163152320,"user_id":18107235828171665340,"amount":678.7329504848955,"transaction_type":"Withdrawal"}"#).expect("Error deserializing the message");

    collection
        .upsert(
            transaction.id.to_string(),
            transaction.clone(),
            UpsertOptions::default(),
        )
        .await
        .expect("Error upserting transaction");

    sleep(Duration::from_secs(5)).await;

    // When
    let response = client
        .get(&format!(
            "{}/transactions/{}",
            &app_data.address, con.collection_name
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Then
    assert_eq!(200, response.status().as_u16());

    let response_body: Vec<Transaction> = response
        .json()
        .await
        .expect("Failed to deserialize response");

    assert_eq!(1, response_body.len());

    drop_scope(&con).await;
}

async fn spawn_app(scope_name: String) -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let mut connection_data = CouchbaseConnection::test_connection(&configuration);
    connection_data.scope_name = scope_name;

    let server = transactions_service::run(listener, connection_data.clone())
        .await
        .expect("Server initialization failed.");

    tokio::spawn(server);

    TestApp {
        address,
        connection_data,
    }
}

async fn create_collection(con: &CouchbaseConnection) -> Collection {
    let bucket = con.cluster.bucket(&con.bucket_name);
    let mgr = bucket.collections();

    match mgr
        .create_collection(
            CollectionSpec::new(
                &con.collection_name,
                &con.scope_name,
                Duration::from_secs(0),
            ),
            CreateCollectionOptions::default(),
        )
        .await
    {
        Ok(_result) => {
            tracing::debug!("Collection created");
        }
        Err(e) => tracing::debug!("Create collection error: {}", e),
    }

    bucket
        .scope(&con.scope_name)
        .collection(&con.collection_name)
}

async fn create_scope(con: &CouchbaseConnection) -> Scope {
    let bucket = con.cluster.bucket(&con.bucket_name);
    let mgr = bucket.collections();

    match mgr
        .create_scope(&con.scope_name, CreateScopeOptions::default())
        .await
    {
        Ok(_result) => {
            tracing::debug!("Scope created");
        }
        Err(e) => tracing::debug!("Create scope error: {}", e),
    }

    bucket.scope(&con.scope_name)
}

async fn drop_scope(con: &CouchbaseConnection) {
    let bucket = con.cluster.bucket(&con.bucket_name);
    let mgr = bucket.collections();

    match mgr
        .drop_scope(&con.scope_name, DropScopeOptions::default())
        .await
    {
        Ok(_) => {
            tracing::debug!("{} scope deleted", &con.collection_name);
        }
        Err(e) => {
            tracing::error!("Error deleting scope: {:?}", e)
        }
    }
}

async fn manage_db_indexing(connection: &CouchbaseConnection) {
    let index_manager = connection.cluster.query_indexes();

    let name = format!(
        "{}`.`{}`.`{}",
        &connection.bucket_name, &connection.scope_name, &connection.collection_name
    );

    match index_manager
        .get_all_indexes(&name, GetAllQueryIndexOptions::default())
        .await
    {
        Ok(results) => {
            if !results.into_iter().any(|index| index.is_primary()) {
                match index_manager
                    .create_primary_index(&name, CreatePrimaryQueryIndexOptions::default())
                    .await
                {
                    Ok(_result) => {
                        tracing::debug!("primary index created");
                    }
                    Err(e) => tracing::error!("got error! {}", e),
                }
            }
        }

        Err(e) => {
            tracing::error!("got error! {}", e)
        }
    };
}
