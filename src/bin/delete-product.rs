use lambda_http::{
    ext::RequestExt,
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request,
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
    // the `delete_product` function.
    // See https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
    //
    // This uses a closure to pass the Service without having to reinstantiate
    // it for every call. This is a bit of a hack, but it's the only way to
    // pass a service to a lambda function.
    //
    // Furthermore, we don't await the result of `delete_product` because
    // async closures aren't stable yet. This way, the closure returns a Future,
    // which matches the signature of the lambda function.
    // See https://github.com/rust-lang/rust/issues/62290
    lambda_runtime::run(handler(|event: Request, ctx: Context| {
        delete_product(&service, event, ctx)
    }))
    .await?;
    Ok(())
}

/// Delete a product
#[instrument(skip(service))]
async fn delete_product(
    service: &Service,
    event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    // Retrieve product ID from event
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

    // Delete product
    info!("Deleting product {}", id);
    let res = service.delete_product(id).await;

    // Return response
    //
    // The service returns a Result based on the success of the operation. If
    // the operation was successful, the Result is Ok(()), otherwise it will
    // contain an Err with the reason.
    match res {
        Ok(_) => {
            info!("Product {} deleted", id);
            Ok(response(
                200,
                json!({"message": "Product deleted"}).to_string(),
            ))
        }
        Err(err) => {
            // Log the error message
            error!("Error deleting the product {}: {}", id, err);
            Ok(response(
                500,
                json!({"message": "Failed to delete product"}).to_string(),
            ))
        }
    }
}
