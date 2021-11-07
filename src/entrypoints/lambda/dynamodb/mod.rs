use crate::{domain, event_bus::EventBus, Event};
use lambda_runtime::Context;
use rayon::prelude::*;
use tracing::{info, instrument};

pub mod model;

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

/// Parse events from DynamoDB Streams
#[instrument(skip(event_bus, event))]
pub async fn parse_events(
    event_bus: &dyn EventBus<E = Event>,
    event: model::DynamoDBEvent,
    _: Context,
) -> Result<(), E> {
    info!("Transform events");
    let events = event
        .records
        .par_iter()
        .map(|record| record.try_into())
        .collect::<Result<Vec<_>, _>>()?;

    info!("Dispatching {} events", events.len());
    domain::send_events(event_bus, &events).await?;
    info!("Done dispatching events");

    Ok(())
}
