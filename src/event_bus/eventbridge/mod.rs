//! EventBridge bus implementation
//!
//! Bus implementation using the AWS SDK for EventBridge.

use super::EventBus;
use crate::{Error, Event};
use async_trait::async_trait;
use aws_sdk_eventbridge::Client;
use futures::future::join_all;
use tracing::{info, instrument};

mod ext;
use ext::EventExt;

/// EventBridge bus implementation.
pub struct EventBridgeBus {
    client: Client,
    bus_name: String,
}

impl EventBridgeBus {
    pub fn new(client: Client, bus_name: String) -> Self {
        Self { client, bus_name }
    }
}

#[async_trait]
impl EventBus for EventBridgeBus {
    type E = Event;

    /// Publish an event to the event bus.
    #[instrument(skip(self))]
    async fn send_event(&self, event: &Self::E) -> Result<(), Error> {
        info!("Publishing event to EventBridge");
        self.client
            .put_events()
            .entries(event.to_eventbridge(&self.bus_name))
            .send()
            .await?;

        Ok(())
    }

    /// Publish a batch of events to the event bus.
    #[instrument(skip(self, events))]
    async fn send_events(&self, events: &[Self::E]) -> Result<(), Error> {
        // Send batches of 10 events at a time
        //
        // EventBridge has a limit of 10 events per `put_events()` request.
        //
        // `send()` returns a Future, so we can use `join_all` to wait for all of the
        // futures to complete. This means we can send all batches at the same time
        // and not have to wait for each batch to complete before sending the next one.
        info!("Publishing events to EventBridge");
        let res = join_all(events.iter().collect::<Vec<_>>().chunks(10).map(|chunk| {
            self.client
                .put_events()
                .set_entries(Some(
                    chunk
                        .iter()
                        .map(|e| e.to_eventbridge(&self.bus_name))
                        .collect::<Vec<_>>(),
                ))
                .send()
        }))
        .await;

        // Retrieve errors from the response vector
        //
        // If any of the responses contained an error, we'll return an error.
        res.into_iter().collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Product};
    use aws_sdk_eventbridge::{Client, Config, Credentials, Region};
    use aws_smithy_client::{erase::DynConnector, test_connection::TestConnection};
    use aws_smithy_http::body::SdkBody;

