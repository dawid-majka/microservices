Manual Testing:

1. docker compose up


2. Create a Kafka Topic:

docker exec -it event-producer-kafka-1 kafka-topics --create --topic transactions --bootstrap-server localhost:29092 --partitions 1 --replication-factor 1

3. run service

cargo run

4. Consume messages:

docker exec -it event-producer-kafka-1 kafka-console-consumer --topic transactions --from-beginning --bootstrap-server localhost:29092


5. docker compose down