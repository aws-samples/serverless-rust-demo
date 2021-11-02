use crate::{Event, Service, utils::inject_lambda_context};
use lambda_runtime::Context;
use rayon::prelude::*;
use tracing::{info, instrument};

mod ext;
pub mod model;

use ext::EventExt;

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

/// Parse events from DynamoDB Streams
#[instrument(skip(service, event))]
pub async fn parse_events(
    service: &Service,
    event: model::DynamoDBEvent,
    ctx: Context,
) -> Result<(), E> {
    let span = inject_lambda_context(&ctx);
    let _guard = span.enter();

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
