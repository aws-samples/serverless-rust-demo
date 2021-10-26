use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request, Response,
};
use products::{get_service, setup_tracing, Service};
use serde_json::json;
use tracing::{error, instrument};

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), E> {
    // Initialize logger
    setup_tracing();

    // Initialize service
    let service = get_service().await;

    // Run Lambda function
    lambda_runtime::run(handler(|event: Request, ctx: Context| {
        get_products(&service, event, ctx)
    }))
    .await?;
    Ok(())
}

#[instrument(skip(service))]
async fn get_products(
    service: &Service,
    _event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    Ok(match service.get_products(None).await {
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
