use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request, Response,
};
use products::{get_store, setup_tracing, Store};
use serde_json::json;
use tracing::{error, instrument};

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
        get_products(&store, event, ctx)
    }))
    .await?;
    Ok(())
}

#[instrument(skip(store))]
async fn get_products(
    store: &StoreSync,
    _event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    Ok(match store.all(None).await {
        Ok(res) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(json!(res).to_string())
            .unwrap(),
        Err(err) => {
            error!("Something went wrong: {:?}", err);
            Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(json!({ "message": format!("Something went wrong: {:?}", err) }).to_string())
                .unwrap()
        }
    })
}
