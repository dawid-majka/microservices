## 1. Event Producer

#### Navigate to the Event Producer directory:
```bash
cd event-producer
```

### terminal 1:
#### Run Kafka Docker Image
```bash
docker compose up
```

### terminal 2:
#### Create Kafka topic
```bash
docker exec -it event-producer-kafka-1 kafka-topics --create --topic transactions --bootstrap-server localhost:29092 --partitions 1 --replication-factor 1
```

#### Start Event Producer
```bash
cargo run
```


## 2. EVENT CONSUMER

#### Navigate to the Event Consumer directory:
```bash
cd ../event-consumer
```
### terminal 1:
#### Run Couchbase Docker Image
```bash
docker compose up
```
### terminal 2:
#### Run Couchbase init script
```bash
docker exec -it couchbase_server bash -c "/opt/init_db.sh"
```

#### Start Event Consumer
```bash
cargo run
```

## 3. TRANSACTION SERVICE

#### Navigate to the Transaction Service directory:
```bash
cd ../transactions-service
```

#### Run server
```bash
cargo run
```

#### To print logs in pretty format install bunyan
```bash
cargo install bunyan
```
and 
```bash
cargo run | bunyan
```

#### Tests with logs
```bash
TEST_LOG=true cargo test | bunyan
```

#### Make request

```bash
curl http://localhost:8080/transactions
```

#### or use requests.http file if you are using REST Client vscode extension 
