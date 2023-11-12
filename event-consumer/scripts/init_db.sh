#!/usr/bin/env bash
set -x
set -eo pipefail

# Check if Couchbase CLI is installed
if ! [ -x "$(command -v couchbase-cli)" ]; then
    echo >&2 "Error: couchbase-cli is not installed."
    exit 1
fi

# Define environment variables
CB_HOST="localhost"
CB_PORT=8091
CB_USERNAME="Administrator"
CB_PASSWORD="password"
BUCKET_NAME="transactions"
SCOPE_NAME="transactions"
COLLECTION_NAME="transactions"
TEST_BUCKET_NAME="test_data"
TEST_SCOPE_NAME="test_data"

# Function to wait for Couchbase to be ready
wait_for_couchbase() {
    echo "Waiting for Couchbase to start..."
    for i in {1..30}; do  # Retry for 30 times with 5 seconds interval
        if curl -sf http://localhost:8091/ui/index.html &> /dev/null; then
            echo "Couchbase Web UI is up and running."
            return 0
        fi
        echo "Waiting for Couchbase to start (attempt: $i)..."
        sleep 5
    done
    echo "Couchbase did not start in time."
    exit 1
}

wait_for_cbq_service() {
    echo "Waiting for Couchbase Query Service to start..."
    for i in {1..30}; do  # Retry for 30 times with 5 seconds interval
        # Attempt to run a simple N1QL query and check for a successful response
        if cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "SELECT RAW 'Couchbase is ready' AS status;" | grep -q 'Couchbase is ready'; then
            echo "Couchbase Query Service is up and running."
            return 0
        fi
        echo "Waiting for Couchbase Query Service to start (attempt: $i)..."
        sleep 5
    done
    echo "Couchbase Query Service did not start in time."
    exit 1
}


# Wait for Couchbase to be ready
wait_for_couchbase

# Initialize a new cluster
if ! couchbase-cli bucket-list -c ${CB_HOST}:${CB_PORT} -u ${CB_USERNAME} -p ${CB_PASSWORD} | grep -q ${BUCKET_NAME}; then
    couchbase-cli cluster-init -c ${CB_HOST}:${CB_PORT} \
        --cluster-username ${CB_USERNAME} \
        --cluster-password ${CB_PASSWORD} \
        --services data,index,query \
        --cluster-ramsize 1024 \
        --cluster-index-ramsize 256 \
        --cluster-fts-ramsize 256 \
        --cluster-eventing-ramsize 256 \
        --cluster-analytics-ramsize 1024
fi

# Create a transactions bucket
if ! couchbase-cli bucket-list -c ${CB_HOST}:${CB_PORT} -u ${CB_USERNAME} -p ${CB_PASSWORD} | grep -q ${BUCKET_NAME}; then
    couchbase-cli bucket-create -c ${CB_HOST}:${CB_PORT} \
        -u ${CB_USERNAME} -p ${CB_PASSWORD} \
        --bucket ${BUCKET_NAME} \
        --bucket-type couchbase \
        --bucket-ramsize 256
fi

# Create a test bucket
if ! couchbase-cli bucket-list -c ${CB_HOST}:${CB_PORT} -u ${CB_USERNAME} -p ${CB_PASSWORD} | grep -q ${TEST_BUCKET_NAME}; then
    couchbase-cli bucket-create -c ${CB_HOST}:${CB_PORT} \
        -u ${CB_USERNAME} -p ${CB_PASSWORD} \
        --bucket ${TEST_BUCKET_NAME} \
        --bucket-type couchbase \
        --bucket-ramsize 256
fi

wait_for_cbq_service

# Check if transactions scope exists
if cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "SELECT * FROM system:keyspaces WHERE name='${SCOPE_NAME}' AND bucket_id='${BUCKET_NAME}';" | grep -q '${SCOPE_NAME}'; then
    echo "Scope ${SCOPE_NAME} already exists in ${BUCKET_NAME}"
else
    # Create scope
    cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "CREATE SCOPE \`${BUCKET_NAME}\`.\`${SCOPE_NAME}\`;"
    echo "Scope ${SCOPE_NAME} created in ${BUCKET_NAME}"
fi

# Check if transactions collection exists
if cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "SELECT * FROM system:keyspaces WHERE name='${COLLECTION_NAME}' AND scope_id='${SCOPE_NAME}';" | grep -q '${COLLECTION_NAME}'; then
    echo "Collection ${COLLECTION_NAME} already exists in scope ${SCOPE_NAME}"
else
    # Create collection
    cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "CREATE COLLECTION \`${BUCKET_NAME}\`.\`${SCOPE_NAME}\`.\`${COLLECTION_NAME}\`;"
    echo "Collection ${COLLECTION_NAME} created in scope ${SCOPE_NAME}"
fi

# Check if test scope exists
if cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "SELECT * FROM system:keyspaces WHERE name='${TEST_SCOPE_NAME}' AND bucket_id='${TEST_BUCKET_NAME}';" | grep -q '${TEST_SCOPE_NAME}'; then
    echo "Scope ${TEST_SCOPE_NAME} already exists in ${TEST_BUCKET_NAME}"
else
    # Create scope
    cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "CREATE SCOPE \`${TEST_BUCKET_NAME}\`.\`${TEST_SCOPE_NAME}\`;"
    echo "Scope ${TEST_SCOPE_NAME} created in ${TEST_BUCKET_NAME}"
fi

# Check if the primary index already exists
INDEX_EXISTS=$(cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} -e http://localhost:8091 -q -s "SELECT COUNT(*) AS count FROM system:indexes WHERE keyspace_id='${BUCKET_NAME}' AND is_primary=true;")
if ! echo "$INDEX_EXISTS" | grep -o '"count": 1' > /dev/null; then
    cbq -u ${CB_USERNAME} -p ${CB_PASSWORD} \
        -s "CREATE PRIMARY INDEX ON \`${BUCKET_NAME}\`.\`${SCOPE_NAME}\`.\`${COLLECTION_NAME}\` USING GSI;"
fi
