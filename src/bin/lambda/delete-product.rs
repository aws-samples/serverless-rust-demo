use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    Request,
};
use products::{entrypoints::lambda::apigateway::delete_product, utils::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
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
        let ctx_string = serde_json::to_string(&ctx).unwrap();
        let ctx_str = ctx_string.as_str();
        let span = tracing::span!(tracing::Level::TRACE, "lambda_handler", lambda_context = ctx_str);
        let _guard = span.enter();

        delete_product(&service, event, ctx)
    }))
    .await?;
    Ok(())
}
