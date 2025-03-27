use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use reqwest::Url;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub struct Instrumentation {
    meter_provider: Option<SdkMeterProvider>,
    service_name: String,
    otel_collector_endpoint: Option<Url>,
}

impl Instrumentation {
    pub fn new(service_name: String) -> Self {
        Self {
            meter_provider: None,
            service_name,
            otel_collector_endpoint: std::env::var("OTEL_COLLECTOR_ENDPOINT")
                .ok()
                .map(|endpoint| Url::parse(&endpoint).expect("Invalid OTEL_COLLECTOR_ENDPOINT")),
        }
    }

    pub fn setup(&mut self) -> anyhow::Result<()> {
        let otel_endpoint = self.otel_collector_endpoint.clone();
        if otel_endpoint.is_none() {
            return Ok(());
        }

        // Set up tracing subscriber without OpenTelemetry
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry().with(env_filter).init();

        // Set up just the metrics exporter
        let export_config = ExportConfig {
            // Can use unwrap here because we know otel_endpoint exists from the check above
            endpoint: otel_endpoint.unwrap().to_string(),
            ..ExportConfig::default()
        };

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_export_config(export_config)
            .build_metrics_exporter(
                Box::new(DefaultAggregationSelector::new()),
                Box::new(DefaultTemporalitySelector::new()),
            )
            .expect("Failed to build metrics exporter");

        let reader = PeriodicReader::builder(exporter, runtime::Tokio)
            .with_interval(Duration::from_secs(5))
            .build();

        let meter_provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                format!("{}{}", self.service_name, "_meter_service"),
            )]))
            .build();

        global::set_meter_provider(meter_provider.clone());
        self.meter_provider = Some(meter_provider);

        Ok(())
    }
}
