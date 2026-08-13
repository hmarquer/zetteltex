use std::path::{Path, PathBuf};

use thiserror::Error;

pub mod i18n;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
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
            Err(AppError::MissingDirectory(crate::tr!(
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
