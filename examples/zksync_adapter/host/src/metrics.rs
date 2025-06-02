use opentelemetry::{global, metrics::{Counter, Gauge, Histogram}};

#[derive(Clone, Debug)]
pub struct AdapterMetrics {
    pub block_proving_time: Histogram<u64>,
    pub latest_height: Gauge<u64>,
    pub blocks_proved: Counter<u64>
}
impl AdapterMetrics {
    pub fn init() -> Self {
        let metrics_meter = global::meter("zksync-adapter-metrics");
        let proving_block_latest_height_gauge = metrics_meter.u64_gauge("proving_block_latest_height").with_unit("Zksync-Block").init();
        let proving_block_time_histogram = metrics_meter
            .u64_histogram("proving_block_time")
            .with_unit("secs")
            .init();
        let blocks_proved = metrics_meter
            .u64_counter("zksync_block_proved")
            .with_description("Number of zksync blocks proved")
            .init();
        Self { block_proving_time: proving_block_time_histogram, latest_height: proving_block_latest_height_gauge, blocks_proved }
    }
}
