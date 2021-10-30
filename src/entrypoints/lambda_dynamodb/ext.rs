//! # Extension traits for DynamoDB entrypoint
//!
//! This module contains extension traits for Product and Event, to transform
//! incoming events from the Lambda function into the types used internally.
//!
//! We cannot reuse the types from the `aws_sdk_dynamodb` crate because they don't
//! implement `serde::Serialize` and `serde::Deserialize`.

use super::model::{AttributeValue, DynamoDBRecord};
use crate::{Error, Event, Product};
use std::collections::HashMap;

pub trait ProductExt {
    type S;
    fn from_dynamodb(item: &HashMap<String, AttributeValue>) -> Result<Self::S, Error>;
}

impl ProductExt for Product {
    type S = Self;

    fn from_dynamodb(item: &HashMap<String, AttributeValue>) -> Result<Self::S, Error> {
        Ok(Product {
            id: item
                .get("id")
                .ok_or(Error::InternalError("id is missing"))?
                .as_s()
                .ok_or(Error::InternalError("id is missing"))?
                .to_string(),
            name: item
                .get("name")
                .ok_or(Error::InternalError("name is missing"))?
                .as_s()
                .ok_or(Error::InternalError("name is missing"))?
                .to_string(),
            price: item
                .get("price")
                .ok_or(Error::InternalError("price is missing"))?
                .as_n()
                .ok_or(Error::InternalError("price is missing"))?,
        })
    }
}

pub trait EventExt {
    type S;
    fn from_dynamodb_record(record: &DynamoDBRecord) -> Result<Self::S, Error>;
}

impl EventExt for Event {
    type S = Self;

