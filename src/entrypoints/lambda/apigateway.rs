use crate::{domain, store, Product};
use lambda_http::{http::StatusCode, IntoResponse, Request, RequestExt, Response};
use serde_json::json;
use tracing::{error, info, instrument, warn};

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

/// Delete a product
#[instrument(skip(store))]
pub async fn delete_product(
    store: &dyn store::StoreDelete,
    event: Request,
) -> Result<impl IntoResponse, E> {
    // Retrieve product ID from event
    //
    // If the event doesn't contain a product ID, we return a 400 Bad Request.
    let path_parameters = event.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            warn!("Missing 'id' parameter in path");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "Missing 'id' parameter in path" }).to_string(),
            ));
        }
    };

    // Delete product
    info!("Deleting product {}", id);
    let res = domain::delete_product(store, id).await;

    // Return response
    //
    // The service returns a Result based on the success of the operation. If
    // the operation was successful, the Result is Ok(()), otherwise it will
    // contain an Err with the reason.
    match res {
        Ok(_) => {
            info!("Product {} deleted", id);
            Ok(response(
                StatusCode::OK,
                json!({"message": "Product deleted"}).to_string(),
            ))
        }
        Err(err) => {
            // Log the error message
            error!("Error deleting the product {}: {}", id, err);
            Ok(response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": "Failed to delete product"}).to_string(),
            ))
        }
    }
}

/// Get a product
#[instrument(skip(store))]
pub async fn get_product(
    store: &dyn store::StoreGet,
    event: Request,
) -> Result<impl IntoResponse, E> {
    // Retrieve product ID from event.
    //
    // If the event doesn't contain a product ID, we return a 400 Bad Request.
    let path_parameters = event.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            warn!("Missing 'id' parameter in path");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "Missing 'id' parameter in path" }).to_string(),
            ));
        }
    };

    // Retrieve product
    info!("Fetching product {}", id);
    let product = domain::get_product(store, id).await;

    // Return response
    //
    // Since the service returns an `Option` within a `Result`, there are three
    // potential scenarios: the product exists, it doesn't exist, or there was
    // an error.
    Ok(match product {
        // Product exists
        Ok(Some(product)) => response(StatusCode::OK, json!(product).to_string()),
        // Product doesn't exist
        Ok(None) => {
            warn!("Product not found: {}", id);
            response(
                StatusCode::NOT_FOUND,
                json!({"message": "Product not found"}).to_string(),
            )
        }
        // Error
        Err(err) => {
            error!("Error fetching product: {}", err);
            response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": "Error fetching product"}).to_string(),
            )
        }
    })
}

/// Retrieve products
#[instrument(skip(store))]
pub async fn get_products(
    store: &dyn store::StoreGetAll,
    _event: Request,
) -> Result<impl IntoResponse, E> {
    // Retrieve products
    // TODO: Add pagination
    let res = domain::get_products(store, None).await;

    // Return response
    Ok(match res {
        // Return a list of products
        Ok(res) => response(StatusCode::OK, json!(res).to_string()),
        // Return an error
        Err(err) => {
            error!("Something went wrong: {:?}", err);
            response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "message": format!("Something went wrong: {:?}", err) }).to_string(),
            )
        }
    })
}

/// Put a product
#[instrument(skip(store))]
pub async fn put_product(
    store: &dyn store::StorePut,
    event: Request,
) -> Result<impl IntoResponse, E> {
    // Retrieve product ID from event.
    //
    // If the event doesn't contain a product ID, we return a 400 Bad Request.
    let path_parameters = event.path_parameters();
    let id = match path_parameters.first("id") {
        Some(id) => id,
        None => {
            warn!("Missing 'id' parameter in path");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "Missing 'id' parameter in path" }).to_string(),
            ));
        }
    };

    // Read product from request
    let product: Product = match event.payload() {
        Ok(Some(product)) => product,
        Ok(None) => {
            warn!("Missing product in request body");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({"message": "Missing product in request body"}).to_string(),
            ));
        }
        Err(err) => {
            warn!("Failed to parse product from request body: {}", err);
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({"message": "Failed to parse product from request body"}).to_string(),
            ));
        }
    };
    info!("Parsed product: {:?}", product);

    // Compare product ID with product ID in body
    if product.id != id {
        warn!(
            "Product ID in path ({}) does not match product ID in body ({})",
            id, product.id
        );
        return Ok(response(
            StatusCode::BAD_REQUEST,
            json!({"message": "Product ID in path does not match product ID in body"}).to_string(),
        ));
    }

    // Put product
    let res = domain::put_product(store, &product).await;

    // Return response
    //
    // If the put was successful, we return a 201 Created. Otherwise, we return
    // a 500 Internal Server Error.
    Ok(match res {
        // Product created
        Ok(_) => {
            info!("Created product {:?}", product.id);
            response(
                StatusCode::CREATED,
                json!({"message": "Product created"}).to_string(),
            )
        }
        // Error creating product
        Err(err) => {
            error!("Failed to create product {}: {}", product.id, err);
            response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": "Failed to create product"}).to_string(),
            )
        }
    })
}

/// HTTP Response with a JSON payload
fn response(status_code: StatusCode, body: String) -> Response<String> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}
