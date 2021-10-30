//! # DynamoDB store implementation
//!
//! Store implementation using the AWS SDK for DynamoDB.

use super::Store;
use crate::{Error, Product, ProductRange};
use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use std::str;
use tracing::{info, instrument};

mod ext;
use ext::{AttributeValuesExt, ProductsExt};

/// DynamoDB store implementation.
///
/// We have to pass a generic type parameter `C` for the underlying client,
/// restricted to something that implements the SmithyConnector trait so we can
/// use it with both the actual AWS SDK client and a mock implementation.
pub struct DynamoDBStore<C> {
    client: Client<C>,
    table_name: String,
}

impl<C> DynamoDBStore<C>
where
    C: aws_smithy_client::bounds::SmithyConnector,
{
    pub fn new(client: Client<C>, table_name: String) -> DynamoDBStore<C> {
        DynamoDBStore { client, table_name }
    }
}

#[async_trait]
impl<C> Store for DynamoDBStore<C>
where
    C: aws_smithy_client::bounds::SmithyConnector,
{
    /// Get all items
    #[instrument(skip(self))]
    async fn all(&self, next: Option<&str>) -> Result<ProductRange, Error> {
        // Scan DynamoDB table
        info!("Scanning DynamoDB table");
        let mut req = self.client.scan().table_name(&self.table_name);
        req = if let Some(next) = next {
            req.exclusive_start_key("id", AttributeValue::S(next.to_owned()))
        } else {
            req
        };
        let res = req.send().await?;

        // Build response
        let products = match res.items {
            Some(items) => items
                .into_iter()
                .map(Product::from_dynamodb)
                .collect::<Result<Vec<Product>, Error>>()?,
            None => Vec::default(),
        };
        let next = res.last_evaluated_key.map(|m| m.get_s("id").unwrap());
        Ok(ProductRange { products, next })
    }
    /// Get item
    #[instrument(skip(self))]
    async fn get(&self, id: &str) -> Result<Option<Product>, Error> {
        info!("Getting item with id '{}' from DynamoDB table", id);
        let res = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await?;

        Ok(match res.item {
            Some(item) => Some(Product::from_dynamodb(item)?),
            None => None,
        })
    }
    /// Create or update an item
    #[instrument(skip(self))]
    async fn put(&self, product: &Product) -> Result<(), Error> {
        info!("Putting item with id '{}' into DynamoDB table", product.id);
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(product.to_dynamodb()))
            .send()
            .await?;

        Ok(())
    }
    /// Delete item
    #[instrument(skip(self))]
    async fn delete(&self, id: &str) -> Result<(), Error> {
        info!("Deleting item with id '{}' from DynamoDB table", id);
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use aws_sdk_dynamodb::{Client, Config, Credentials, Region};
    use aws_smithy_client::test_connection::TestConnection;
    use aws_smithy_http::body::SdkBody;

    /// Config for mocking DynamoDB
    async fn get_mock_config() -> Config {
        let cfg = aws_config::from_env()
            .region(Region::new("eu-west-1"))
            .credentials_provider(Credentials::from_keys("accesskey", "privatekey", None))
            .load()
            .await;

        Config::new(&cfg)
    }

    fn get_request_builder() -> http::request::Builder {
        http::Request::builder()
            .header("content-type", "application/x-amz-json-1.0")
            .uri(http::uri::Uri::from_static(
                "https://dynamodb.eu-west-1.amazonaws.com/",
            ))
    }

    #[tokio::test]
    async fn test_all_empty() -> Result<(), Error> {
        // GIVEN a DynamoDBStore with no items
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "DynamoDB_20120810.Scan")
                .body(SdkBody::from(r#"{"TableName": "test"}"#))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(r#"{"Items": []}"#))
                .unwrap(),
        )]);
        let client = Client::from_conf_conn(get_mock_config().await, conn.clone());
        let store = DynamoDBStore::new(client, "test".to_string());

        // WHEN getting all items
        let res = store.all(None).await?;

        // THEN the response is empty
        assert_eq!(res.products.len(), 0);
        // AND the request matches the expected request
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_all() -> Result<(), Error> {
        // GIVEN a DynamoDBStore with one item
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "DynamoDB_20120810.Scan")
                .body(SdkBody::from(r#"{"TableName": "test"}"#)).unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(r#"{"Items": [{"id": {"S": "1"}, "name": {"S": "test1"}, "price": {"N": "1.0"}}]}"#))
                .unwrap(),
        )]);
        let client = Client::from_conf_conn(get_mock_config().await, conn.clone());
        let store = DynamoDBStore::new(client, "test".to_string());

        // WHEN getting all items
        let res = store.all(None).await?;

        // THEN the response has one item
        assert_eq!(res.products.len(), 1);
        // AND the item has the correct id
        assert_eq!(res.products[0].id, "1");
        // AND the item has the correct name
        assert_eq!(res.products[0].name, "test1");
        // AND the item has the correct price
        assert_eq!(res.products[0].price, 1.0);
        // AND the request matches the expected request
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_all_next() -> Result<(), Error> {
        // GIVEN a DynamoDBStore with a last evaluated key
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "DynamoDB_20120810.Scan")
                .body(SdkBody::from(r#"{"TableName": "test"}"#))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(
                    r#"{"Items": [], "LastEvaluatedKey": {"id": {"S": "1"}}}"#,
                ))
                .unwrap(),
        )]);
        let client = Client::from_conf_conn(get_mock_config().await, conn.clone());
        let store = DynamoDBStore::new(client, "test".to_string());

        // WHEN getting all items
        let res = store.all(None).await?;

        // THEN the response has a next key
        assert_eq!(res.next, Some("1".to_string()));
        // AND the request matches the expected request
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> Result<(), Error> {
        // GIVEN a DynamoDBStore
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "DynamoDB_20120810.DeleteItem")
                .body(SdkBody::from(
                    r#"{"TableName": "test", "Key": {"id": {"S": "1"}}}"#,
                ))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from("{}"))
                .unwrap(),
        )]);
        let client = Client::from_conf_conn(get_mock_config().await, conn.clone());
        let store = DynamoDBStore::new(client, "test".to_string());

        // WHEN deleting an item
        store.delete("1").await?;

        // THEN the request matches the expected request
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_get() -> Result<(), Error> {
        // GIVEN a DynamoDBStore with one item
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "DynamoDB_20120810.GetItem")
                .body(SdkBody::from(r#"{"TableName": "test", "Key": {"id": {"S": "1"}}}"#))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(r#"{"Item": {"id": {"S": "1"}, "name": {"S": "test1"}, "price": {"N": "1.0"}}}"#))
                .unwrap(),
        )]);
        let client = Client::from_conf_conn(get_mock_config().await, conn.clone());
        let store = DynamoDBStore::new(client, "test".to_string());

        // WHEN getting an item
        let res = store.get("1").await?;

        // THEN the response has the correct values
        if let Some(product) = res {
            assert_eq!(product.id, "1");
            assert_eq!(product.name, "test1");
            assert_eq!(product.price, 1.0);
        } else {
            panic!("Expected product to be Some");
        }
        // AND the request matches the expected request
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_put() -> Result<(), Error> {
        // GIVEN an empty DynamoDBStore and a product
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "DynamoDB_20120810.PutItem")
                .body(SdkBody::from(r#"{"TableName":"test","Item":{"id":{"S":"1"},"name":{"S":"test1"},"price":{"N":"1.5"}}}"#))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(r#"{"Attributes": {"id": {"S": "1"}, "name": {"S": "test1"}, "price": {"N": "1.5"}}}"#))
                .unwrap(),
        )]);
        let client = Client::from_conf_conn(get_mock_config().await, conn.clone());
        let store = DynamoDBStore::new(client, "test".to_string());
        let product = Product {
            id: "1".to_string(),
            name: "test1".to_string(),
            price: 1.5,
        };

        // WHEN putting an item
        store.put(&product).await?;

        // THEN the request matches the expected request
        conn.assert_requests_match(&vec![]);

        Ok(())
    }
}
