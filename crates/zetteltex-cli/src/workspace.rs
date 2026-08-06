use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Result};
use zetteltex_core::WorkspacePaths;

const TEMPLATE_NOTE: &str = include_str!("../../../template/note.tex");
const TEMPLATE_PROJECT: &str = include_str!("../../../template/project.tex");
const TEMPLATE_STYLE: &str = include_str!("../../../template/style.sty");
const TEMPLATE_TEXBOOK: &str = include_str!("../../../template/texbook.cls");
const TEMPLATE_TEXNOTE: &str = include_str!("../../../template/texnote.cls");

pub fn init_workspace(root: &str) -> Result<()> {
    let root_path = Path::new(root);
    fs::create_dir_all(root_path.join("notes/slipbox"))?;
    fs::create_dir_all(root_path.join("projects"))?;
    let workspace_template = root_path.join("template");
    fs::create_dir_all(&workspace_template)?;

    let docs_path = root_path.join("notes/documents.tex");
    if !docs_path.exists() {
        fs::write(&docs_path, "% zetteltex: documents main index\n")?;
    }

    let template_files = [
        ("note.tex", TEMPLATE_NOTE),
        ("project.tex", TEMPLATE_PROJECT),
        ("style.sty", TEMPLATE_STYLE),
        ("texbook.cls", TEMPLATE_TEXBOOK),
        ("texnote.cls", TEMPLATE_TEXNOTE),
    ];

    for (name, content) in template_files {
        let dst = workspace_template.join(name);
        if !dst.exists() {
            fs::write(dst, content)?;
        }
    }

    println!("Workspace inicializado correctamente en '{}'", root);
    println!("Directorios creados e inicializados: notes/slipbox, projects, template");
    Ok(())
}

pub fn read_template_file_or_suggest_init(paths: &WorkspacePaths, name: &str) -> Result<String> {
    let p = paths.template.join(name);
    if !p.exists() {
        eprintln!("Plantilla '{}' no encontrada en '{}'. Ejecuta `zetteltex --workspace-root {} init` para crear las plantillas.", name, paths.template.display(), paths.root.display());
        bail!("Missing template: {}", p.display());
    }
    let s = fs::read_to_string(&p)?;
    Ok(s)
}

pub fn ensure_template_available_or_suggest_init(paths: &WorkspacePaths) -> Result<()> {
    // Check for at least one of the core template files that TeX needs.
    let candidates = ["texnote.cls", "texbook.cls", "style.sty", "note.tex"];
    for c in candidates {
        if paths.template.join(c).exists() {
            return Ok(());
        }
    }
    eprintln!("No se encontraron archivos de plantilla en '{}'. Ejecuta `zetteltex --workspace-root {} init` para crear las plantillas necesarias.", paths.template.display(), paths.root.display());
    bail!("Missing template directory: {}", paths.template.display());
}

fn prompt_user(prompt: &str, default: &str) -> anyhow::Result<String> {
    print!("{} [{}]: ", prompt, default);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

pub fn init_config_interactive(paths: &WorkspacePaths) -> anyhow::Result<std::process::ExitCode> {
    let config_path = paths.root.join("zetteltex.toml");

    if config_path.exists() {
        print!("El archivo {} ya existe. ¿Deseas sobrescribirlo? (y/N): ", config_path.display());
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Operación cancelada.");
            return Ok(std::process::ExitCode::SUCCESS);
        }
    }

    println!("\n=== Configuración interactiva de ZettelTeX ===");
    println!("Pulsa Enter para mantener los valores por defecto.\n");

    let pdf_output_dir = prompt_user("Directorio de salida para PDFs compilados", "pdf")?;
    let obsidian_vault = prompt_user("Ruta a tu vault de Obsidian (deja vacío si no usas)", "")?;
    let notes_subdir = prompt_user("Subdirectorio de notas en la vault", "")?;
    let projects_subdir = prompt_user("Subdirectorio de proyectos en la vault", "")?;
    let max_results = prompt_user("Número máximo de resultados en búsquedas fuzzy", "30")?;
    let history_results = prompt_user("Número de resultados de historial al abrir fuzzy sin query", "10")?;
    let selection_color = prompt_user("Color de selección en búsquedas (ej. magenta, blue, green, red)", "magenta")?;

    let config_content = format!(r#"# Configuración de ZettelTeX
# Este archivo ha sido auto-generado por `zetteltex init_config`

[render]
# Directorio donde se guardarán los archivos PDF compilados
pdf_output_dir = "{}"

[export]
# Ruta a la vault de Obsidian (opcional)
obsidian_vault = "{}"
# Subdirectorio para las notas dentro de obsidian_vault
notes_subdir = "{}"
# Subdirectorio para los proyectos dentro de obsidian_vault
projects_subdir = "{}"

[fuzzy]
# Número máximo de resultados a mostrar en búsquedas
max_results = {}
# Número de resultados a mostrar cuando la búsqueda está vacía (historial + populares)
history_results = {}
# Color de acento de la interfaz (en ANSI, por ejemplo 'blue', 'green', 'magenta')
selection_color = "{}"
"#, pdf_output_dir, obsidian_vault, notes_subdir, projects_subdir, max_results, history_results, selection_color);

    std::fs::write(&config_path, config_content)?;
    println!("\n¡Archivo de configuración guardado exitosamente en {}!", config_path.display());

    Ok(std::process::ExitCode::SUCCESS)
}