    // Config for mocking EventBridge
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
            .header("content-type", "application/x-amz-json-1.1")
            .uri(http::uri::Uri::from_static(
                "https://events.eu-west-1.amazonaws.com/",
            ))
    }

    #[tokio::test]
    async fn test_send_event() -> Result<(), Error> {
        // GIVEN a mock EventBridge client
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "AWSEvents.PutEvents")
                .body(SdkBody::from(
                    r#"{"Entries":[{"Source":"rust-products","Resources":["test-id"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id\",\"name\":\"test-name\",\"price\":10.0}}","EventBusName":"test-bus"}]}"#,
                ))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from("{}"))
                .unwrap(),
        )]);
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
        let event_bus = EventBridgeBus::new(client, "test-bus".to_string());

        // WHEN we send an event
        let event = Event::Created {
            product: Product {
                id: "test-id".to_string(),
                name: "test-name".to_string(),
                price: 10.0,
            },
        };
        event_bus.send_event(&event).await?;

        // THEN the request should have been sent to EventBridge
        assert_eq!(conn.requests().len(), 1);
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_send_events() -> Result<(), Error> {
        // GIVEN a mock EventBridge client
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "AWSEvents.PutEvents")
                .body(SdkBody::from(
                    r#"{"Entries":[{"Source":"rust-products","Resources":["test-id"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id\",\"name\":\"test-name\",\"price\":10.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-2"],"DetailType":"ProductDeleted","Detail":"{\"type\":\"Deleted\",\"product\":{\"id\":\"test-id-2\",\"name\":\"test-name-2\",\"price\":20.0}}","EventBusName":"test-bus"}]}"#,
                ))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from("{}"))
                .unwrap(),
        )]);
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
        let event_bus = EventBridgeBus::new(client, "test-bus".to_string());

        // WHEN we send a batch of events
        let events = vec![
            Event::Created {
                product: Product {
                    id: "test-id".to_string(),
                    name: "test-name".to_string(),
                    price: 10.0,
                },
            },
            Event::Deleted {
                product: Product {
                    id: "test-id-2".to_string(),
                    name: "test-name-2".to_string(),
                    price: 20.0,
                },
            },
        ];
        event_bus.send_events(&events).await?;

        // THEN the request should have been sent to EventBridge
        assert_eq!(conn.requests().len(), 1);
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_send_events0() -> Result<(), Error> {
        // GIVEN a mock EventBridge client
        let conn: TestConnection<SdkBody> = TestConnection::new(vec![]);
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
        let event_bus = EventBridgeBus::new(client, "test-bus".to_string());

        // WHEN we send zero events
        event_bus.send_events(&vec![]).await?;

        // THEN no request should have been sent to EventBridge
        assert_eq!(conn.requests().len(), 0);
        conn.assert_requests_match(&vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_send_events15() -> Result<(), Error> {
        // GIVEN a mock EventBridge client
        let conn = TestConnection::new(vec![(
            get_request_builder()
                .header("x-amz-target", "AWSEvents.PutEvents")
                .body(SdkBody::from(
                    r#"{"Entries":[{"Source":"rust-products","Resources":["test-id-0"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-0\",\"name\":\"test-name-0\",\"price\":10.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-1"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-1\",\"name\":\"test-name-1\",\"price\":11.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-2"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-2\",\"name\":\"test-name-2\",\"price\":12.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-3"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-3\",\"name\":\"test-name-3\",\"price\":13.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-4"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-4\",\"name\":\"test-name-4\",\"price\":14.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-5"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-5\",\"name\":\"test-name-5\",\"price\":15.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-6"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-6\",\"name\":\"test-name-6\",\"price\":16.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-7"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-7\",\"name\":\"test-name-7\",\"price\":17.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-8"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-8\",\"name\":\"test-name-8\",\"price\":18.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-9"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-9\",\"name\":\"test-name-9\",\"price\":19.0}}","EventBusName":"test-bus"}]}"#,
                ))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from("{}"))
                .unwrap(),
        ), (
            get_request_builder()
                .header("x-amz-target", "AWSEvents.PutEvents")
                .body(SdkBody::from(
                    r#"{"Entries":[{"Source":"rust-products","Resources":["test-id-10"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-10\",\"name\":\"test-name-10\",\"price\":20.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-11"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-11\",\"name\":\"test-name-11\",\"price\":21.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-12"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-12\",\"name\":\"test-name-12\",\"price\":22.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-13"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-13\",\"name\":\"test-name-13\",\"price\":23.0}}","EventBusName":"test-bus"},{"Source":"rust-products","Resources":["test-id-14"],"DetailType":"ProductCreated","Detail":"{\"type\":\"Created\",\"product\":{\"id\":\"test-id-14\",\"name\":\"test-name-14\",\"price\":24.0}}","EventBusName":"test-bus"}]}"#,
                ))
                .unwrap(),
            http::Response::builder()
                .status(200)
                .body(SdkBody::from("{}"))
                .unwrap(),
        )]);
        let client =
            Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
        let event_bus = EventBridgeBus::new(client, "test-bus".to_string());

        // WHEN we send 15 events
        let events = (0..15)
            .map(|i| Event::Created {
                product: Product {
                    id: format!("test-id-{}", i),
                    name: format!("test-name-{}", i),
                    price: 10.0 + i as f64,
                },
            })
            .collect::<Vec<_>>();
        event_bus.send_events(&events).await?;

        // THEN two requests should have been sent to EventBridge
        assert_eq!(conn.requests().len(), 2);
        conn.assert_requests_match(&vec![]);

        Ok(())
    }
}
