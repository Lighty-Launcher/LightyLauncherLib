// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Error types for the event system.

use thiserror::Error;
use tokio::sync::broadcast::error::{RecvError,TryRecvError};

/// Errors that can occur when receiving events
#[derive(Debug, Error, Clone)]
pub enum EventReceiveError {
    #[error("Event bus has been dropped - all senders closed")]
    BusDropped,

    #[error("Receiver lagged behind by {skipped} events - some events were missed")]
    Lagged { skipped: u64 },
}

/// Errors that can occur when trying to receive events (non-blocking)
#[derive(Debug, Error, Clone)]
pub enum EventTryReceiveError {
    #[error("No events available in the channel")]
    Empty,

    #[error("Event bus has been dropped - all senders closed")]
    BusDropped,

    #[error("Receiver lagged behind by {skipped} events - some events were missed")]
    Lagged { skipped: u64 },
}

/// Errors that can occur when sending events
#[derive(Debug, Error)]
pub enum EventSendError {
    #[error("No active receivers - event was not sent")]
    NoReceivers,
}

impl From<RecvError> for EventReceiveError {
    fn from(err: RecvError) -> Self {
        match err {
            RecvError::Closed => EventReceiveError::BusDropped,
            RecvError::Lagged(skipped) => {
                EventReceiveError::Lagged { skipped }
            }
        }
    }
}

impl From<TryRecvError> for EventTryReceiveError {
    fn from(err: TryRecvError) -> Self {
        match err {
            TryRecvError::Empty => EventTryReceiveError::Empty,
            TryRecvError::Closed => {
                EventTryReceiveError::BusDropped
            }
            TryRecvError::Lagged(skipped) => {
                EventTryReceiveError::Lagged { skipped }
            }
        }
    }
}

/// Type alias for event receive operations
pub type EventReceiveResult<T> = Result<T, EventReceiveError>;

/// Type alias for event try receive operations
pub type EventTryReceiveResult<T> = Result<T, EventTryReceiveError>;

/// Type alias for event send operations
pub type EventSendResult<T> = Result<T, EventSendError>;
