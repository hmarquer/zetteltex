use std::path::{Path, PathBuf};

use thiserror::Error;

pub mod i18n;

pub type Result<T> = std::result::Result<T, ZettelError>;

#[derive(Debug, Error)]
pub enum ZettelError {
    #[error("{0}")]
    MissingDirectory(String),
    #[error("{0}")]
    InvalidArgument(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub notes_slipbox: PathBuf,
    pub projects: PathBuf,
    pub template: PathBuf,
}

impl WorkspacePaths {
    pub fn discover(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let paths = Self {
            notes_slipbox: root.join("notes/slipbox"),
            projects: root.join("projects"),
            template: root.join("template"),
            root,
        };
        paths.validate()?;
        Ok(paths)
    }

    pub fn validate(&self) -> Result<()> {
        let mut missing = Vec::new();

        if !is_existing_dir(&self.notes_slipbox) {
            missing.push(self.notes_slipbox.display().to_string());
        }
        if !is_existing_dir(&self.projects) {
            missing.push(self.projects.display().to_string());
        }
        if !is_existing_dir(&self.template) {
            missing.push(self.template.display().to_string());
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(ZettelError::MissingDirectory(crate::tr!(
                "directorio de trabajo no encontrado: {}. Revisa --workspace-root y la estructura minima (notes/slipbox, projects, template)",
                "working directory not found: {}. Check --workspace-root and the minimal structure (notes/slipbox, projects, template)",
                missing.join(", ")
            )))
        }
    }
}

fn is_existing_dir(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

/// Valida que `name` sea un único componente de ruta seguro para usar como
/// nombre de nota/proyecto. Rechaza vacíos, `.`/`..`, separadores (`/`, `\\`)
/// y rutas absolutas, cerrando la clase de vulnerabilidad de path traversal.
pub fn validate_component_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\'])
        || Path::new(name).is_absolute()
    {
        return Err(ZettelError::InvalidArgument(format!(
            "invalid name '{name}': must be a single path component"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_and_separators() {
        assert!(validate_component_name("../../evil").is_err());
        assert!(validate_component_name("a/b").is_err());
        assert!(validate_component_name("a\\b").is_err());
        assert!(validate_component_name("..").is_err());
        assert!(validate_component_name(".").is_err());
        assert!(validate_component_name("").is_err());
        assert!(validate_component_name("/abs").is_err());
    }

    #[test]
    fn accepts_simple_names() {
        assert!(validate_component_name("my-note").is_ok());
        assert!(validate_component_name("nota_1").is_ok());
    }
}
