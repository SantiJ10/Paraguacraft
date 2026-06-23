//! Cuentas offline (No-Premium) con UUID determinista, identico al de los
//! servidores vanilla en modo offline. Espeja `_offline_uuid` de `core.py`.

use md5::{Digest, Md5};
use uuid::Uuid;

/// UUID v3 estilo `OfflinePlayer:<name>` (mismo algoritmo que Minecraft).
pub fn offline_uuid(username: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{username}").as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest);
    bytes[6] = (bytes[6] & 0x0f) | 0x30; // version 3
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variante RFC 4122
    Uuid::from_bytes(bytes).to_string()
}
