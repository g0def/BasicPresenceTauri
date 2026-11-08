//! Cœur métier de BasicPresence, indépendant de toute UI.
//!
//! Clean Architecture : `domain` (entités, contrats) ← `application` (use
//! cases, DTO) ← `infrastructure` (libSQL, crypto, export…). Le point de
//! composition [`services::build_services`] assemble les implémentations ;
//! [`services::open_services`] ouvre le stockage d'un répertoire de données.
//! Le binaire desktop (Slint) appelle directement les use cases de
//! [`services::Services`] — il n'y a plus de frontière IPC.

pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod services;

#[cfg(test)]
mod integration_tests;
