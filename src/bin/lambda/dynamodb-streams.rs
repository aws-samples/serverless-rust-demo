use lambda_runtime::{service_fn, LambdaEvent};
use products::{
    entrypoints::lambda::dynamodb::{model::DynamoDBEvent, parse_events},
    utils::*,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Initialize logger
    setup_tracing();

    // Initialize event bus
    let event_bus = get_event_bus().await;

    // Run the Lambda function
    //
    // This is the entry point for the Lambda function. The `lambda_runtime`
    // crate will take care of contacting the Lambda runtime API and invoking
    // the `parse_events` function.
    // See https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
    //
    // This uses a closure to pass the Service without having to reinstantiate
    // it for every call. This is a bit of a hack, but it's the only way to
    // pass the event bus to a lambda function.
    //
    // Furthermore, we don't await the result of `parse_events` because
    // async closures aren't stable yet. This way, the closure returns a Future,
    // which matches the signature of the lambda function.
    // See https://github.com/rust-lang/rust/issues/62290
    lambda_runtime::run(service_fn(|event: LambdaEvent<DynamoDBEvent>| {
        let (event, ctx) = event.into_parts();
        parse_events(&event_bus, event, ctx)
    }))
    .await?;
    Ok(())
}
