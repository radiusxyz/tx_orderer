#[macro_export]
macro_rules! blocking_emit {
    ($event:expr) => {
        $crate::event::event_manager().blocking_send($event_type)
    };
}

#[macro_export]
macro_rules! emit {
    ($event:expr) => {
        $crate::event::event_manager().send($event).await
    };
}
