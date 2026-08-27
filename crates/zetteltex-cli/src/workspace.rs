use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Result};
use zetteltex_core::WorkspacePaths;

use crate::i18n::tr;

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

    println!(
        "{} '{}'",
        tr("Workspace inicializado correctamente en", "Workspace initialized successfully at"),
        root
    );
    println!("{}", tr!("Directorios creados e inicializados: notes/slipbox, projects, template", "Directories created and initialized: notes/slipbox, projects, template"));
    Ok(())
}

pub fn read_template_file_or_suggest_init(paths: &WorkspacePaths, name: &str) -> Result<String> {
    let p = paths.template.join(name);
    if !p.exists() {
        eprintln!(
            "{} '{}' {} '{}'. {} `zetteltex --workspace-root {} init` {}.",
            tr("Plantilla", "Template"),
            name,
            tr("no encontrada en", "not found in"),
            paths.template.display(),
            tr("Ejecuta", "Run"),
            paths.root.display(),
            tr("para crear las plantillas", "to create the templates")
        );
        bail!("{}: {}", tr("Missing template", "Missing template"), p.display());
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
    eprintln!(
        "{} '{}'. {} `zetteltex --workspace-root {} init` {}.",
        tr("No se encontraron archivos de plantilla en", "No template files found in"),
        paths.template.display(),
        tr("Ejecuta", "Run"),
        paths.root.display(),
        tr("para crear las plantillas necesarias", "to create the required templates")
    );
    bail!("{}: {}", tr("Missing template directory", "Missing template directory"), paths.template.display());
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
        print!(
            "{} (y/N): ",
            tr!("El archivo {} ya existe. ¿Deseas sobrescribirlo?", "The file {} already exists. Do you want to overwrite it?", config_path.display())
        );
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("{}", tr("Operación cancelada.", "Operation canceled."));
            return Ok(std::process::ExitCode::SUCCESS);
        }
    }

    println!(
        "\n=== {} ===",
        tr("Configuración interactiva de ZettelTeX", "Interactive ZettelTeX configuration")
    );
    println!("{}", tr!("Pulsa Enter para mantener los valores por defecto.\n", "Press Enter to keep the default values.\n"));

    let lang = prompt_user(
        tr("Idioma de la interfaz (es|en)", "Interface language (es|en)"),
        "en",
    )?;
    let editor = prompt_user(
        tr("Editor preferido (code, vim, nvim, o ruta personalizada)", "Preferred editor (code, vim, nvim, or custom path)"),
        "code",
    )?;
    let pdf_output_dir = prompt_user(tr("Directorio de salida para PDFs compilados", "Output directory for compiled PDFs"), "pdf")?;
    let html_output_dir = prompt_user(tr("Directorio de salida para HTML compilado", "Output directory for compiled HTML"), "html")?;
    let obsidian_vault = prompt_user(tr("Ruta a tu vault de Obsidian (deja vacío si no usas)", "Path to your Obsidian vault (leave empty if unused)"), "vault")?;
    let notes_subdir = prompt_user(tr("Subdirectorio de notas en la vault", "Notes subdirectory in the vault"), "notes")?;
    let projects_subdir = prompt_user(tr("Subdirectorio de proyectos en la vault", "Projects subdirectory in the vault"), "projects")?;
    let max_results = prompt_user(tr("Número máximo de resultados en búsquedas fuzzy", "Maximum number of results in fuzzy searches"), "20")?;
    let history_results = prompt_user(tr("Número de resultados de historial al abrir fuzzy sin query", "Number of history results when opening fuzzy without a query"), "20")?;
    let selection_color = prompt_user(tr("Color de selección en búsquedas (ej. magenta, blue, green, red)", "Selection color in searches (e.g. magenta, blue, green, red)"), "magenta")?;

    let config_content = format!(
        r#"# Configuración de ZettelTeX
# Este archivo ha sido auto-generado por `zetteltex init_config`
# ZettelTeX configuration / auto-generated by `zetteltex init_config`

[general]
# Idioma de la interfaz: es o en / Interface language: es or en
lang = "{}"
# Editor para el comando edit / Editor for the edit command
editor = "{}"

[render]
# Directorio donde se guardarán los archivos PDF compilados
# Directory where compiled PDF files are stored
pdf_output_dir = "{}"
# Directorio donde se guardarán los archivos HTML compilados
# Directory where compiled HTML files are stored
html_output_dir = "{}"

[export]
# Ruta a la vault de Obsidian (opcional) / Path to your Obsidian vault (optional)
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
"#,
        lang, editor, pdf_output_dir, html_output_dir, obsidian_vault, notes_subdir, projects_subdir, max_results, history_results, selection_color
    );

    std::fs::write(&config_path, config_content)?;
    println!(
        "\n{} {}!",
        tr("¡Archivo de configuración guardado exitosamente en", "Configuration file saved successfully to"),
        config_path.display()
    );

    Ok(std::process::ExitCode::SUCCESS)
}
