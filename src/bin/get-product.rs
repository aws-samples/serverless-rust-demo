use lambda_http::{
    ext::RequestExt,
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request, Response,
};
use products::{get_service, setup_tracing, Service};
use serde_json::json;
use tracing::{info, instrument, warn};

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), E> {
    // Initialize logger
    setup_tracing();

    // Initialize service
    let service = get_service().await;

    // Run Lambda function
    lambda_runtime::run(handler(|event: Request, ctx: Context| {
        get_product(&service, event, ctx)
    }))
    .await?;
    Ok(())
}

#[instrument(skip(service))]
async fn get_product(
    service: &Service,
    event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    // Retrieve product ID from event
    let path_parameters = event.path_parameters();
    let id = path_parameters.get("id").expect("id must be set");
    info!("Fetching product {}", id);
    let product = service.get_product(id).await?;

    Ok(match product {
        Some(product) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(json!(product).to_string())
            .unwrap(),
        None => {
            warn!("Product not found: {}", id);
            Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(json!({"message": "Product not found"}).to_string())
                .unwrap()
        }
    })
}
