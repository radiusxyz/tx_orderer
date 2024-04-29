use std::collections::HashMap;

use crate::event::{
    subscription::{Listener, Subscription},
    Event,
};

pub struct EventManager {
    events: HashMap<&'static str, Subscription>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: HashMap::default(),
        }
    }

    pub fn register_event<E: Event>(mut self) -> Self {
        self.events.insert(E::id(), Subscription::new());
        self
    }

    pub fn blocking_send<E: Event>(&self, event: E) {
        let subscription = self.events.get(E::id()).unwrap();
        subscription.blocking_send(event);
    }

    pub async fn send<E: Event>(&self, event: E) {
        let subscription = self.events.get(E::id()).unwrap();
        subscription.send(event).await;
    }

    pub fn blocking_subscribe<E: Event>(&self) -> Listener<E> {
        let subscription = self.events.get(E::id()).unwrap();
        subscription.blocking_subscribe::<E>()
    }

    pub async fn subscribe<E: Event>(&self) -> Listener<E> {
        let subscription = self.events.get(E::id()).unwrap();
        subscription.subscribe().await
    }
}
