use chrono::prelude::*;
use std::collections::BTreeMap;
use tracing_subscriber::{prelude::*, Layer};

/// Setup tracing
pub fn setup_tracing() {
    let layer = LambdaLayer::new(tracing::Level::INFO);
    tracing_subscriber::registry().with(layer).init();
}

struct LambdaVisitor<'a> {
    pub data: &'a mut BTreeMap<String, serde_json::Value>,
}

impl<'a> tracing::field::Visit for LambdaVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.data
            .insert(field.name().to_string(), format!("{:?}", value).into());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.data.insert(field.name().to_string(), value.into());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.data.insert(field.name().to_string(), value.into());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.data.insert(field.name().to_string(), value.into());
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.data.insert(field.name().to_string(), value.into());
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.data
            .insert(field.name().to_string(), format!("{:?}", value).into());
    }
}

struct LambdaLayer {
    level: tracing::Level,
}

impl LambdaLayer {
    pub fn new(level: tracing::Level) -> Self {
        Self { level }
    }
}

impl<S> Layer<S> for LambdaLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();

        if metadata.level() > &self.level {
            return;
        }

        let mut data = BTreeMap::new();
        let mut visitor = LambdaVisitor { data: &mut data };
        event.record(&mut visitor);
        
        let output = serde_json::json!({
            "level": metadata.level().to_string(),
            "location": format!("{}:{}", metadata.file().unwrap_or("UNKNOWN"), metadata.line().unwrap_or(0)),
            "target": metadata.target(),
            "message": data,
            "timestamp": Utc::now().to_rfc3339(),
        });
        println!("{}", serde_json::to_string(&output).unwrap());
    }
}
