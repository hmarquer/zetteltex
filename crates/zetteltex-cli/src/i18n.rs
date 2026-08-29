//! Re-export de la infraestructura i18n de zetteltex-core para uso dentro del
//! CLI. En cada modulo basta con:
//!
//! ```ignore
//! use crate::i18n::{lang, set_lang, tr, Lang};
//! use crate::i18n::tr;
//! ```
//!
//! `tr("es", "en")` devuelve el texto en el idioma activo; `tr!(...)` formatea
//! con argumentos.

pub use zetteltex_core::i18n::tr;
pub use zetteltex_core::i18n::{set_lang, Lang};
pub use zetteltex_core::tr;
