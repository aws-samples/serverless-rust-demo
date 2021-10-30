use super::EventBus;
use crate::{Error, Event};
use async_trait::async_trait;
use aws_sdk_eventbridge::Client;
use futures::future::join_all;
use tracing::{info, instrument};

mod ext;
use ext::EventExt;

pub struct EventBridgeBus<C> {
    client: Client<C>,
    bus_name: String,
}

impl<C> EventBridgeBus<C>
where
    C: aws_smithy_client::bounds::SmithyConnector,
{
    pub fn new(client: Client<C>, bus_name: String) -> Self {
        Self { client, bus_name }
    }
}

#[async_trait]
impl<C> EventBus for EventBridgeBus<C>
where
    C: aws_smithy_client::bounds::SmithyConnector,
{
    /// Publish an event to the event bus.
    #[instrument(skip(self))]
    async fn send_event(&self, event: &Event) -> Result<(), Error> {
        info!("Publishing event to EventBridge");
        self.client
            .put_events()
            .entries(event.to_eventbridge(&self.bus_name))
            .send()
            .await?;

        Ok(())
    }

    /// Publish a batch of events to the event bus.
    #[instrument(skip(self))]
    async fn send_events(&self, events: &[&Event]) -> Result<(), Error> {
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
