// Copyright 2025 The Drasi Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Process-wide link registry connecting a state machine reaction to its
//! companion source.
//!
//! The reaction and the source are independent components (created separately,
//! even via separate FFI descriptor calls), but they live in the same address
//! space — the host process for static linking, or the single loaded plugin
//! `.so` for dynamic linking. This registry lets them rendezvous by `source_id`:
//!
//! * The **source** publishes a fresh sender every time it starts and clears it
//!   on stop. It owns the receiving end of the channel and dispatches incoming
//!   [`SourceChange`]s to its subscribers.
//! * The **reaction** looks up the current sender by `source_id` each time it has
//!   a state transition to emit. Looking it up per push (rather than caching)
//!   keeps the link correct across source restarts.
//!
//! Persistence is the safety net: the reaction always writes the transition to
//! the state store before pushing, so even if no sender is registered (source not
//! yet started), the change is reflected the next time a query bootstraps from
//! the source.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use drasi_core::models::SourceChange;
use tokio::sync::mpsc;

/// Default capacity of the reaction→source channel.
pub const CHANNEL_CAPACITY: usize = 1024;

type SenderMap = HashMap<String, mpsc::Sender<SourceChange>>;

static SENDERS: LazyLock<Mutex<SenderMap>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Create a fresh channel for `source_id`, publish its sender, and return the
/// receiver for the source's dispatch loop. Any previously published sender for
/// the same id is replaced (e.g. on source restart).
pub fn register_source(source_id: &str) -> mpsc::Receiver<SourceChange> {
    let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
    SENDERS
        .lock()
        .expect("state-machine sender registry poisoned")
        .insert(source_id.to_string(), tx);
    rx
}

/// Remove the published sender for `source_id` (called when the source stops).
pub fn unregister_source(source_id: &str) {
    SENDERS
        .lock()
        .expect("state-machine sender registry poisoned")
        .remove(source_id);
}

/// Get the current sender for `source_id`, if a source is registered.
pub fn current_sender(source_id: &str) -> Option<mpsc::Sender<SourceChange>> {
    SENDERS
        .lock()
        .expect("state-machine sender registry poisoned")
        .get(source_id)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_then_send_and_receive() {
        let id = "test-source-reg-1";
        let mut rx = register_source(id);
        let tx = current_sender(id).expect("sender should be registered");

        let metadata = drasi_core::models::ElementMetadata {
            reference: drasi_core::models::ElementReference::new(id, "e1"),
            labels: std::sync::Arc::from(vec![std::sync::Arc::from("X")]),
            effective_from: 0,
        };
        tx.send(SourceChange::Delete { metadata }).await.unwrap();

        let got = rx.recv().await;
        assert!(matches!(got, Some(SourceChange::Delete { .. })));

        unregister_source(id);
        assert!(current_sender(id).is_none());
    }

    #[tokio::test]
    async fn restart_replaces_sender() {
        let id = "test-source-reg-2";
        let _rx1 = register_source(id);
        let _rx2 = register_source(id); // simulates a restart
                                        // The latest sender is the one returned.
        assert!(current_sender(id).is_some());
        unregister_source(id);
    }
}
