use opentelemetry::global;
use opentelemetry::metrics::{Counter, Gauge, Histogram};
use std::time::Instant;

#[derive(Clone)]
pub struct MempoolMetrics {
    pub mempool_txn_count_gauge: Gauge<u64>,
    pub mempool_txn_count_histogram: Histogram<u64>,
}
impl MempoolMetrics {
    pub fn init() -> Self {
        let metrics_meter = global::meter("nexus-metrics");
        let mempool_txn_count_gauge = metrics_meter.u64_gauge("mempool_txn_count").with_unit("Transactions").init();
        let mempool_txn_count_histogram = metrics_meter
            .u64_histogram("mempool_txn_count_histogram")
            .with_unit("Transactions")
            .init();
        Self {
            mempool_txn_count_gauge,
            mempool_txn_count_histogram,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExecutionMetrics {
    pub batch_execution_time: Histogram<u64>,
    pub total_batch_execution_time: Histogram<u64>,
    pub batch_number_counter: Counter<u64>,
    pub number_of_transactions_batch: Histogram<u64>,
}
impl ExecutionMetrics {
    pub fn init() -> Self {
        let metrics_meter = global::meter("nexus-metrics");
        let batch_execution_time = metrics_meter.u64_histogram("batch_execution_time").init();
        let total_batch_execution_time = metrics_meter.u64_histogram("total_batch_execution_time").init();
        let batch_number_counter = metrics_meter.u64_counter("block_number").with_unit("Blocks").init();
        let number_of_transactions_batch = metrics_meter.u64_histogram("number_of_transactions").init();
        Self {
            batch_execution_time,
            total_batch_execution_time,
            batch_number_counter,
            number_of_transactions_batch,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProvingMetrics {
    pub batch_proving_time: Histogram<u64>,
}
impl ProvingMetrics {
    pub fn init() -> Self {
        let metrics_meter = global::meter("nexus-prover-metrics");
        let batch_proving_time = metrics_meter.u64_histogram("batch_proving_time").init();
        Self { batch_proving_time }
    }
}

#[derive(Clone)]
pub struct ApiMetrics {
    submit_tx_counter: Counter<u64>,
    tx_status_counter: Counter<u64>,
    get_block_counter: Counter<u64>,
    get_state_counter: Counter<u64>,
    get_state_hex_counter: Counter<u64>,
    get_header_counter: Counter<u64>,
    range_counter: Counter<u64>,
    block_proof_counter: Counter<u64>,

    submit_tx_response_time: Histogram<f64>,
    tx_status_response_time: Histogram<f64>,
    get_block_response_time: Histogram<f64>,
    get_state_response_time: Histogram<f64>,
    get_state_hex_response_time: Histogram<f64>,
    get_header_response_time: Histogram<f64>,
    range_response_time: Histogram<f64>,
    block_proof_response_time: Histogram<f64>,
}

impl ApiMetrics {
    pub fn new() -> Self {
        let meter = global::meter("nexus_api");

        let submit_tx_counter = meter
            .u64_counter("nexus_api_submit_tx_requests")
            .with_description("Number of transaction submission requests")
            .init();

        let tx_status_counter = meter
            .u64_counter("nexus_api_tx_status_requests")
            .with_description("Number of transaction status requests")
            .init();

        let get_block_counter = meter
            .u64_counter("nexus_api_get_block_requests")
            .with_description("Number of block retrieval requests")
            .init();

        let get_state_counter = meter
            .u64_counter("nexus_api_get_state_requests")
            .with_description("Number of account state requests")
            .init();

        let get_state_hex_counter = meter
            .u64_counter("nexus_api_get_state_hex_requests")
            .with_description("Number of hex account state requests")
            .init();

        let get_header_counter = meter
            .u64_counter("nexus_api_get_header_requests")
            .with_description("Number of header requests")
            .init();

        let range_counter = meter
            .u64_counter("nexus_api_range_requests")
            .with_description("Number of block range requests")
            .init();

        let block_proof_counter = meter
            .u64_counter("nexus_api_get_block_proof_requests")
            .with_description("Number of block proof requests")
            .init();

        // Create histograms for response times (in milliseconds)
        let submit_tx_response_time = meter
            .f64_histogram("nexus_api_submit_tx_response_time")
            .with_description("Response time for transaction submission")
            .with_unit("ms")
            .init();

        let tx_status_response_time = meter
            .f64_histogram("nexus_api_tx_status_response_time")
            .with_description("Response time for transaction status requests")
            .with_unit("ms")
            .init();

        let get_block_response_time = meter
            .f64_histogram("nexus_api_get_block_response_time")
            .with_description("Response time for block retrieval")
            .with_unit("ms")
            .init();

        let get_state_response_time = meter
            .f64_histogram("nexus_api_get_state_response_time")
            .with_description("Response time for account state requests")
            .with_unit("ms")
            .init();

        let get_state_hex_response_time = meter
            .f64_histogram("nexus_api_get_state_hex_response_time")
            .with_description("Response time for hex account state requests")
            .with_unit("ms")
            .init();

        let get_header_response_time = meter
            .f64_histogram("nexus_api_get_header_response_time")
            .with_description("Response time for header requests")
            .with_unit("ms")
            .init();

        let range_response_time = meter
            .f64_histogram("nexus_api_range_response_time")
            .with_description("Response time for block range requests")
            .with_unit("ms")
            .init();

        let block_proof_response_time = meter
            .f64_histogram("nexus_api_block_proof_response_time")
            .with_description("Response time for block proof requests")
            .init();

        Self {
            submit_tx_counter,
            tx_status_counter,
            get_block_counter,
            get_state_counter,
            get_state_hex_counter,
            get_header_counter,
            range_counter,
            block_proof_counter,

            submit_tx_response_time,
            tx_status_response_time,
            get_block_response_time,
            get_state_response_time,
            get_state_hex_response_time,
            get_header_response_time,
            range_response_time,
            block_proof_response_time,
        }
    }

    // Measure response time and increment counter for submit_tx endpoint
    pub fn record_submit_tx_request(&self, start_time: Instant) {
        self.submit_tx_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.submit_tx_response_time.record(duration, &[]);
    }

    // Measure response time and increment counter for tx_status endpoint
    pub fn record_tx_status_request(&self, start_time: Instant) {
        self.tx_status_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.tx_status_response_time.record(duration, &[]);
    }

    // Measure response time and increment counter for get_block endpoint
    pub fn record_get_block_request(&self, start_time: Instant) {
        self.get_block_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.get_block_response_time.record(duration, &[]);
    }

    // Measure response time and increment counter for get_state endpoint
    pub fn record_get_state_request(&self, start_time: Instant) {
        self.get_state_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.get_state_response_time.record(duration, &[]);
    }

    // Measure response time and increment counter for get_state_hex endpoint
    pub fn record_get_state_hex_request(&self, start_time: Instant) {
        self.get_state_hex_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.get_state_hex_response_time.record(duration, &[]);
    }

    // Measure response time and increment counter for get_header endpoint
    pub fn record_get_header_request(&self, start_time: Instant) {
        self.get_header_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.get_header_response_time.record(duration, &[]);
    }

    // Measure response time and increment counter for range endpoint
    pub fn record_range_request(&self, start_time: Instant) {
        self.range_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.range_response_time.record(duration, &[]);
    }

    // Measure response time and increment counter for block_proof endpoint
    pub fn record_block_proof_request(&self, start_time: Instant) {
        self.range_counter.add(1, &[]);
        let duration = start_time.elapsed().as_millis() as f64;
        self.block_proof_response_time.record(duration, &[]);
    }
}
