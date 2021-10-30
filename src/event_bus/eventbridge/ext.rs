use crate::Event;
use aws_sdk_eventbridge::model::PutEventsRequestEntry;

static SOURCE: &str = "rust-products";

pub trait EventExt {
    fn to_eventbridge(&self, bus_name: &str) -> PutEventsRequestEntry;
}

impl EventExt for Event {
    fn to_eventbridge(&self, bus_name: &str) -> PutEventsRequestEntry {
        PutEventsRequestEntry::builder()
            .event_bus_name(bus_name)
            .source(SOURCE)
            .detail_type(match self {
                Event::Created { .. } => "ProductCreated",
                Event::Updated { .. } => "ProductUpdated",
                Event::Deleted { .. } => "ProductDeleted",
            })
            .resources(self.id())
            .detail(serde_json::to_string(self).unwrap())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Product;

    #[test]
    fn test_to_eventbridge() {
        let event = Event::Created {
            product: Product {
                id: "123".to_string(),
                name: "test".to_string(),
                price: 10.0,
            },
        };
        let entry = event.to_eventbridge("test-bus");
        assert_eq!(entry.event_bus_name.unwrap(), "test-bus");
        assert_eq!(entry.source.unwrap(), SOURCE);
        assert_eq!(entry.detail_type.unwrap(), "ProductCreated");
        assert_eq!(entry.resources.unwrap(), vec!["123".to_string()]);
        assert_eq!(
            entry.detail.unwrap(),
            serde_json::to_string(&event).unwrap()
        );
    }
}
