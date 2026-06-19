//! Logica de dominio del launcher, separada por modulo.
//!
//! Fase 1: solo `hardware` esta implementado. El resto son esqueletos con la
//! responsabilidad documentada, listos para las fases siguientes.

pub mod hardware;

// Fase 2
pub mod accounts;
pub mod instances;
pub mod java;

// Fase 3
pub mod downloads;
pub mod loaders;

// Fase 4
pub mod ai;
pub mod diagnostics;
pub mod servers;
pub mod updater;
