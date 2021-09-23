use lambda_http::{
    ext::RequestExt,
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request, Response,
};
use products::{get_store, setup_tracing, Store};
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
        delete_product(&store, event, ctx)
    }))
    .await?;
    Ok(())
}

#[instrument(skip(store))]
async fn delete_product(
    store: &StoreSync,
    event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    // Retrieve product ID from event
    let path_parameters = event.path_parameters();
    let id = path_parameters.get("id").expect("id must be set");
    info!("Deleting product {}", id);

    match store.delete(id).await {
        Ok(_) => {
            info!("Product {} deleted", id);
            Ok(Response::builder()
                .status(200)
                .body(json!({"message": "OK"}).to_string())
                .unwrap())
        }
        Err(err) => {
            warn!("Failed to delete product {}: {}", id, err);
            Ok(Response::builder()
                .status(400)
                .body(json!({"message": "Failed to delete product"}).to_string())
                .unwrap())
        }
    }
}
