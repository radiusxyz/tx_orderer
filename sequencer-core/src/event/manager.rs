use std::{collections::HashMap, fmt::Debug, sync::Arc};

use serde::{de::DeserializeOwned, ser::Serialize};
use tokio::sync::Mutex;

use crate::{
    error::Error,
    event::subscription::{Listener, Subscription},
};

pub struct EventManager {
    events: Arc<Mutex<HashMap<&'static str, Subscription>>>,
}

unsafe impl Send for EventManager {}

unsafe impl Sync for EventManager {}

impl Clone for EventManager {
    fn clone(&self) -> Self {
        Self {
            events: self.events.clone(),
        }
    }
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(HashMap::default())),
        }
    }

    // pub async fn register_event<E>(&self) -> Result<(), Error>
    // where
    //     E: Clone + Debug + DeserializeOwned + Serialize
    // {
    //     let mut events_lock = self.events.lock().await;
    //     events_lock.get() {}
    //     Ok(())
    // }

    // pub fn register_event<E: Event>(mut self) -> Self {
    //     self.events.insert(E::id(), Subscription::new());
    //     self
    // }

    // pub fn blocking_send<E: Event>(&self, event: E) {
    //     let subscription = self.events.get(E::id()).unwrap();
    //     subscription.blocking_send(event);
    // }

    // pub async fn send<E: Event>(&self, event: E) {
    //     let subscription = self.events.get(E::id()).unwrap();
    //     subscription.send(event).await;
    // }

    // pub fn blocking_subscribe<E: Event>(&self) -> Listener<E> {
    //     let subscription = self.events.get(E::id()).unwrap();
    //     subscription.blocking_subscribe::<E>()
    // }

    // pub async fn subscribe<E: Event>(&self) -> Listener<E> {
    //     let subscription = self.events.get(E::id()).unwrap();
    //     subscription.subscribe().await
    // }
}
