use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::BatchConfigBuilder;
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::{runtime, Resource};
use reqwest::Url;
use std::time::Duration;
use tracing::log::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
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
        let otel_endpoint = self.otel_collector_endpoint.clone().expect("OTEL_COLLECTOR_ENDPOINT not set");

        let tracing_subscriber =
            tracing_subscriber::registry().with(EnvFilter::builder().with_default_directive(LevelFilter::Info.into()).from_env()?);

        let batch_config = BatchConfigBuilder::default().with_max_export_batch_size(128).build();
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_endpoint(otel_endpoint.to_string()))
            .with_trace_config(
                Config::default().with_resource(Resource::new(vec![KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    format!("{}{}", self.service_name, "_trace_service"),
                )])),
            )
            .with_batch_config(batch_config)
            .install_batch(runtime::Tokio)
            .expect("Failed to install tracer provider");
        global::set_tracer_provider(provider.clone());
        let tracer = provider.tracer(format!("{}{}", self.service_name, "_subscriber"));

        let export_config = ExportConfig {
            endpoint: otel_endpoint.to_string(),
            ..ExportConfig::default()
        };
        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_export_config(export_config)
            .build_metrics_exporter(
                Box::new(DefaultAggregationSelector::new()),
                Box::new(DefaultTemporalitySelector::new()),
            );
        let reader = PeriodicReader::builder(
            exporter.expect("Failed to build metrics exporter"),
            runtime::Tokio,
        )
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
        tracing_subscriber.with(OpenTelemetryLayer::new(tracer)).init();

        Ok(())
    }
}
