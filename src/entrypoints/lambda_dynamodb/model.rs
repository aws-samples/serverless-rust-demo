//! # DynamoDB Event models
//!
//! Models for the DynamoDB event entrypoint.
//!
//! We cannot use the models provided by the AWS SDK for Rust, as they do not
//! implement the `serde::Serialize` and `serde::Deserialize` traits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct DynamoDBEvent {
    #[serde(rename = "Records")]
    pub records: Vec<DynamoDBRecord>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DynamoDBRecord {
    #[serde(rename = "awsRegion")]
    pub aws_region: String,

    #[serde(rename = "dynamodb")]
    pub dynamodb: DynamoDBStreamRecord,

    #[serde(rename = "eventID")]
    pub event_id: String,

    #[serde(rename = "eventName")]
    pub event_name: String,

    #[serde(rename = "eventSource")]
    pub event_source: String,

    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: String,

    #[serde(rename = "eventVersion")]
    pub event_version: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DynamoDBStreamRecord {
    #[serde(rename = "ApproximateCreationDateTime", default)]
    pub approximate_creation_date_time: Option<f64>,

    #[serde(rename = "Keys", default)]
    pub keys: HashMap<String, AttributeValue>,

    #[serde(rename = "NewImage", default)]
    pub new_image: HashMap<String, AttributeValue>,

    #[serde(rename = "OldImage", default)]
    pub old_image: HashMap<String, AttributeValue>,

    #[serde(rename = "SequenceNumber")]
    pub sequence_number: String,

    #[serde(rename = "SizeBytes")]
    pub size_bytes: f64,

    #[serde(rename = "StreamViewType")]
    pub stream_view_type: String,
}

/// Attribute Value
///
/// This is a copy of the `AttributeValue` struct from the AWS SDK for Rust,
/// but without blob and `is_`-prefixed methods.
/// See https://docs.rs/aws-sdk-dynamodb/0.0.22-alpha/aws_sdk_dynamodb/model/enum.AttributeValue.html
#[derive(Deserialize, Serialize, Debug)]
pub enum AttributeValue {
    // B(Blob),
    Bool(bool),
    // Bs(Vec<Blob>),
    L(Vec<AttributeValue>),
    M(HashMap<String, AttributeValue>),
    N(String),
    Ns(Vec<String>),
    Null(bool),
    S(String),
    Ss(Vec<String>),
}

impl AttributeValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_l(&self) -> Option<&Vec<AttributeValue>> {
        match self {
            AttributeValue::L(l) => Some(l),
            _ => None,
        }
    }
    pub fn as_m(&self) -> Option<&HashMap<String, AttributeValue>> {
        match self {
            AttributeValue::M(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_n(&self) -> Option<f64> {
        match self {
            AttributeValue::N(n) => n.parse::<f64>().ok(),
            _ => None,
        }
    }
    pub fn as_ns(&self) -> Vec<f64> {
        match self {
            AttributeValue::Ns(ns) => 
                ns.iter()
                    .filter_map(|n| n.parse::<f64>().ok())
                    .collect(),
            _ => Default::default(),
        }
    }
    pub fn as_null(&self) -> Option<bool> {
        match self {
            AttributeValue::Null(null) => Some(*null),
            _ => None,
        }
    }
    pub fn as_s(&self) -> Option<&str> {
        match self {
            AttributeValue::S(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_ss(&self) -> Vec<String> {
        match self {
            AttributeValue::Ss(ss) => ss.to_owned(),
            _ => Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        // Example from https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html
        let data = r#"
        {
            "Records": [
              {
                "eventID": "1",
                "eventVersion": "1.0",
                "dynamodb": {
                  "Keys": {
                    "Id": {
                      "N": "101"
                    }
                  },
                  "NewImage": {
                    "Message": {
                      "S": "New item!"
                    },
                    "Id": {
                      "N": "101"
                    }
                  },
                  "StreamViewType": "NEW_AND_OLD_IMAGES",
                  "SequenceNumber": "111",
                  "SizeBytes": 26
                },
                "awsRegion": "us-west-2",
                "eventName": "INSERT",
                "eventSourceARN": "someARN",
                "eventSource": "aws:dynamodb"
              },
              {
                "eventID": "2",
                "eventVersion": "1.0",
                "dynamodb": {
                  "OldImage": {
                    "Message": {
                      "S": "New item!"
                    },
                    "Id": {
                      "N": "101"
                    }
                  },
                  "SequenceNumber": "222",
                  "Keys": {
                    "Id": {
                      "N": "101"
                    }
                  },
                  "SizeBytes": 59,
                  "NewImage": {
                    "Message": {
                      "S": "This item has changed"
                    },
                    "Id": {
                      "N": "101"
                    }
                  },
                  "StreamViewType": "NEW_AND_OLD_IMAGES"
                },
                "awsRegion": "us-west-2",
                "eventName": "MODIFY",
                "eventSourceARN": "someARN",
                "eventSource": "aws:dynamodb"
            }]
        }"#;

        let event: DynamoDBEvent = serde_json::from_str(data).unwrap();

        assert_eq!(event.records.len(), 2);
        assert_eq!(event.records[0].event_name, "INSERT");
        assert_eq!(
            event.records[0]
                .dynamodb
                .new_image
                .get("Message")
                .unwrap()
                .as_s(),
            Some("New item!")
        );
        assert_eq!(event.records[1].event_name, "MODIFY");
        assert_eq!(
            event.records[1]
                .dynamodb
                .old_image
                .get("Message")
                .unwrap()
                .as_s(),
            Some("New item!")
        );
    }
}
