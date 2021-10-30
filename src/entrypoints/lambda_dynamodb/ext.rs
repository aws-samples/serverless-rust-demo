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
