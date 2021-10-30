use crate::{utils::response, Product, Service};
use lambda_http::{ext::RequestExt, lambda_runtime::Context, Body, IntoResponse, Request};
use serde_json::json;
use tracing::{error, info, instrument, warn};

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

/// Delete a product
#[instrument(skip(service))]
pub async fn delete_product(
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

/// Get a product
#[instrument(skip(service))]
pub async fn get_product(
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
        Ok(Some(product)) => response(200, json!(product).to_string()),
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

/// Retrieve products
#[instrument(skip(service))]
pub async fn get_products(
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

/// Put a product
#[instrument(skip(service))]
pub async fn put_product(
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
