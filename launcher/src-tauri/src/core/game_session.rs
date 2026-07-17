//! Estado global de partida activa: bloquea sync PvP y actualizaciones pesadas mientras MC corre.

use std::collections::HashSet;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static RUNNING: AtomicBool = AtomicBool::new(false);
static PENDING_PVP_SYNC: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn set_running(running: bool) {
    RUNNING.store(running, Ordering::SeqCst);
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

pub fn queue_pvp_sync(instance_id: impl Into<String>) {
    if let Ok(mut pending) = PENDING_PVP_SYNC.lock() {
        pending.insert(instance_id.into());
    }
}

pub fn take_pending_pvp_sync() -> Vec<String> {
    PENDING_PVP_SYNC
        .lock()
        .map(|mut pending| pending.drain().collect())
        .unwrap_or_default()
}
