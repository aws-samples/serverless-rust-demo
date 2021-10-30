use crate::{store, event_bus, product, Service};
use lambda_http::Response;
use tracing::{info, instrument};

/// Setup tracing
pub fn setup_tracing() {
    let subscriber = tracing_subscriber::fmt().json().finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}

/// Retrieve a service
///
/// This is pre-configured to configure a service with DynamoDB store as backend.
#[instrument]
pub async fn get_service() -> Service {
    // Get AWS Configuration
    let config = aws_config::load_from_env().await;

    // Initialize a DynamoDB store
    let client = aws_sdk_dynamodb::Client::new(&config);
    let table_name = std::env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    info!(
        "Initializing DynamoDB store with table name: {}",
        table_name
    );
    let store = Box::new(store::DynamoDBStore::new(client, table_name));

    // Initialize an EventBridge if the environment variable is set
    let event_bus: Box<dyn event_bus::EventBus<E = product::Event> + Sync + Send> = match std::env::var("EVENT_BUS_NAME") {
        Ok(event_bus_name) => {
            info!("Initializing EventBridge bus with name: {}", event_bus_name);
            let client = aws_sdk_eventbridge::Client::new(&config);
            Box::new(event_bus::EventBridgeBus::new(client, event_bus_name))
        }
        Err(_) => {
            info!("No EventBridge bus configured");
            Box::new(event_bus::VoidBus::new())
        }
    };

    // Return a service with the store
    Service::new(store, event_bus)
}

/// HTTP Response with a JSON payload
pub fn response(status_code: u16, body: String) -> Response<String> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}
