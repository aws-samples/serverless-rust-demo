use products::{entrypoints::container::*, utils::*};

#[tokio::main]
async fn main() -> Result<(), rocket::Error> {
    let store = get_store().await;
    let config = Config::new(store);

    setup_tracing();

    rocket::build()
        .mount(
            "/",
            rocket::routes![get_products, get_product, put_product, delete_product],
        )
        .manage(config)
        .launch()
        .await
}
