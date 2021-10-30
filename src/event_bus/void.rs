use super::EventBus;
use crate::{Error, Event};
use async_trait::async_trait;

#[derive(Default)]
pub struct VoidBus;

impl VoidBus {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventBus for VoidBus {
    type E = Event;

    async fn send_event(&self, _: &Self::E) -> Result<(), Error> {
        Err(Error::InternalError("send_event is not supported"))
    }

    async fn send_events(&self, _: &[Self::E]) -> Result<(), Error> {
        Err(Error::InternalError("send_events is not supported"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Product;

    #[tokio::test]
    async fn test_send_event() {
        let bus = VoidBus;
        let event = Event::Created {
            product: Product {
                id: "123".to_string(),
                name: "test".to_string(),
                price: 10.0,
            },
        };
        let result = bus.send_event(&event).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_events() {
        let bus = VoidBus;
        let event = Event::Created {
            product: Product {
                id: "123".to_string(),
                name: "test".to_string(),
                price: 10.0,
            },
        };
        let result = bus.send_events(&[event]).await;
        assert!(result.is_err());
    }
}
