use crate::{DynamoDBStore, Service};
// use opentelemetry_otlp::WithExportConfig;
use tracing::info;
// use tracing_subscriber::layer::SubscriberExt;
// use tracing_subscriber::Registry;

// Setup tracing
pub fn setup_tracing() {
    // let otlp_exporter = opentelemetry_otlp::new_exporter()
    //     .tonic()
    //     .with_endpoint("http://127.0.0.1:55680")
    //     .with_timeout(std::time::Duration::from_secs(5));
    // let tracer = opentelemetry_otlp::new_pipeline()
    //     .tracing()
    //     .with_exporter(otlp_exporter)
    //     .install_simple()
    //     .expect("Failed to create OpenTelemetry tracer");
    // let subscriber = Registry::default().with(tracing_opentelemetry::layer().with_tracer(tracer));
    let subscriber = tracing_subscriber::fmt().json().finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}

pub async fn get_service() -> Service {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let table_name = std::env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    info!(
        "Initializing DynamoDB store with table name: {}",
        table_name
    );
    let store = Box::new(DynamoDBStore::new(client, &table_name));
    Service::new(store)
}
