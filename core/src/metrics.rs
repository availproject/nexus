use opentelemetry::global;
use opentelemetry::metrics::{Counter, Gauge};

#[derive(Clone)]
pub struct BlockNumberMetrics {
    pub block_number_counter: Counter<u64>,
}
impl BlockNumberMetrics {
    fn init() -> Self {
        let metrics_meter = global::meter("nexus-metrics");
        let block_number_counter = metrics_meter.u64_counter("block_number").with_unit("Blocks").init();
        Self {
            block_number_counter,
        }
    }
}

#[derive(Clone)]
pub struct MempoolMetrics {
    pub mempool_txn_count_gauge: Gauge<u64>,
}
impl MempoolMetrics {
    pub(crate) fn init() -> Self {
        let metrics_meter = global::meter("nexus-metrics");
        let mempool_txn_count_gauge = metrics_meter.u64_gauge("mempool_txn_count").with_unit("Transactions").init();
        Self {
            mempool_txn_count_gauge,
        }
    }
}
