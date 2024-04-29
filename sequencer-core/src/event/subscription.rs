use std::{marker::PhantomData, sync::Arc};

use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};

use crate::event::Event;

pub struct Subscription {
    list: Arc<Mutex<Vec<Sender<Vec<u8>>>>>,
}

impl Clone for Subscription {
    fn clone(&self) -> Self {
        Self {
            list: self.list.clone(),
        }
    }
}

impl Subscription {
    pub fn new() -> Self {
        Self {
            list: Arc::new(Mutex::new(Vec::default())),
        }
    }

    pub fn blocking_send<E: Event>(&self, event: E) {
        let event_vec = event.to_vec();
        let list_lock = self.list.blocking_lock();
        for subscriber in list_lock.iter() {
            subscriber.blocking_send(event_vec.clone()).unwrap();
        }
    }

    pub async fn send<E: Event>(&self, event: E) {
        let event_vec = event.to_vec();
        let list_lock = self.list.lock().await;
        for subscriber in list_lock.iter() {
            subscriber.send(event_vec.clone()).await.unwrap();
        }
    }

    pub fn blocking_subscribe<E: Event>(&self) -> Listener<E> {
        let (sender, receiver) = channel::<Vec<u8>>(8);
        let mut list_lock = self.list.blocking_lock();
        list_lock.push(sender);
        receiver.into()
    }

    pub async fn subscribe<E: Event>(&self) -> Listener<E> {
        let (sender, receiver) = channel::<Vec<u8>>(8);
        let mut list_lock = self.list.lock().await;
        list_lock.push(sender);
        receiver.into()
    }
}

pub struct Listener<E: Event>(Receiver<Vec<u8>>, PhantomData<E>);

impl<E: Event> From<Receiver<Vec<u8>>> for Listener<E> {
    fn from(value: Receiver<Vec<u8>>) -> Self {
        Self(value, PhantomData)
    }
}

impl<E: Event> Listener<E> {
    pub fn blocking_listen(&mut self) -> Option<E> {
        match self.0.blocking_recv() {
            Some(event_vec) => Some(E::from_vec(event_vec)),
            None => None,
        }
    }

    pub async fn listen(&mut self) -> Option<E> {
        match self.0.recv().await {
            Some(event_vec) => Some(E::from_vec(event_vec)),
            None => None,
        }
    }
}