    fn from_dynamodb_record(record: &DynamoDBRecord) -> Result<Self::S, Error> {
        match record.event_name.as_str() {
            "INSERT" => {
                let product = Product::from_dynamodb(&record.dynamodb.new_image)?;
                Ok(Event::Created { product })
            }
            "MODIFY" => {
                let old = Product::from_dynamodb(&record.dynamodb.old_image)?;
                let new = Product::from_dynamodb(&record.dynamodb.new_image)?;
                Ok(Event::Updated { old, new })
            }
            "REMOVE" => {
                let product = Product::from_dynamodb(&record.dynamodb.old_image)?;
                Ok(Event::Deleted { product })
            }
            _ => Err(Error::InternalError("Unknown event type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::model;
    use super::*;

    #[test]
    fn test_product_from_dynamodb() {
        let mut value = HashMap::new();
        value.insert("id".to_string(), AttributeValue::S("1".to_string()));
        value.insert(
            "name".to_string(),
            AttributeValue::S("Product 1".to_string()),
        );
        value.insert("price".to_string(), AttributeValue::N("1.0".to_string()));

        let product = Product::from_dynamodb(&value).unwrap();
        assert_eq!(product.id, "1");
        assert_eq!(product.name, "Product 1");
        assert_eq!(product.price, 1.0);
    }

    #[test]
    fn test_event_from_dynamodb_insert() {
        let mut record = DynamoDBRecord {
            event_id: "event_id".to_string(),
            event_name: "INSERT".to_string(),
            event_source: "aws:dynamodb".to_string(),
            event_version: "1".to_string(),
            aws_region: "us-east-1".to_string(),
            dynamodb: model::DynamoDBStreamRecord {
                approximate_creation_date_time: Some(64.0),
                keys: HashMap::new(),
                new_image: HashMap::new(),
                old_image: HashMap::new(),
                sequence_number: "sequence_number".to_string(),
                size_bytes: 64.0,
                stream_view_type: "stream_view_type".to_string(),
            },
            event_source_arn: "arn:aws:dynamodb:us-east-1:123456789012:table/Products/stream/2020-01-01T00:00:00.000".to_owned(),
        };
        record
            .dynamodb
            .new_image
            .insert("id".to_string(), AttributeValue::S("1".to_string()));
        record.dynamodb.new_image.insert(
            "name".to_string(),
            AttributeValue::S("Product 1".to_string()),
        );
        record
            .dynamodb
            .new_image
            .insert("price".to_string(), AttributeValue::N("1.0".to_string()));

        let event = Event::from_dynamodb_record(&record).unwrap();
        match event {
            Event::Created { product } => {
                assert_eq!(product.id, "1");
                assert_eq!(product.name, "Product 1");
                assert_eq!(product.price, 1.0);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[test]
    fn test_event_from_dynamodb_remove() {
        let mut record = DynamoDBRecord {
            event_id: "event_id".to_string(),
            event_name: "REMOVE".to_string(),
            event_source: "aws:dynamodb".to_string(),
            event_version: "1".to_string(),
            aws_region: "us-east-1".to_string(),
            dynamodb: model::DynamoDBStreamRecord {
                approximate_creation_date_time: Some(64.0),
                keys: HashMap::new(),
                new_image: HashMap::new(),
                old_image: HashMap::new(),
                sequence_number: "sequence_number".to_string(),
                size_bytes: 64.0,
                stream_view_type: "stream_view_type".to_string(),
            },
            event_source_arn: "arn:aws:dynamodb:us-east-1:123456789012:table/Products/stream/2020-01-01T00:00:00.000".to_owned(),
        };
        record
            .dynamodb
            .old_image
            .insert("id".to_string(), AttributeValue::S("1".to_string()));
        record.dynamodb.old_image.insert(
            "name".to_string(),
            AttributeValue::S("Product 1".to_string()),
        );
        record
            .dynamodb
            .old_image
            .insert("price".to_string(), AttributeValue::N("1.0".to_string()));

        let event = Event::from_dynamodb_record(&record).unwrap();
        match event {
            Event::Deleted { product } => {
                assert_eq!(product.id, "1");
                assert_eq!(product.name, "Product 1");
                assert_eq!(product.price, 1.0);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[test]
    fn test_event_from_dynamodb_modify() {
        let mut record = DynamoDBRecord {
            event_id: "event_id".to_string(),
            event_name: "MODIFY".to_string(),
            event_source: "aws:dynamodb".to_string(),
            event_version: "1".to_string(),
            aws_region: "us-east-1".to_string(),
            dynamodb: model::DynamoDBStreamRecord {
                approximate_creation_date_time: Some(64.0),
                keys: HashMap::new(),
                new_image: HashMap::new(),
                old_image: HashMap::new(),
                sequence_number: "sequence_number".to_string(),
                size_bytes: 64.0,
                stream_view_type: "stream_view_type".to_string(),
            },
            event_source_arn: "arn:aws:dynamodb:us-east-1:123456789012:table/Products/stream/2020-01-01T00:00:00.000".to_owned(),
        };
        record
            .dynamodb
            .old_image
            .insert("id".to_string(), AttributeValue::S("1".to_string()));
        record.dynamodb.old_image.insert(
            "name".to_string(),
            AttributeValue::S("Product 1".to_string()),
        );
        record
            .dynamodb
            .old_image
            .insert("price".to_string(), AttributeValue::N("1.0".to_string()));
        record
            .dynamodb
            .new_image
            .insert("id".to_string(), AttributeValue::S("2".to_string()));
        record.dynamodb.new_image.insert(
            "name".to_string(),
            AttributeValue::S("Product 2".to_string()),
        );
        record
            .dynamodb
            .new_image
            .insert("price".to_string(), AttributeValue::N("2.0".to_string()));

        let event = Event::from_dynamodb_record(&record).unwrap();
        match event {
            Event::Updated { old, new } => {
                assert_eq!(old.id, "1");
                assert_eq!(old.name, "Product 1");
                assert_eq!(old.price, 1.0);
                assert_eq!(new.id, "2");
                assert_eq!(new.name, "Product 2");
                assert_eq!(new.price, 2.0);
            }
            _ => panic!("Unexpected event type"),
        }
    }
}
