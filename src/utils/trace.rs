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

pub fn inject_lambda_context(ctx: &Context) -> tracing::Span {
    let ctx_string = serde_json::to_string(ctx).unwrap();
    let ctx_str = ctx_string.as_str();
    tracing::span!(tracing::Level::TRACE, "lambda_handler", lambda_context = ctx_str)
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
                    Some(v.clone())
                } else {
                    None
                }
            })
            .filter_map(|c| c)
            .collect::<Vec<_>>()
        } else {
            Default::default()
        };
        let lambda_ctx = lambda_ctxs.first();

        let mut data = BTreeMap::new();
        let mut visitor = LambdaVisitor { data: &mut data };
        event.record(&mut visitor);

        let output = serde_json::json!({
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
        });

        let output = if let Some(lambda_ctx) = lambda_ctx {
            if let serde_json::Value::Object(mut output) = output {
                output.insert("function_name".to_string(), lambda_ctx.env_config.function_name.clone().into());
                output.insert("function_memory_size".to_string(), lambda_ctx.env_config.memory.into());
                output.insert("function_arn".to_string(), lambda_ctx.invoked_function_arn.clone().into());
                output.insert("function_request_id".to_string(), lambda_ctx.request_id.clone().into());
                output.insert("xray_trace_id".to_string(), lambda_ctx.xray_trace_id.clone().into());

                serde_json::Value::Object(output)
            } else {
                output
            }
        } else {
            output
        };

        println!("{}", serde_json::to_string(&output).unwrap());
    }
}
