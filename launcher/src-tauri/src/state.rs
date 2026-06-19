//! Estado global de la aplicacion.
//!
//! En Fase 2-3 contendra: cola de descargas (tokio), procesos de Minecraft
//! en ejecucion, cache de versiones/loaders, sesiones de cuentas, etc.

#[derive(Default)]
pub struct AppState {
    // Marcador de inicializacion; se ira llenando en fases siguientes.
    pub _initialized: std::sync::atomic::AtomicBool,
}
