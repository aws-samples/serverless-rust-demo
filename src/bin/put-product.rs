use lambda_http::{
    ext::RequestExt,
    handler,
    lambda_runtime::{self, Context},
    Body, IntoResponse, Request,
};
use products::{utils::*, Product, Service};
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
    // the `put_product` function.
    // See https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html
    //
    // This uses a closure to pass the Service without having to reinstantiate
    // it for every call. This is a bit of a hack, but it's the only way to
    // pass a service to a lambda function.
    //
    // Furthermore, we don't await the result of `put_product` because
    // async closures aren't stable yet. This way, the closure returns a Future,
    // which matches the signature of the lambda function.
    // See https://github.com/rust-lang/rust/issues/62290
    lambda_runtime::run(handler(|event: Request, ctx: Context| {
        put_product(&service, event, ctx)
    }))
    .await?;
    Ok(())
}

/// Put a product
#[instrument(skip(service))]
async fn put_product(
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

    // Read body from request
    let body = match event.body() {
        Body::Text(body) => body.to_owned(),
        Body::Binary(body) => String::from_utf8(body.to_vec())?,
        _ => {
            warn!("Request body is not a string");
            return Ok(response(
                400,
                json!({"message": "Request body is not a string"}).to_string(),
            ));
        }
    };
    info!("Received product: {}", body);

    // Parse product from body
    let product: Product = if let Ok(product) = serde_json::from_str(&body) {
        product
    } else {
        warn!("Failed to parse product from request body: {}", body);
        return Ok(response(
            400,
            json!({"message": "Failed to parse product from request body"}).to_string(),
        ));
    };
    info!("Parsed product: {:?}", product);

    // Compare product ID with product ID in body
    if product.id != id {
        warn!(
            "Product ID in path ({}) does not match product ID in body ({})",
            id, product.id
        );
        return Ok(response(
            400,
            json!({"message": "Product ID in path does not match product ID in body"}).to_string(),
        ));
    }

    // Put product
    let res = service.put_product(&product).await;

    // Return response
    //
    // If the put was successful, we return a 201 Created. Otherwise, we return
    // a 500 Internal Server Error.
    Ok(match res {
        // Product created
        Ok(_) => {
            info!("Created product {:?}", product.id);
            response(201, json!({"message": "Product created"}).to_string())
        }
        // Error creating product
        Err(err) => {
            error!("Failed to create product {}: {}", product.id, err);
            response(
                500,
                json!({"message": "Failed to create product"}).to_string(),
            )
        }
    })
}
