use crate::{Event, Service};
use lambda_runtime::Context;
use rayon::prelude::*;
use tracing::{info, instrument};

mod ext;
pub mod model;

use ext::EventExt;

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

#[instrument(skip(service))]
pub async fn parse_events(
    service: &Service,
    event: model::DynamoDBEvent,
    _: Context,
) -> Result<(), E> {
    info!("Transform events");
    let events = event
        .records
        .par_iter()
        .map(|record| Event::from_dynamodb_record(record))
        .collect::<Result<Vec<_>, _>>()?;

    info!("Dispatching {} events", events.len());
    service.send_events(&events).await?;
    info!("Done dispatching events");

    Ok(())
}
