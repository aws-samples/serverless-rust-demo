use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    Body, IntoResponse, Request, Response,
};
use products::{get_store, setup_tracing, Product, Store};
use serde_json::json;
use tracing::{info, instrument, warn};

type StoreSync = dyn Store + Send + Sync;
type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), E> {
    // Initialize logger
    setup_tracing();

    // Initialize DynamoDB store
    let store = get_store().await;

    // Run Lambda function
    lambda_runtime::run(handler(|event: Request, ctx: Context| {
        put_product(&store, event, ctx)
    }))
    .await?;
    Ok(())
}

#[instrument(skip(store))]
async fn put_product(
    store: &StoreSync,
    event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    // Parse body from request
    let body = match event.body() {
        Body::Text(body) => body.to_owned(),
        Body::Binary(body) => String::from_utf8(body.to_vec())?,
        _ => {
            warn!("Request body is not a string");
            return Ok(Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(json!({"message": "Bad Request"}).to_string())
                .unwrap());
        }
    };
    info!("Received product: {}", body);

    let product: Product = if let Ok(product) = serde_json::from_str(&body) {
        product
    } else {
        warn!("Failed to parse product from request body: {}", body);
        return Ok(Response::builder()
            .status(400)
            .header("Content-Type", "application/json")
            .body(json!({"message": "Bad Request"}).to_string())
            .unwrap());
    };
    info!("Parsed product: {:?}", product);

    store.put(&product).await?;
    info!("Stored product {:?}", product.id);

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(json!({"message": "OK"}).to_string())
        .unwrap())
}
