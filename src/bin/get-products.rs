use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request,
};
use products::{utils::*, Service};
use serde_json::json;
use tracing::{error, instrument};

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
    // the `get_products` function.
    // See https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
    //
    // This uses a closure to pass the Service without having to reinstantiate
    // it for every call. This is a bit of a hack, but it's the only way to
    // pass a service to a lambda function.
    //
    // Furthermore, we don't await the result of `get_products` because
    // async closures aren't stable yet. This way, the closure returns a Future,
    // which matches the signature of the lambda function.
    // See https://github.com/rust-lang/rust/issues/62290
    lambda_runtime::run(handler(|event: Request, ctx: Context| {
        get_products(&service, event, ctx)
    }))
    .await?;
    Ok(())
}

/// Retrieve products
#[instrument(skip(service))]
async fn get_products(
    service: &Service,
    _event: Request,
    _: Context,
) -> Result<impl IntoResponse, E> {
    // Retrieve products
    // TODO: Add pagination
    let res = service.get_products(None).await;

    // Return response
    Ok(match res {
        // Return a list of products
        Ok(res) => response(200, json!(res).to_string()),
        // Return an error
        Err(err) => {
            error!("Something went wrong: {:?}", err);
            response(
                500,
                json!({ "message": format!("Something went wrong: {:?}", err) }).to_string(),
            )
        }
    })
}
