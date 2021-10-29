use lambda_http::{
    ext::RequestExt,
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request, Response,
};
use products::{utils::*, Service};
use serde_json::json;
use tracing::{error, info, instrument, warn};

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), E> {
    // Initialize logger
    setup_tracing();

    // Initialize service
    let service = get_service().await;

    // Run the Lambda function
    //
    // This is the entry point for the Lambda function. The `lambda_runtime`
    // crate will take care of contacting the Lambda runtime API and invoking
    // the `get_product` function.
    // See https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
    //
    // This uses a closure to pass the Service without having to reinstantiate
    // it for every call. This is a bit of a hack, but it's the only way to
    // pass a service to a lambda function.
    //
    // Furthermore, we don't await the result of `get_product` because
    // async closures aren't stable yet. This way, the closure returns a Future,
    // which matches the signature of the lambda function.
    // See https://github.com/rust-lang/rust/issues/62290
    lambda_runtime::run(handler(|event: Request, ctx: Context| {
        get_product(&service, event, ctx)
    }))
    .await?;
    Ok(())
}

/// Get a product
#[instrument(skip(service))]
async fn get_product(
    service: &Service,
    event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    // Retrieve product ID from event.
    //
    // If the event doesn't contain a product ID, we return a 400 Bad Request.
    let path_parameters = event.path_parameters();
    let id = match path_parameters.get("id") {
        Some(id) => id,
        None => {
            warn!("Missing 'id' parameter in path");
            return Ok(response(
                400,
                json!({ "message": "Missing 'id' parameter in path" }).to_string(),
            ));
        }
    };

    // Retrieve product
    info!("Fetching product {}", id);
    let product = service.get_product(id).await;

    // Return response
    //
    // Since the service returns an `Option` within a `Result`, there are three
    // potential scenarios: the product exists, it doesn't exist, or there was
    // an error.
    Ok(match product {
        // Product exists
        Ok(Some(product)) => Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(json!(product).to_string())
            .unwrap(),
        // Product doesn't exist
        Ok(None) => {
            warn!("Product not found: {}", id);
            response(404, json!({"message": "Product not found"}).to_string())
        }
        // Error
        Err(err) => {
            error!("Error fetching product: {}", err);
            response(
                500,
                json!({"message": "Error fetching product"}).to_string(),
            )
        }
    })
}
