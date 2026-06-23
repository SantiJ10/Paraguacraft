//! Persistencia JSON atomica y duradera.
//!
//! Escribe a `<archivo>.tmp` y hace `rename` (atomico en NTFS y POSIX): o queda
//! el archivo nuevo completo, o el viejo intacto. Evita corrupcion si el proceso
//! muere durante el flush. Espeja la logica de `Api._guardar` del Python.

pub mod keys;

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AppResult;

/// Lee y deserializa un JSON. Devuelve `None` si no existe o esta corrupto.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Escribe `value` como JSON de forma atomica y durable.
pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(value)?;
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&data)?;
        f.flush()?;
        // fsync best-effort: algunos FS (red/virtuales) no lo soportan.
        let _ = f.sync_all();
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Igual que `write_json_atomic`, restringiendo permisos a solo-usuario en Unix.
/// Para archivos sensibles (tokens).
pub fn write_secret_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    write_json_atomic(path, value)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}
