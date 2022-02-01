//! # DynamoDB store implementation
//!
//! Store implementation using the AWS SDK for DynamoDB.

use super::{Store, StoreDelete, StoreGet, StoreGetAll, StorePut};
use crate::{Error, Product, ProductRange};
use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use std::collections::HashMap;
use tracing::{info, instrument};

mod ext;
use ext::AttributeValuesExt;

/// DynamoDB store implementation.
pub struct DynamoDBStore {
    client: Client,
    table_name: String,
}

impl DynamoDBStore {
    pub fn new(client: Client, table_name: String) -> DynamoDBStore {
        DynamoDBStore { client, table_name }
    }
}

impl Store for DynamoDBStore {}

#[async_trait]
impl StoreGetAll for DynamoDBStore {
    /// Get all items
    #[instrument(skip(self))]
    async fn all(&self, next: Option<&str>) -> Result<ProductRange, Error> {
        // Scan DynamoDB table
        info!("Scanning DynamoDB table");
        let mut req = self.client.scan().table_name(&self.table_name).limit(20);
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
                .map(|v| v.try_into())
                .collect::<Result<Vec<Product>, Error>>()?,
            None => Vec::default(),
        };
        let next = res.last_evaluated_key.map(|m| m.get_s("id").unwrap());
        Ok(ProductRange { products, next })
    }
}

#[async_trait]
impl StoreGet for DynamoDBStore {
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
            Some(item) => Some(item.try_into()?),
            None => None,
        })
    }
}

#[async_trait]
impl StorePut for DynamoDBStore {
    /// Create or update an item
    #[instrument(skip(self))]
    async fn put(&self, product: &Product) -> Result<(), Error> {
        info!("Putting item with id '{}' into DynamoDB table", product.id);
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(product.into()))
            .send()
            .await?;

        Ok(())
    }
}

#[async_trait]
impl StoreDelete for DynamoDBStore {
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

impl From<&Product> for HashMap<String, AttributeValue> {
    /// Convert a &Product into a DynamoDB item
    fn from(value: &Product) -> HashMap<String, AttributeValue> {
        let mut retval = HashMap::new();
        retval.insert("id".to_owned(), AttributeValue::S(value.id.clone()));
        retval.insert("name".to_owned(), AttributeValue::S(value.name.clone()));
        retval.insert(
            "price".to_owned(),
            AttributeValue::N(format!("{:}", value.price)),
        );

        retval
    }
}
impl TryFrom<HashMap<String, AttributeValue>> for Product {
    type Error = Error;

    /// Try to convert a DynamoDB item into a Product
    ///
    /// This could fail as the DynamoDB item might be missing some fields.
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(Product {
            id: value
                .get_s("id")
                .ok_or(Error::InternalError("Missing id"))?,
            name: value
                .get_s("name")
                .ok_or(Error::InternalError("Missing name"))?,
            price: value
                .get_n("price")
                .ok_or(Error::InternalError("Missing price"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use aws_sdk_dynamodb::{Client, Config, Credentials, Region};
    use aws_smithy_client::{erase::DynConnector, test_connection::TestConnection};
    use aws_smithy_http::body::SdkBody;

    /// Config for mocking DynamoDB
    async fn get_mock_config() -> Config {
        let cfg = aws_config::from_env()
            .region(Region::new("eu-west-1"))
            .credentials_provider(Credentials::new(
                "accesskey",
                "privatekey",
                None,
                None,
                "dummy",
            ))
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
                .body(SdkBody::from(r#"{"TableName":"test","Limit":20}"#))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(r#"{"Items": []}"#))
                .unwrap(),
        )]);
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
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
                .body(SdkBody::from(r#"{"TableName":"test","Limit":20}"#)).unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(r#"{"Items": [{"id": {"S": "1"}, "name": {"S": "test1"}, "price": {"N": "1.0"}}]}"#))
                .unwrap(),
        )]);
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
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
                .body(SdkBody::from(r#"{"TableName":"test","Limit":20}"#))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from(
                    r#"{"Items": [], "LastEvaluatedKey": {"id": {"S": "1"}}}"#,
                ))
                .unwrap(),
        )]);
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
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
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
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
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
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
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
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

    #[test]
    fn product_from_dynamodb() {
        let mut value = HashMap::new();
        value.insert("id".to_owned(), AttributeValue::S("id".to_owned()));
        value.insert("name".to_owned(), AttributeValue::S("name".to_owned()));
        value.insert("price".to_owned(), AttributeValue::N("1.0".to_owned()));

        let product = Product::try_from(value).unwrap();
        assert_eq!(product.id, "id");
        assert_eq!(product.name, "name");
        assert_eq!(product.price, 1.0);
    }

    #[test]
    fn product_to_dynamodb() -> Result<(), Error> {
        let product = Product {
            id: "id".to_owned(),
            name: "name".to_owned(),
            price: 1.5,
        };

        let value: HashMap<String, AttributeValue> = (&product).into();
        assert_eq!(value.get("id").unwrap().as_s().unwrap(), "id");
        assert_eq!(value.get("name").unwrap().as_s().unwrap(), "name");
        assert_eq!(value.get("price").unwrap().as_n().unwrap(), "1.5");

        Ok(())
    }
}
