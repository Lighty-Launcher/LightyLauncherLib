//! Real-time stdout / stderr capture from the spawned game process.

use lighty_launcher::prelude::*;

pub fn log(e: ConsoleOutputEvent) {
    let prefix = match e.stream {
        ConsoleStream::Stdout => "[GAME]",
        ConsoleStream::Stderr => "[ERR ]",
    };
    print!("{} {}", prefix, e.line);
}
