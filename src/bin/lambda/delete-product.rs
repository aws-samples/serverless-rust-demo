use lambda_http::{service_fn, Request};
use products::{entrypoints::lambda::apigateway::delete_product, utils::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Initialize logger
    setup_tracing();

    // Initialize store
    let store = get_store().await;

    // Run the Lambda function
    //
    // This is the entry point for the Lambda function. The `lambda_http`
    // crate will take care of contacting the Lambda runtime API and invoking
    // the `delete_product` function.
    // See https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
    //
    // This uses a closure to pass the Service without having to reinstantiate
    // it for every call. This is a bit of a hack, but it's the only way to
    // pass a store to a lambda function.
    //
    // Furthermore, we don't await the result of `delete_product` because
    // async closures aren't stable yet. This way, the closure returns a Future,
    // which matches the signature of the lambda function.
    // See https://github.com/rust-lang/rust/issues/62290
    lambda_http::run(service_fn(|event: Request| delete_product(&store, event))).await?;
    Ok(())
}
