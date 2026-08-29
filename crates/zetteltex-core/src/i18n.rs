use std::sync::atomic::{AtomicU8, Ordering};

/// Idioma activo de la aplicacion. El CLI lo fija al arranque a partir de la
/// configuracion `[general] lang`; sin configuracion (o con cualquier valor
/// distinto de `es`) se usa ingles por defecto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Es,
}

impl Lang {
    /// Parsea un valor de configuracion (p. ej. `es`). El idioma por defecto es
    /// ingles: cualquier valor distinto de `es`/`spanish` cae a `En`.
    pub fn parse(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "es" | "spanish" => Lang::Es,
            _ => Lang::En,
        }
    }

    pub fn is_es(self) -> bool {
        self == Lang::Es
    }
}

const EN: u8 = 0;
const ES: u8 = 1;

static LANG: AtomicU8 = AtomicU8::new(EN);

/// Fija el idioma activo. Puede invocarse en cualquier momento (p. ej. dentro
/// de la configuracion interactiva, para que los mensajes siguientes cambien
/// de idioma al instante). Es seguro bajo concurrencia: cada lectura usa el
/// valor atomico.
pub fn set_lang(lang: Lang) {
    LANG.store(
        match lang {
            Lang::Es => ES,
            Lang::En => EN,
        },
        Ordering::Release,
    );
}

pub fn lang() -> Lang {
    match LANG.load(Ordering::Acquire) {
        ES => Lang::Es,
        _ => Lang::En,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_to_english() {
        assert_eq!(Lang::parse(""), Lang::En);
        assert_eq!(Lang::parse("garbage"), Lang::En);
        assert_eq!(Lang::parse("FR"), Lang::En);
        assert_eq!(Lang::parse("en"), Lang::En);
        assert_eq!(Lang::parse("ENGLISH"), Lang::En);
    }

    #[test]
    fn parse_only_explicit_es_selects_spanish() {
        assert_eq!(Lang::parse("es"), Lang::Es);
        assert_eq!(Lang::parse("ES"), Lang::Es);
        assert_eq!(Lang::parse("Spanish"), Lang::Es);
    }

    #[test]
    fn set_lang_can_switch_mid_session() {
        set_lang(Lang::En);
        assert_eq!(lang(), Lang::En);
        assert_eq!(tr("es", "en"), "en");

        set_lang(Lang::Es);
        assert_eq!(lang(), Lang::Es);
        assert_eq!(tr("es", "en"), "es");
    }
}
