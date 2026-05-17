// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Event modules for different components.

pub mod auth;
pub mod core;
pub mod java;
pub mod launch;
pub mod loader;
pub mod modloader;
pub mod console;

pub use auth::AuthEvent;
pub use core::CoreEvent;
pub use java::JavaEvent;
pub use launch::LaunchEvent;
pub use loader::LoaderEvent;
pub use modloader::ModloaderEvent;
pub use console::{
    ConsoleOutputEvent, ConsoleStream, InstanceDeletedEvent, InstanceExitedEvent,
    InstanceLaunchedEvent, InstanceWindowAppearedEvent,
};
