// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Broadcast-based event system for LightyLauncher.

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

mod errors;
pub mod module;

pub use errors::{
    EventReceiveError, EventReceiveResult, EventSendError, EventSendResult,
    EventTryReceiveError, EventTryReceiveResult,
};
pub use module::{
    AuthEvent, ConsoleOutputEvent, ConsoleStream, CoreEvent, InstanceDeletedEvent,
    InstanceExitedEvent, InstanceLaunchedEvent, InstanceWindowAppearedEvent, JavaEvent,
    LaunchEvent, LoaderEvent, ModloaderEvent,
};

/// Event bus for broadcasting events to multiple listeners
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new EventBus with the specified buffer capacity.
    /// Slow receivers surface [`EventReceiveError::Lagged`] when overflow drops events.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to events emitted after this call.
    pub fn subscribe(&self) -> EventReceiver {
        EventReceiver {
            receiver: self.sender.subscribe(),
        }
    }

    /// Emit an event. Silently dropped if no active subscribers.
    pub fn emit(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}

/// Receiver for events from an EventBus
pub struct EventReceiver {
    receiver: broadcast::Receiver<Event>,
}

impl EventReceiver {
    /// Wait for the next event (async).
    pub async fn next(&mut self) -> EventReceiveResult<Event> {
        self.receiver.recv().await.map_err(Into::into)
    }

    /// Try to receive an event without blocking.
    pub fn try_next(&mut self) -> EventTryReceiveResult<Event> {
        self.receiver.try_recv().map_err(Into::into)
    }
}

/// Root event enum containing all event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    Auth(AuthEvent),
    Java(JavaEvent),
    Launch(LaunchEvent),
    Loader(LoaderEvent),
    Modloader(ModloaderEvent),
    Core(CoreEvent),
    InstanceLaunched(InstanceLaunchedEvent),
    InstanceWindowAppeared(InstanceWindowAppearedEvent),
    InstanceExited(InstanceExitedEvent),
    ConsoleOutput(ConsoleOutputEvent),
    InstanceDeleted(InstanceDeletedEvent),
}

use once_cell::sync::Lazy;

/// Global event bus instance used by the library to emit events.
pub static EVENT_BUS: Lazy<EventBus> = Lazy::new(|| EventBus::new(1000));
