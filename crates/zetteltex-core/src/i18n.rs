use std::sync::OnceLock;

/// Idioma activo de la aplicacion. El CLI lo fija al arranque a partir de la
/// configuracion `[general] lang`; sin configuracion se usa ingles por defecto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Es,
}

impl Lang {
    /// Parsea un valor de configuracion (p. ej. `es`, `en`, `spanish`).
    pub fn parse(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "en" | "english" => Lang::En,
            _ => Lang::Es,
        }
    }

    pub fn is_es(self) -> bool {
        self == Lang::Es
    }
}

static LANG: OnceLock<Lang> = OnceLock::new();

/// Fija el idioma global (una sola vez, al arrancar).
pub fn set_lang(lang: Lang) {
    let _ = LANG.set(lang);
}

pub fn lang() -> Lang {
    *LANG.get().unwrap_or(&Lang::En)
}

/// Devuelve el texto en el idioma activo. Usar para mensajes sin argumentos.
pub fn tr(es: &'static str, en: &'static str) -> &'static str {
    match lang() {
        Lang::Es => es,
        Lang::En => en,
    }
}

/// Devuelve una `String` con el mensaje formateado en el idioma activo.
///
/// Ambas cadenas deben ser literales con los mismos placeholders `{}`, y los
/// argumentos se pasan una unica vez (solo se evalua la rama elegida):
///
/// ```ignore
/// println!("{}", tr!("Plan: nota='{}'", "Plan: note='{}'", name));
/// ```
#[macro_export]
macro_rules! tr {
    ($es:literal, $en:literal $(, $arg:expr)*) => {
        match $crate::i18n::lang() {
            $crate::i18n::Lang::Es => format!($es $(, $arg)*),
            $crate::i18n::Lang::En => format!($en $(, $arg)*),
        }
    };
}
