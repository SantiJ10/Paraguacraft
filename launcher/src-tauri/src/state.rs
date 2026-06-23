//! Estado global de la aplicacion.
//!
//! Eficiencia (Regla 3): el `reqwest::Client` NO esta siempre vivo. Se crea
//! perezosamente al primer uso y se **destruye en idle** (`shutdown_network`),
//! liberando el pool de conexiones, sockets y la RAM asociada. Un contador de
//! operaciones activas evita cortar descargas en curso. No hay hilos ni timers
//! propios: todo corre sobre el runtime tokio de Tauri y vuelve a 0% CPU.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::core::accounts::microsoft::DeviceAuth;
use crate::models::JavaInstallation;

pub struct AppState {
    /// Cliente HTTP reutilizable, creado bajo demanda y liberado en idle.
    http: Mutex<Option<reqwest::Client>>,
    /// Operaciones de red en curso (para no destruir el cliente a mitad de descarga).
    net_active: AtomicUsize,
    /// Estado transitorio del device-code flow de Microsoft.
    pub ms_device: Mutex<Option<DeviceAuth>>,
    /// Cache perezosa de Javas detectados.
    pub java_cache: Mutex<Option<Vec<JavaInstallation>>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            http: Mutex::new(None),
            net_active: AtomicUsize::new(0),
            ms_device: Mutex::new(None),
            java_cache: Mutex::new(None),
        }
    }
}

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("ParaguacraftLauncher/2.0 (+https://paraguacraft.gg)")
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .pool_max_idle_per_host(24)
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

impl AppState {
    /// Devuelve el cliente HTTP, creandolo si hace falta. Clonar es barato
    /// (Arc interno) y comparte el mismo pool de conexiones.
    pub fn client(&self) -> reqwest::Client {
        let mut guard = self.http.lock().unwrap();
        if let Some(c) = guard.as_ref() {
            return c.clone();
        }
        let c = build_client();
        *guard = Some(c.clone());
        c
    }

    /// Marca el inicio de una operacion de red. Devuelve el cliente.
    pub fn net_begin(&self) -> reqwest::Client {
        self.net_active.fetch_add(1, Ordering::SeqCst);
        self.client()
    }

    /// Marca el fin de una operacion de red; si no quedan activas, libera el cliente.
    pub fn net_end(&self) {
        if self.net_active.fetch_sub(1, Ordering::SeqCst) <= 1 {
            self.shutdown_network();
        }
    }

    /// Libera el cliente HTTP y su pool de conexiones si no hay operaciones activas.
    /// Idempotente; seguro de llamar tras lanzar el juego.
    pub fn shutdown_network(&self) {
        if self.net_active.load(Ordering::SeqCst) == 0 {
            *self.http.lock().unwrap() = None;
        }
    }

    /// Devuelve (cliente, guard). Al soltar el guard se decrementa el contador y,
    /// si no quedan operaciones, se libera el cliente. Patron RAII a prueba de `?`.
    pub fn net_scope(&self) -> (reqwest::Client, NetGuard<'_>) {
        let client = self.net_begin();
        (client, NetGuard { state: self })
    }
}

/// Guard RAII: garantiza el teardown de red aunque la operacion falle con `?`.
pub struct NetGuard<'a> {
    state: &'a AppState,
}

impl Drop for NetGuard<'_> {
    fn drop(&mut self) {
        self.state.net_end();
    }
}
