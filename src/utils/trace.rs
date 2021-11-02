use chrono::prelude::*;
use lambda_runtime::Context;
use std::collections::BTreeMap;
use tracing_subscriber::{prelude::*, Layer};
use tracing::{span::Attributes, Id};

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

struct LambdaContextVisitor {
    pub context: Option<Context>,
}

impl tracing::field::Visit for LambdaContextVisitor {
    fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "lambda_context" {
            let context = serde_json::from_str(value).ok();
            self.context = context;
        }
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
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = LambdaContextVisitor { context: None };
        attrs.record(&mut visitor);
        if let Some(context) = visitor.context {
            println!("INJECTING CONTEXT: {:?}", context);
            let span = ctx.span(id).unwrap();
            let mut extensions = span.extensions_mut();
            extensions.insert(context);
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();
        if metadata.level() > &self.level {
            return;
        }

        // Find Lambda context
        let lambda_ctxs = if let Some(scope) = ctx.event_scope(event) {
            scope
            .from_root()
            .map(|span| {
                if let Some(v) = span.extensions().get::<Context>() {
                    println!("FOUND CONTEXT: {:?}", v);
                    Some(v.clone())
                } else {
                    None
                }
            })
            .filter(Option::is_some)
            .collect::<Vec<_>>()
        } else {
            println!("EVENT_SCOPE NOT FOUND");
            vec![]
        };
        let lambda_ctx = lambda_ctxs.first().unwrap();

        let mut data = BTreeMap::new();
        let mut visitor = LambdaVisitor { data: &mut data };
        event.record(&mut visitor);

        let output = if let Some(lambda_ctx) = lambda_ctx {
            // Lambda context found
            //
            // Adding keys based on the Lambda context
            serde_json::json!({
                "level": metadata.level().to_string(),
                "location": format!("{}:{}", metadata.file().unwrap_or("UNKNOWN"), metadata.line().unwrap_or(0)),
                "target": metadata.target(),
                // If data has only one key named 'message', we can just use that as the message.
                // This is the default key when using macros such as `info!()` or `debug!()`.
                "message": if data.len() == 1 && data.contains_key("message") {
                    data.remove("message").unwrap().into()
                } else {
                    serde_json::to_value(data).unwrap()
                },
                "timestamp": Utc::now().to_rfc3339(),

                // Lambda context keys
                "function_name": lambda_ctx.env_config.function_name,
                "function_memory_size": lambda_ctx.env_config.memory,
                "function_arn": lambda_ctx.invoked_function_arn,
                "function_request_id": lambda_ctx.request_id,
                "xray_trace_id": lambda_ctx.xray_trace_id,
            })
        } else {
            // No Lambda context found
            serde_json::json!({
                "level": metadata.level().to_string(),
                "location": format!("{}:{}", metadata.file().unwrap_or("UNKNOWN"), metadata.line().unwrap_or(0)),
                "target": metadata.target(),
                // If data has only one key named 'message', we can just use that as the message.
                // This is the default key when using macros such as `info!()` or `debug!()`.
                "message": if data.len() == 1 && data.contains_key("message") {
                    data.remove("message").unwrap().into()
                } else {
                    serde_json::to_value(data).unwrap()
                },
                "timestamp": Utc::now().to_rfc3339(),
            })
        };
        println!("{}", serde_json::to_string(&output).unwrap());
    }
}
