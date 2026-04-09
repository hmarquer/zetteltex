use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::OnceLock;
use std::sync::{mpsc, Arc, Mutex};
use std::{collections::{HashMap, HashSet}, fs};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use crossterm::cursor::Show;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size as terminal_size, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use regex::Regex;
use serde::{Deserialize, Serialize};
use strsim::normalized_levenshtein;
use zetteltex_core::WorkspacePaths;
use zetteltex_db::init_database;
use zetteltex_parser::{parse_note, parse_project_inclusions};
use tracing::{error, warn};

const DEFAULT_RECENT_FILES: usize = 10;
const DEFAULT_RENAME_RECENT_INDEX: usize = 1;
const DEFAULT_RENDER_WORKERS: usize = 4;

const TEMPLATE_NOTE: &str = include_str!("../../../template/note.tex");
const TEMPLATE_PROJECT: &str = include_str!("../../../template/project.tex");
const TEMPLATE_STYLE: &str = include_str!("../../../template/style.sty");
const TEMPLATE_TEXBOOK: &str = include_str!("../../../template/texbook.cls");
const TEMPLATE_TEXNOTE: &str = include_str!("../../../template/texnote.cls");

#[derive(Debug, Parser)]
#[command(name = "zetteltex")]
#[command(about = "CLI Rust para gestionar ZettelTeX")]
struct Cli {
    #[arg(long, default_value = ".")]
    workspace_root: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "newnote")]
    Newnote { name: String },
    #[command(name = "init")]
    Init,
    #[command(name = "init_config")]
    InitConfig,
    #[command(name = "rename_file")]
    RenameFile { old: String, new: String },
    #[command(name = "rename_label")]
    RenameLabel {
        note: String,
        old_label: String,
        new_label: String,
    },
    #[command(name = "rename")]
    Rename { name: String },
    #[command(name = "clean")]
    Clean,
    #[command(name = "remove_note")]
    RemoveNote { name: String },
    #[command(name = "list_recent_files")]
    ListRecentFiles { n: Option<usize> },
    #[command(name = "list_unreferenced")]
    ListUnreferenced,
    #[command(name = "rename_recent")]
    RenameRecent { n: Option<usize> },
    #[command(name = "addtodocuments")]
    AddToDocuments { name: String },
    #[command(name = "list_citations")]
    ListCitations { name: String },

    #[command(name = "newproject")]
    Newproject { name: String },
    #[command(name = "list_projects")]
    ListProjects,
    #[command(name = "list_project_inclusions")]
    ListProjectInclusions { project: String },
    #[command(name = "list_note_projects")]
    ListNoteProjects { note: String },
    #[command(name = "export_project")]
    ExportProject {
        folder: String,
        texfile: Option<String>,
    },
    #[command(name = "export_draft")]
    ExportDraft {
        input_file: String,
        output_file: String,
    },
    #[command(name = "to_md")]
    ToMd { note: String },

    #[command(name = "export_markdown")]
    ExportMarkdown { note: String },
    #[command(name = "export_project_markdown")]
    ExportProjectMarkdown { project: String },
    #[command(name = "export_all_markdown")]
    ExportAllMarkdown,
    #[command(name = "export_all_notes_markdown")]
    ExportAllNotesMarkdown,
    #[command(name = "export_all_projects_markdown")]
    ExportAllProjectsMarkdown,

    #[command(name = "render")]
    Render {
        name: String,
        format: Option<String>,
        biber: Option<bool>,
    },
    #[command(name = "render_project")]
    RenderProject {
        name: String,
        format: Option<String>,
        biber: Option<bool>,
    },
    #[command(name = "render_all")]
    RenderAll {
        format: Option<String>,
        #[arg(long, short = 'j')]
        workers: Option<usize>,
    },
    #[command(name = "render_all_pdf")]
    RenderAllPdf {
        #[arg(long, short = 'j')]
        workers: Option<usize>,
    },
    #[command(name = "render_all_projects")]
    RenderAllProjects {
        format: Option<String>,
        #[arg(long, short = 'j')]
        workers: Option<usize>,
    },
    #[command(name = "render_updates")]
    RenderUpdates {
        format: Option<String>,
        #[arg(long, short = 'j')]
        workers: Option<usize>,
    },
    #[command(name = "biber")]
    Biber {
        name: String,
        folder: Option<String>,
    },
    #[command(name = "biber_project")]
    BiberProject {
        name: String,
        folder: Option<String>,
    },

    #[command(name = "synchronize")]
    Synchronize,
    #[command(name = "force_synchronize_notes")]
    ForceSynchronizeNotes,
    #[command(name = "force_synchronize_projects")]
    ForceSynchronizeProjects,
    #[command(name = "force_synchronize")]
    ForceSynchronize,
    #[command(name = "validate_references")]
    ValidateReferences,
    #[command(name = "remove_duplicate_citations")]
    RemoveDuplicateCitations,

    #[command(name = "edit")]
    Edit { name: Option<String> },

    #[command(name = "fuzzy")]
    Fuzzy {
        #[arg(long, default_value_t = false)]
        inline: bool,
        #[arg(long, hide = true)]
        action: Option<String>,
        #[arg(long, hide = true)]
        query: Option<String>,
        #[arg(long, hide = true)]
        item: Option<String>,
        #[arg(long, hide = true)]
        clipboard_text: Option<String>,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    if let Some(Commands::Init) = &cli.command {
        return match init_workspace(&cli.workspace_root) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error inicializando workspace: {e}");
                ExitCode::from(1)
            }
        };
    }

    let paths = match WorkspacePaths::discover(&cli.workspace_root) {
        Ok(paths) => paths,
        Err(e) => {
            error!("{e}");
            eprintln!("Error de workspace: {e}");
            return ExitCode::from(2);
        }
    };

    match cli.command {
        None => {
            println!("zetteltex: usa --help para ver comandos disponibles");
            ExitCode::SUCCESS
        }
        Some(command) => match run_command(command, &paths) {
            Ok(code) => code,
            Err(e) => {
                error!("{e}");
                eprintln!("Error: {e}");
                ExitCode::from(1)
            }
        },
    }
}

fn run_command(command: Commands, paths: &WorkspacePaths) -> Result<ExitCode> {
    match command {
        Commands::Init => {
            // Este comando ya fue manejado en `main()` antes de cargar los paths, 
            // pero Rust requiere que el pattern matching sea exhaustivo.
            Ok(ExitCode::SUCCESS)
        }
        Commands::InitConfig => {
            init_config_interactive(paths)
        }
        Commands::RenameFile { old, new } => {
            rename_file(paths, &old, &new)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenameLabel {
            note,
            old_label,
            new_label,
        } => {
            rename_label(paths, &note, &old_label, &new_label)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Rename { name } => {
            rename_interactive(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Clean => {
            let stats = clean_generated_note_artifacts(paths)?;
            println!("Clean summary: {} pdf(s), {} markdown(s) removed", stats.pdf_removed, stats.markdown_removed);
            Ok(ExitCode::SUCCESS)
        }
        Commands::RemoveNote { name } => {
            remove_note(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::AddToDocuments { name } => {
            add_to_documents(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Newproject { name } => {
            create_project(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Newnote { name } => {
            create_note(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListRecentFiles { n } => {
            list_recent_files(paths, n.unwrap_or(DEFAULT_RECENT_FILES))?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListUnreferenced => {
            list_unreferenced(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenameRecent { n } => {
            rename_recent(paths, n.unwrap_or(DEFAULT_RENAME_RECENT_INDEX))?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListCitations { name } => {
            list_citations(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportProject { folder, texfile } => {
            export_project(paths, &folder, texfile.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportDraft {
            input_file,
            output_file,
        } => {
            export_draft(paths, &input_file, &output_file)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ToMd { note } => {
            to_md(paths, &note)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportMarkdown { note } => {
            export_markdown(paths, &note)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportProjectMarkdown { project } => {
            export_project_markdown(paths, &project)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportAllNotesMarkdown => {
            export_all_notes_markdown(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportAllProjectsMarkdown => {
            export_all_projects_markdown(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportAllMarkdown => {
            export_all_markdown(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Render {
            name,
            format,
            biber,
        } => {
            render_note_cmd(
                paths,
                &name,
                format.as_deref().unwrap_or("pdf"),
                biber.unwrap_or(false),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderProject {
            name,
            format,
            biber,
        } => {
            render_project_cmd(
                paths,
                &name,
                format.as_deref().unwrap_or("pdf"),
                biber.unwrap_or(false),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderAll { format, workers } => {
            render_all_notes_cmd(
                paths,
                format.as_deref().unwrap_or("pdf"),
                workers.unwrap_or(DEFAULT_RENDER_WORKERS),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderAllPdf { workers } => {
            render_all_notes_cmd(paths, "pdf", workers.unwrap_or(DEFAULT_RENDER_WORKERS))?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderAllProjects { format, workers } => {
            render_all_projects_cmd(
                paths,
                format.as_deref().unwrap_or("pdf"),
                workers.unwrap_or(DEFAULT_RENDER_WORKERS),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderUpdates { format, workers } => {
            render_updates_cmd(
                paths,
                format.as_deref().unwrap_or("pdf"),
                workers.unwrap_or(DEFAULT_RENDER_WORKERS),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Biber { name, folder } => {
            run_biber_cmd(paths, &name, folder.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::BiberProject { name, folder } => {
            run_biber_project_cmd(paths, &name, folder.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RemoveDuplicateCitations => {
            remove_duplicate_citations_cmd(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Edit { name } => {
            edit_cmd(paths, name.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Fuzzy {
            inline,
            action,
            query,
            item,
            clipboard_text,
        } => {
            fuzzy_cmd(
                paths,
                inline,
                action.as_deref(),
                query.as_deref(),
                item.as_deref(),
                clipboard_text,
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Synchronize => {
            let note_stats = synchronize_notes(paths)?;
            let project_stats = synchronize_projects(paths)?;
            println!(
                "Sincronizacion completa: {} nota(s), {} link(s), {} referencia(s) sin resolver, {} proyecto(s), {} inclusion(es), {} inclusion(es) sin nota",
                note_stats.notes_synced,
                note_stats.links_built,
                note_stats.unresolved_references,
                project_stats.projects_synced,
                project_stats.inclusions_synced,
                project_stats.missing_notes
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ForceSynchronizeNotes => {
            let stats = synchronize_notes(paths)?;
            println!(
                "Force synchronize notas: {} nota(s), {} link(s), {} referencia(s) sin resolver",
                stats.notes_synced, stats.links_built, stats.unresolved_references
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ForceSynchronizeProjects => {
            let stats = synchronize_projects(paths)?;
            println!(
                "Force synchronize proyectos: {} proyecto(s), {} inclusion(es), {} inclusion(es) sin nota",
                stats.projects_synced, stats.inclusions_synced, stats.missing_notes
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ForceSynchronize => {
            let note_stats = synchronize_notes(paths)?;
            let project_stats = synchronize_projects(paths)?;
            println!(
                "Force synchronize completo: {} nota(s), {} proyecto(s)",
                note_stats.notes_synced, project_stats.projects_synced
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListProjects => {
            let db = init_database(&paths.root.join("slipbox.db"))?;
            let projects = db.list_projects()?;
            if projects.is_empty() {
                println!("No projects found in database.");
                return Ok(ExitCode::SUCCESS);
            }

            println!("Projects:");
            for (idx, project) in projects.iter().enumerate() {
                println!("{}:\t{}", idx + 1, project.name);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListProjectInclusions { project } => {
            let _ = synchronize_notes(paths)?;
            let _ = synchronize_projects(paths)?;
            let db = init_database(&paths.root.join("slipbox.db"))?;
            let inclusions = db.list_project_inclusions_by_name(&project)?;

            if inclusions.is_empty() {
                println!("No inclusions found for project {project}");
                return Ok(ExitCode::SUCCESS);
            }

            println!("Inclusions in project \"{project}\":");
            for (idx, inc) in inclusions.iter().enumerate() {
                if inc.tag.is_empty() {
                    println!(
                        "{}. {} (in {})",
                        idx + 1,
                        inc.note_filename,
                        inc.source_file
                    );
                } else {
                    println!(
                        "{}. {} [tag: {}] (in {})",
                        idx + 1,
                        inc.note_filename,
                        inc.tag,
                        inc.source_file
                    );
                }
            }
            println!("Total: {} notes included", inclusions.len());
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListNoteProjects { note } => {
            let _ = synchronize_notes(paths)?;
            let _ = synchronize_projects(paths)?;
            let db = init_database(&paths.root.join("slipbox.db"))?;
            let projects = db.list_note_projects(&note)?;

            if projects.is_empty() {
                println!("Note {note} is not included in any project");
                return Ok(ExitCode::SUCCESS);
            }

            println!("Projects including note \"{note}\":");
            for (idx, p) in projects.iter().enumerate() {
                if p.tag.is_empty() {
                    println!("{}. {}/{}", idx + 1, p.project_name, p.source_file);
                } else {
                    println!(
                        "{}. {}/{} [tag: {}]",
                        idx + 1,
                        p.project_name,
                        p.source_file,
                        p.tag
                    );
                }
            }
            println!("Total: {} projects", projects.len());
            Ok(ExitCode::SUCCESS)
        }
        Commands::ValidateReferences => {
            let _ = synchronize_notes(paths)?;
            let issues = validate_references(paths)?;

            if issues.is_empty() {
                println!("✓ Todas las referencias son validas");
                return Ok(ExitCode::SUCCESS);
            }

            println!("Se encontraron {} referencia(s) rota(s):", issues.len());
            for issue in issues {
                println!(
                    "- [{}] {} -> {} [{}]",
                    issue.kind, issue.source, issue.target_note, issue.target_label
                );
            }
            Ok(ExitCode::from(1))
        }
    }
}

#[derive(Debug)]
struct SyncStats {
    notes_synced: usize,
    links_built: usize,
    unresolved_references: usize,
}

#[derive(Debug)]
struct ProjectSyncStats {
    projects_synced: usize,
    inclusions_synced: usize,
    missing_notes: usize,
}

#[derive(Debug)]
struct ValidationIssue {
    kind: &'static str,
    source: String,
    target_note: String,
    target_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FuzzyItemKind {
    Note,
    Project,
}

#[derive(Debug, Clone)]
struct FuzzyItem {
    display: String,
    name: String,
    name_lower: String,
    kind: FuzzyItemKind,
}

#[derive(Debug, Clone)]
struct NotePopularity {
    in_refs: f64,
    out_refs: f64,
    total: f64,
}

#[derive(Debug, Default)]
struct FuzzyIndex {
    items: Vec<FuzzyItem>,
    note_content_lower: HashMap<String, String>,
    note_content_original: HashMap<String, String>,
    note_popularity: HashMap<String, NotePopularity>,
    project_preview: HashMap<String, Vec<String>>,
    settings: FuzzySettings,
}

#[derive(Debug, Clone)]
struct FuzzySettings {
    max_results: usize,
    in_refs_weight: f64,
    out_refs_weight: f64,
    accent_color: Color,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ZetteltexConfig {
    #[serde(default)]
    render: RenderConfig,
    #[serde(default)]
    export: ExportConfig,
    #[serde(default)]
    fuzzy: FuzzyConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RenderConfig {
    pdf_output_dir: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ExportConfig {
    obsidian_vault: Option<String>,
    notes_subdir: Option<String>,
    projects_subdir: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct FuzzyConfig {
    max_results: Option<usize>,
    in_refs_weight: Option<f64>,
    out_refs_weight: Option<f64>,
    selection_color: Option<String>,
    state_file: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FuzzyStateFile {
    #[serde(default)]
    history: Vec<String>,
    #[serde(default)]
    popularity_cache: Vec<FuzzyPopularityRow>,
    db_mtime_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FuzzyPopularityRow {
    filename: String,
    in_refs: i64,
    out_refs: i64,
}

impl Default for FuzzySettings {
    fn default() -> Self {
        Self {
            max_results: FUZZY_MAX_RESULTS_DEFAULT,
            in_refs_weight: FUZZY_IN_REFS_WEIGHT_DEFAULT,
            out_refs_weight: FUZZY_OUT_REFS_WEIGHT_DEFAULT,
            accent_color: FUZZY_ACCENT_COLOR_DEFAULT,
        }
    }
}

#[derive(Debug, Clone)]
enum FuzzyUiAction {
    CopyExhyperref { item: FuzzyItem },
    CopyExcref { item: FuzzyItem },
    OpenEditor { item: FuzzyItem },
    OpenPdf { item: FuzzyItem },
    CreateFromQuery { query: String },
    CreateFromClipboard,
}

const FUZZY_MAX_RESULTS_DEFAULT: usize = 50;
const FUZZY_IN_REFS_WEIGHT_DEFAULT: f64 = 1.5;
const FUZZY_OUT_REFS_WEIGHT_DEFAULT: f64 = 1.0;
const FUZZY_HISTORY_LIMIT: usize = 20;
const FUZZY_ACCENT_COLOR_DEFAULT: Color = Color::LightMagenta;

fn synchronize_notes(paths: &WorkspacePaths) -> Result<SyncStats> {
    let db_path = paths.root.join("slipbox.db");
    let db = init_database(&db_path)?;

    // We can also skip parsing files if they haven't changed recently.
    // Let's read all known db edit dates once to avoid N SELECTs.
    // But realistically, transactions will solve 99% of the slowdown!
    db.begin_transaction()?;

    let mut parsed_by_note = HashMap::new();
    let mut notes_synced = 0usize;

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("tex") {
            continue;
        }

        let filename = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let content = fs::read_to_string(&path)?;
        let parsed = parse_note(&content)?;

        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());
        let modified_utc: DateTime<Utc> = modified.into();

        let note_id = db.upsert_note(&filename, modified_utc)?;
        db.replace_labels(note_id, &parsed.labels)?;
        db.replace_citations(note_id, &parsed.citations)?;

        parsed_by_note.insert(filename, parsed);
        notes_synced += 1;
    }

    db.clear_links()?;
    let mut links_built = 0usize;
    let mut unresolved_references = 0usize;

    for (source_note, parsed) in parsed_by_note {
        let Some(source_id) = db.note_id_by_filename(&source_note)? else {
            continue;
        };

        for reference in parsed.references {
            if let Some(target_label_id) =
                db.target_label_id(&reference.target_note, &reference.target_label)?
            {
                db.insert_link(source_id, target_label_id)?;
                links_built += 1;
            } else {
                unresolved_references += 1;
            }
        }
    }
    
    db.commit_transaction()?;

    Ok(SyncStats {
        notes_synced,
        links_built,
        unresolved_references,
    })
}

fn validate_references(paths: &WorkspacePaths) -> Result<Vec<ValidationIssue>> {
    let db_path = paths.root.join("slipbox.db");
    let db = init_database(&db_path)?;
    let mut issues = Vec::new();

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("tex") {
            continue;
        }

        let source = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let content = fs::read_to_string(&path)?;
        let parsed = parse_note(&content)?;

        for reference in parsed.references {
            if !db.note_exists(&reference.target_note)? {
                issues.push(ValidationIssue {
                    kind: "missing_note",
                    source: source.clone(),
                    target_note: reference.target_note,
                    target_label: reference.target_label,
                });
                continue;
            }

            if !db.label_exists(&reference.target_note, &reference.target_label)? {
                issues.push(ValidationIssue {
                    kind: "missing_label",
                    source: source.clone(),
                    target_note: reference.target_note,
                    target_label: reference.target_label,
                });
            }
        }
    }

    Ok(issues)
}

fn synchronize_projects(paths: &WorkspacePaths) -> Result<ProjectSyncStats> {
    let db_path = paths.root.join("slipbox.db");
    let db = init_database(&db_path)?;

    let mut projects_synced = 0usize;
    let mut inclusions_synced = 0usize;
    let mut missing_notes = 0usize;

    db.begin_transaction()?;

    for entry in fs::read_dir(&paths.projects)? {
        let entry = entry?;
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }

        let Some(project_name) = project_dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let project_filename = format!("{project_name}.tex");
        let project_main = project_dir.join(&project_filename);
        if !project_main.exists() {
            continue;
        }

        let modified = fs::metadata(&project_main)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());
        let modified_utc: DateTime<Utc> = modified.into();
        let project_id = db.upsert_project(project_name, &project_filename, modified_utc)?;
        projects_synced += 1;

        let mut tex_files = Vec::new();
        collect_tex_files(&project_dir, &mut tex_files)?;

        let mut resolved_inclusions = Vec::new();
        for tex_file in tex_files {
            let content = fs::read_to_string(&tex_file)?;
            let inclusions = parse_project_inclusions(&content)?;
            let source_file = tex_file
                .strip_prefix(&project_dir)
                .unwrap_or(&tex_file)
                .to_string_lossy()
                .replace('\\', "/");

            for inclusion in inclusions {
                if let Some(note_id) = resolve_note_id(&db, &inclusion.note_filename)? {
                    resolved_inclusions.push((note_id, source_file.clone(), inclusion.tag));
                    inclusions_synced += 1;
                } else {
                    missing_notes += 1;
                }
            }
        }

        db.replace_project_inclusions(project_id, &resolved_inclusions)?;
    }

    db.commit_transaction()?;

    Ok(ProjectSyncStats {
        projects_synced,
        inclusions_synced,
        missing_notes,
    })
}

fn collect_tex_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_tex_files(&path, out)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("tex") {
            out.push(path);
        }
    }
    Ok(())
}

fn resolve_note_id(db: &zetteltex_db::Database, note_ref: &str) -> Result<Option<i64>> {
    let normalized = note_ref.trim().trim_end_matches(".tex");

    if let Some(id) = db.note_id_by_filename(normalized)? {
        return Ok(Some(id));
    }

    let lower = normalized.to_lowercase();
    if let Some(id) = db.note_id_by_filename(&lower)? {
        return Ok(Some(id));
    }

    let kebab = camel_to_kebab(normalized);
    if let Some(id) = db.note_id_by_filename(&kebab)? {
        return Ok(Some(id));
    }

    Ok(None)
}

fn camel_to_kebab(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_is_alnum = false;

    for ch in input.chars() {
        if ch.is_uppercase() {
            if prev_is_alnum {
                out.push('-');
            }
            for c in ch.to_lowercase() {
                out.push(c);
            }
            prev_is_alnum = true;
        } else if ch == '_' || ch == ' ' {
            out.push('-');
            prev_is_alnum = false;
        } else {
            out.push(ch);
            prev_is_alnum = ch.is_alphanumeric();
        }
    }

    out
}

fn create_project(paths: &WorkspacePaths, project_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if db.project_id_by_name(project_name)?.is_some() {
        bail!("A project with name {project_name} already exists in the database");
    }

    let project_dir = paths.projects.join(project_name);
    fs::create_dir_all(&project_dir)?;

    let project_filename = format!("{project_name}.tex");
    let project_tex_path = project_dir.join(&project_filename);
    if !project_tex_path.exists() {
        let template_path = paths.template.join("project.tex");
        let template = fs::read_to_string(&template_path)?;
        let title = title_from_name(project_name);
        let updated = replace_title(&template, &title);
        fs::write(&project_tex_path, updated)?;
    }

    db.upsert_project(project_name, &project_filename, Utc::now())?;
    println!(
        "Project {project_name} created at {}",
        project_dir.display()
    );
    Ok(())
}

fn create_note(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if db.note_exists(note_name)? {
        bail!(
            "A note with file name {note_name} already exists in the database. If this is not the case then run zetteltex synchronize and try again"
        );
    }

    let note_tex_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    if !note_tex_path.exists() {
        let template_path = paths.template.join("note.tex");
        let template = fs::read_to_string(&template_path)?;
        let title = title_from_name(note_name);
        let updated = replace_title(&template, &title);
        fs::write(&note_tex_path, updated)?;
    } else {
        println!(
            "File {} already exists, skipping copying the template",
            note_tex_path.display()
        );
    }

    add_to_documents(paths, note_name)?;

    db.upsert_note(note_name, Utc::now())?;
    Ok(())
}

fn list_recent_files(paths: &WorkspacePaths, n: usize) -> Result<()> {
    let recent = recent_note_names(paths)?;
    if recent.is_empty() {
        println!("No notes found in database.");
        return Ok(());
    }

    for (idx, name) in recent.into_iter().take(n).enumerate() {
        println!("{}:\t{}", idx + 1, name);
    }

    Ok(())
}

fn list_unreferenced(paths: &WorkspacePaths) -> Result<()> {
    let _ = synchronize_notes(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let notes = db.list_unreferenced_notes()?;

    if notes.is_empty() {
        println!("No unreferenced notes found.");
        return Ok(());
    }

    for (idx, note) in notes.iter().enumerate() {
        println!("{}: {}", idx + 1, note);
    }

    Ok(())
}

fn rename_recent(paths: &WorkspacePaths, n: usize) -> Result<()> {
    if n == 0 {
        bail!("n must be >= 1");
    }

    let _ = synchronize_notes(paths)?;
    let recent = recent_note_names(paths)?;
    if n > recent.len() {
        bail!(
            "Requested recent index {n} out of range ({} note(s))",
            recent.len()
        );
    }

    let current = recent[n - 1].clone();
    print!("Change file name to [{}]: ", current);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let new_name = input.trim();

    if new_name.is_empty() || new_name == current {
        println!("No changes made");
        return Ok(());
    }

    rename_file(paths, &current, new_name)
}

fn add_to_documents(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let documents_path = paths.root.join("notes").join("documents.tex");
    let line = format!("\\externaldocument[{note_name}-]{{{note_name}}}\n");
    let mut current = String::new();
    if documents_path.exists() {
        current = fs::read_to_string(&documents_path)?;
    }
    if !current.contains(&line) {
        current.push_str(&line);
        fs::write(&documents_path, current)?;
    }

    Ok(())
}

fn recent_note_names(paths: &WorkspacePaths) -> Result<Vec<String>> {
    let mut entries: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("tex") {
            continue;
        }
        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        entries.push((modified, path));
    }

    entries.sort_by(|a, b| b.0.cmp(&a.0));

    let names = entries
        .into_iter()
        .filter_map(|(_, path)| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect::<Vec<_>>();

    Ok(names)
}

fn list_citations(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!("Query returned no rows");
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let content = fs::read_to_string(&note_path)?;
    let parsed = parse_note(&content)?;

    let mut unique = std::collections::BTreeSet::new();
    for citation in parsed.citations {
        unique.insert(citation);
    }

    for citation in unique {
        println!("{citation}");
    }

    Ok(())
}

fn export_project(
    paths: &WorkspacePaths,
    project_folder: &str,
    texfile: Option<&str>,
) -> Result<()> {
    let texfile = texfile
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{project_folder}.tex"));

    let input_path = paths.projects.join(project_folder).join(&texfile);
    if !input_path.exists() {
        bail!("Project file not found: {}", input_path.display());
    }

    let output_dir = paths.projects.join(project_folder).join("standalone");
    fs::create_dir_all(&output_dir)?;
    let output_path = output_dir.join(&texfile);

    let transclude_re = Regex::new(r"\\transclude(?:\[([^\]]+)\])?\{([^}]+)\}")?;
    let mut output = String::new();

    for raw_line in fs::read_to_string(&input_path)?.lines() {
        let line_without_transcludes = transclude_re.replace_all(raw_line, "").to_string();
        output.push_str(line_without_transcludes.trim());
        output.push('\n');

        for caps in transclude_re.captures_iter(raw_line) {
            let tag = caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "note".to_string());
            let note_name = caps.get(2).map(|m| m.as_str().trim()).unwrap_or_default();
            let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
            let note_content = fs::read_to_string(&note_path)?;

            if let Some(block) = extract_tagged_block(&note_content, &tag)? {
                output.push_str(block.trim());
                output.push('\n');
            } else {
                bail!(
                    "Tag <*{}>...</{}> not found in {}",
                    tag,
                    tag,
                    note_path.display()
                );
            }
        }
    }

    fs::write(output_path, output)?;
    Ok(())
}

fn export_draft(paths: &WorkspacePaths, input_file: &str, output_file: &str) -> Result<()> {
    let input_path = resolve_workspace_path(paths, input_file);
    if !input_path.exists() {
        bail!("Input file not found: {}", input_path.display());
    }

    let output_path = resolve_workspace_path(paths, output_file);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let execute_re = Regex::new(r"\\ExecuteMetaData\[([^\]]+)\]\{([^}]+)\}")?;
    let mut output = String::new();
    let input_parent = input_path.parent().unwrap_or(paths.root.as_path());

    for raw_line in fs::read_to_string(&input_path)?.lines() {
        let line_without_exec = execute_re.replace_all(raw_line, "").to_string();
        output.push_str(line_without_exec.trim());
        output.push('\n');

        for caps in execute_re.captures_iter(raw_line) {
            let import_file = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let tag = caps.get(2).map(|m| m.as_str()).unwrap_or("note");

            let mut import_path = PathBuf::from(import_file);
            if !import_path.is_absolute() {
                let candidate = input_parent.join(&import_path);
                import_path = if candidate.exists() {
                    candidate
                } else {
                    paths.root.join(&import_path)
                };
            }

            let import_content = fs::read_to_string(&import_path)?;
            if let Some(block) = extract_tagged_block(&import_content, tag)? {
                output.push_str(block.trim());
                output.push('\n');
            } else {
                bail!(
                    "Tag <*{}>...</{}> not found in {}",
                    tag,
                    tag,
                    import_path.display()
                );
            }
        }
    }

    fs::write(output_path, output)?;
    Ok(())
}

fn to_md(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let content = fs::read_to_string(&note_path)?;

    let excref_re = Regex::new(r"\\excref\{([^}]+)\}\{([^}]+)\}")?;
    let exref_re = Regex::new(r"\\exref\[([^\]]+)\]\{([^}]+)\}")?;
    let exhyperref_re =
        Regex::new(r"\\exhyperref(?:\[[^\]]+\])?\{([^}]+)\}\{([^}]+)\}\{([^}]*)\}")?;

    let out1 = excref_re
        .replace_all(&content, |caps: &regex::Captures<'_>| {
            format!("[[{}#{}]]", &caps[1], &caps[2])
        })
        .to_string();
    let out2 = exref_re
        .replace_all(&out1, |caps: &regex::Captures<'_>| {
            format!("[[{}#{}]]", &caps[2], &caps[1])
        })
        .to_string();
    let out3 = exhyperref_re
        .replace_all(&out2, |caps: &regex::Captures<'_>| {
            format!("[[{}#{}|{}]]", &caps[1], &caps[2], &caps[3])
        })
        .to_string();

    let markdown_dir = paths.root.join("markdown");
    fs::create_dir_all(&markdown_dir)?;
    let out_path = markdown_dir.join(format!("{note_name}.md"));
    fs::write(&out_path, out3)?;
    println!("Markdown exported to {}", out_path.display());
    Ok(())
}

fn export_markdown(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    println!(
        "Plan export_markdown: nota='{}' | sync=true | salida={} ",
        note_name,
        export_notes_dir(paths).display()
    );
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    export_note_markdown_file(paths, note_name)
}

fn export_project_markdown(paths: &WorkspacePaths, project_name: &str) -> Result<()> {
    println!(
        "Plan export_project_markdown: proyecto='{}' | sync=true | salida={}",
        project_name,
        export_projects_dir(paths).display()
    );
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    export_project_markdown_file(paths, project_name)
}

fn export_all_notes_markdown(paths: &WorkspacePaths) -> Result<()> {
    let note_names: Vec<String> = fs::read_dir(&paths.notes_slipbox)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("tex"))
        .filter_map(|path| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();

    println!(
        "Plan export_all_notes_markdown: notas={} | sync=true | salida={}",
        note_names.len(),
        export_notes_dir(paths).display()
    );

    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;

    let mut count = 0usize;
    for name in &note_names {
        export_note_markdown_file(paths, name)?;
        count += 1;
    }

    println!(
        "Exported {count} note(s) to {}",
        export_notes_dir(paths).display()
    );
    Ok(())
}

fn export_all_projects_markdown(paths: &WorkspacePaths) -> Result<()> {
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;

    let projects = db.list_projects()?;
    println!(
        "Plan export_all_projects_markdown: proyectos={} | sync=true | salida={}",
        projects.len(),
        export_projects_dir(paths).display()
    );

    let mut count = 0usize;
    for p in projects {
        export_project_markdown_file(paths, &p.name)?;
        count += 1;
    }

    println!(
        "Exported {count} project(s) to {}",
        export_projects_dir(paths).display()
    );
    Ok(())
}

fn export_all_markdown(paths: &WorkspacePaths) -> Result<()> {
    println!(
        "Plan export_all_markdown: notas->{} | proyectos->{} | sync=true",
        export_notes_dir(paths).display(),
        export_projects_dir(paths).display()
    );
    export_all_notes_markdown(paths)?;
    export_all_projects_markdown(paths)?;
    Ok(())
}

fn export_note_markdown_file(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!("Note {note_name} not found in database");
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let content = fs::read_to_string(&note_path)?;
    let parsed = parse_note(&content)?;

    let out_dir = export_notes_dir(paths);
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{note_name}.md"));

    let title = extract_title_from_tex_content(&content).unwrap_or_else(|| note_name.to_string());
    let tags = subject_tags_for_note(paths, note_name)?;
    let references = parsed
        .references
        .into_iter()
        .map(|r| r.target_note)
        .collect::<std::collections::BTreeSet<_>>();
    let keywords = extract_keywords_from_tex_content(&content);

    let mut out = String::new();
    if !tags.is_empty() || title != note_name {
        out.push_str("---\n");
        if title != note_name {
            out.push_str(&format!("title: \"{}\"\n", title.replace('"', "\\\"")));
        }
        if !tags.is_empty() {
            out.push_str("tags:\n");
            for tag in &tags {
                out.push_str(&format!("  - {tag}\n"));
            }
        }
        out.push_str("---\n\n");
    }

    out.push_str(&format!("[[{note_name}.pdf]]\n"));
    out.push_str(&format!("![[{note_name}.pdf]]\n\n"));

    if !references.is_empty() {
        out.push_str("## Referencias\n");
        for r in &references {
            out.push_str(&format!("- [{r}](./{r}.md)\n"));
        }
        out.push('\n');
    }

    if !keywords.is_empty() {
        out.push_str("## Etiquetas\n");
        for (k, txt) in keywords {
            out.push_str(&format!("#{k} {txt}\n"));
        }
    }

    fs::write(&out_path, out)?;
    Ok(())
}

fn export_project_markdown_file(paths: &WorkspacePaths, project_name: &str) -> Result<()> {
    let project_dir = paths.projects.join(project_name);
    let main_tex = project_dir.join(format!("{project_name}.tex"));
    if !main_tex.exists() {
        bail!("Project main tex not found: {}", main_tex.display());
    }

    let db = init_database(&paths.root.join("slipbox.db"))?;
    let inclusions = db.list_project_inclusions_by_name(project_name)?;
    let content = fs::read_to_string(&main_tex)?;

    let out_dir = export_projects_dir(paths);
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{project_name}.md"));

    let title =
        extract_title_from_tex_content(&content).unwrap_or_else(|| project_name.to_string());
    let clean_project = clean_project_tag(project_name);
    let keywords = extract_keywords_from_tex_content(&content);

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("title: \"{}\"\n", title.replace('"', "\\\"")));
    if !clean_project.is_empty() {
        out.push_str("tags:\n");
        out.push_str(&format!("  - {}\n", clean_project));
    }
    out.push_str("---\n\n");

    out.push_str(&format!("[[{project_name}.pdf]]\n"));
    out.push_str(&format!("![[{project_name}.pdf]]\n\n"));

    if !inclusions.is_empty() {
        out.push_str("## Notas incluidas\n");
        let mut current_source = String::new();
        for inc in inclusions {
            let source_base = inc.source_file.trim_end_matches(".tex");
            if source_base != current_source {
                out.push_str(&format!("\n### {}\n", source_base));
                current_source = source_base.to_string();
            }
            out.push_str(&format!(
                "- [{}](./{}.md)\n",
                inc.note_filename, inc.note_filename
            ));
        }
        out.push('\n');
    }

    if !keywords.is_empty() {
        out.push_str("## Etiquetas\n");
        for (k, txt) in keywords {
            out.push_str(&format!("#{k} {txt}\n"));
        }
    }

    fs::write(&out_path, out)?;
    Ok(())
}

fn export_notes_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    let vault = config
        .export
        .obsidian_vault
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("jabberwocky"));

    let subdir = config
        .export
        .notes_subdir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("latex").join("zettelkasten"));

    vault.join(subdir)
}

fn export_projects_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    let vault = config
        .export
        .obsidian_vault
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("jabberwocky"));

    let subdir = config
        .export
        .projects_subdir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("latex").join("asignaturas"));

    vault.join(subdir)
}

fn subject_tags_for_note(paths: &WorkspacePaths, note_name: &str) -> Result<Vec<String>> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let projects = db.list_note_projects(note_name)?;
    let mut tags = std::collections::BTreeSet::new();
    for p in projects {
        let clean = clean_project_tag(&p.project_name);
        if clean.is_empty() {
            continue;
        }
        let source = p.source_file.trim_end_matches(".tex");
        tags.insert(format!("{clean}/{source}"));
    }
    Ok(tags.into_iter().collect())
}

fn clean_project_tag(project_name: &str) -> String {
    let without_prefix = project_name
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim_start_matches('-');
    without_prefix.to_string()
}

fn extract_title_from_tex_content(content: &str) -> Option<String> {
    let token = "\\title{";
    let start = content.find(token)? + token.len();
    let mut depth = 1usize;
    let mut i = start;
    let bytes = content.as_bytes();

    while i < bytes.len() {
        match bytes[i] as char {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(content[start..i].trim().to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

fn extract_keywords_from_tex_content(content: &str) -> Vec<(String, String)> {
    let keys = [
        "TODO:",
        "FIXME:",
        "DEMOSTRACION",
        "DEMOSTRACIÓN",
        "ORDENAR",
        "COMPLETAR",
        "EJERCICIO",
        "REVISAR",
        "FALTA",
    ];

    let mut out = Vec::new();
    for line in content.lines() {
        for key in keys {
            if let Some(idx) = line.find(key) {
                let txt = line[idx + key.len()..].trim().to_string();
                out.push((key.trim_end_matches(':').to_string(), txt));
            }
        }
    }
    out
}

fn init_workspace(root: &str) -> Result<()> {
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

fn render_note_cmd(
    paths: &WorkspacePaths,
    name: &str,
    format: &str,
    with_biber: bool,
) -> Result<()> {
    if format != "pdf" {
        bail!("Unsupported format: {format}");
    }

    let passes = if with_biber { 2 } else { 1 };
    println!(
        "Plan render: nota='{}' | formato={} | pasadas={} | biber={} | salida={}",
        name,
        format,
        passes,
        with_biber,
        pdf_output_dir(paths).display()
    );

    render_note_single_pass(paths, name)?;

    if with_biber {
        run_biber_cmd(paths, name, None)?;
        render_note_single_pass(paths, name)?;
    }

    let db = init_database(&paths.root.join("slipbox.db"))?;
    db.set_note_last_build_date_pdf(name, Utc::now())?;
    Ok(())
}

fn render_project_cmd(
    paths: &WorkspacePaths,
    name: &str,
    format: &str,
    with_biber: bool,
) -> Result<()> {
    if format != "pdf" {
        bail!("Unsupported format: {format}");
    }

    let passes = if with_biber { 2 } else { 1 };
    println!(
        "Plan render_project: proyecto='{}' | formato={} | pasadas={} | biber={} | salida={}",
        name,
        format,
        passes,
        with_biber,
        pdf_output_dir(paths).display()
    );

    render_project_single_pass(paths, name)?;

    if with_biber {
        run_biber_project_cmd(paths, name, None)?;
        render_project_single_pass(paths, name)?;
    }

    let db = init_database(&paths.root.join("slipbox.db"))?;
    db.set_project_last_build_date_pdf(name, Utc::now())?;
    Ok(())
}

fn render_all_notes_cmd(paths: &WorkspacePaths, format: &str, workers: usize) -> Result<()> {
    if format != "pdf" {
        bail!("Unsupported format: {format}");
    }

    let db = init_database(&paths.root.join("slipbox.db"))?;
    let mut note_names = Vec::new();
    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("tex") {
            continue;
        }
        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
            note_names.push(name.to_string());
        }
    }

    let mut with_citations = HashMap::new();
    for name in &note_names {
        with_citations.insert(name.clone(), note_contains_citations(paths, name)?);
    }

    let notes_with_biber = with_citations.values().filter(|v| **v).count();
    println!(
        "Plan render_all: notas={} | workers={} | pasadas=2 | formato={} | con_biber={} | salida={}",
        note_names.len(),
        workers.max(1).min(note_names.len().max(1)),
        format,
        notes_with_biber,
        pdf_output_dir(paths).display()
    );

    let paths_pass1 = paths.clone();
    run_parallel_render_with_progress(
        "Render notas · pasada 1/2",
        note_names.clone(),
        workers,
        move |name| {
            render_note_single_pass(&paths_pass1, name)?;
            if with_citations.get(name).copied().unwrap_or(false) {
                run_biber_cmd(&paths_pass1, name, None)?;
            }
            Ok(())
        },
    )?;

    let paths_pass2 = paths.clone();
    run_parallel_render_with_progress(
        "Render notas · pasada 2/2",
        note_names.clone(),
        workers,
        move |name| {
            render_note_single_pass(&paths_pass2, name)?;
            Ok(())
        },
    )?;

    for name in &note_names {
        db.set_note_last_build_date_pdf(name, Utc::now())?;
    }

    Ok(())
}

fn render_all_projects_cmd(paths: &WorkspacePaths, format: &str, workers: usize) -> Result<()> {
    if format != "pdf" {
        bail!("Unsupported format: {format}");
    }

    let db = init_database(&paths.root.join("slipbox.db"))?;
    let mut project_names = Vec::new();
    for entry in fs::read_dir(&paths.projects)? {
        let entry = entry?;
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
            let main = dir.join(format!("{name}.tex"));
            if main.exists() {
                project_names.push(name.to_string());
            }
        }
    }

    println!(
        "Plan render_all_projects: proyectos={} | workers={} | pasadas=2 | formato={} | biber=true | salida={}",
        project_names.len(),
        workers.max(1).min(project_names.len().max(1)),
        format,
        pdf_output_dir(paths).display()
    );

    let paths_pass1 = paths.clone();
    run_parallel_render_with_progress(
        "Render proyectos · pasada 1/2",
        project_names.clone(),
        workers,
        move |name| {
            render_project_single_pass(&paths_pass1, name)?;
            run_biber_project_cmd(&paths_pass1, name, None)?;
            Ok(())
        },
    )?;

    let paths_pass2 = paths.clone();
    run_parallel_render_with_progress(
        "Render proyectos · pasada 2/2",
        project_names.clone(),
        workers,
        move |name| {
            render_project_single_pass(&paths_pass2, name)?;
            Ok(())
        },
    )?;

    for name in &project_names {
        db.set_project_last_build_date_pdf(name, Utc::now())?;
    }

    Ok(())
}

fn render_updates_cmd(paths: &WorkspacePaths, format: &str, workers: usize) -> Result<()> {
    if format != "pdf" {
        bail!("Unsupported format: {format}");
    }

    println!(
        "Plan render_updates: workers={} | formato={} | pasadas_notas=1/2 | pasadas_proyectos=2 | salida={} ",
        workers.max(1),
        format,
        pdf_output_dir(paths).display()
    );
    println!("Preparando render_updates: sincronizando indices...");
    let _ = run_with_sqlite_lock_retry("synchronize notes", || synchronize_notes(paths))?;
    let _ = run_with_sqlite_lock_retry("synchronize projects", || synchronize_projects(paths))?;

    let db = run_with_sqlite_lock_retry("open database", || {
        init_database(&paths.root.join("slipbox.db"))
    })?;
    let notes = db
        .notes_needing_render()?
        .into_iter()
        .map(|n| (n.clone(), db.note_has_citations(&n).unwrap_or(false)))
        .collect::<Vec<_>>();
    let projects = db.projects_needing_render()?;

    let mut note_names = Vec::new();
    let mut with_citations = HashMap::new();
    for (name, with_biber) in notes {
        note_names.push(name.clone());
        with_citations.insert(name, with_biber);
    }

    println!(
        "Render updates: {} nota(s), {} proyecto(s)",
        note_names.len(),
        projects.len()
    );

    if note_names.is_empty() && projects.is_empty() {
        println!("No hay elementos pendientes de renderizado.");
        return Ok(());
    }

    let paths_notes = paths.clone();
    run_parallel_render_with_progress(
        "Render updates · notas",
        note_names.clone(),
        workers,
        move |name| {
            render_note_single_pass(&paths_notes, name)?;
            if with_citations.get(name).copied().unwrap_or(false) {
                run_biber_cmd(&paths_notes, name, None)?;
                render_note_single_pass(&paths_notes, name)?;
            }
            Ok(())
        },
    )?;

    for name in &note_names {
        run_with_sqlite_lock_retry("update note last_build_date_pdf", || {
            db.set_note_last_build_date_pdf(name, Utc::now())
        })?;
    }

    let paths_projects = paths.clone();
    run_parallel_render_with_progress(
        "Render updates · proyectos",
        projects.clone(),
        workers,
        move |name| {
            render_project_single_pass(&paths_projects, name)?;
            run_biber_project_cmd(&paths_projects, name, None)?;
            render_project_single_pass(&paths_projects, name)?;
            Ok(())
        },
    )?;

    for name in &projects {
        run_with_sqlite_lock_retry("update project last_build_date_pdf", || {
            db.set_project_last_build_date_pdf(name, Utc::now())
        })?;
    }

    Ok(())
}

fn run_with_sqlite_lock_retry<T, F>(label: &str, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    const MAX_ATTEMPTS: usize = 8;

    for attempt in 1..=MAX_ATTEMPTS {
        match operation() {
            Ok(value) => return Ok(value),
            Err(err) => {
                let retryable = is_sqlite_lock_error(&err);
                if retryable && attempt < MAX_ATTEMPTS {
                    let backoff_ms = 200_u64 * attempt as u64;
                    warn!(
                        "{} hit sqlite lock (attempt {}/{}), retrying in {}ms",
                        label, attempt, MAX_ATTEMPTS, backoff_ms
                    );
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                    continue;
                }
                return Err(err);
            }
        }
    }

    bail!("{} failed after retries", label)
}

fn is_sqlite_lock_error(err: &anyhow::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("database is locked")
        || msg.contains("database table is locked")
        || msg.contains("database busy")
}

#[derive(Debug)]
enum RenderEvent {
    Started(String),
    Finished(String),
    Failed { file: String, error: String },
}

#[derive(Debug, Clone, Copy)]
struct ProgressLineLayout {
    max_cols: usize,
    file_width: usize,
    bar_width: usize,
    counter_digits: usize,
}

fn run_parallel_render_with_progress<F>(
    phase_label: &str,
    items: Vec<String>,
    workers: usize,
    job: F,
) -> Result<()>
where
    F: Fn(&str) -> Result<()> + Send + Sync + 'static,
{
    if items.is_empty() {
        return Ok(());
    }

    let worker_count = workers.max(1).min(items.len());
    let total = items.len() as u64;
    let use_tty_progress = std::io::stdout().is_terminal();
    let progress_layout = if use_tty_progress {
        Some(build_progress_line_layout(total))
    } else {
        None
    };

    let queue = Arc::new(Mutex::new(items));
    let job = Arc::new(job);
    let (event_tx, event_rx) = mpsc::channel::<RenderEvent>();

    let mut handles = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let queue = Arc::clone(&queue);
        let job = Arc::clone(&job);
        let event_tx = event_tx.clone();
        handles.push(std::thread::spawn(move || {
            loop {
                let next = {
                    let mut guard = queue.lock().expect("render queue lock poisoned");
                    guard.pop()
                };

                let Some(file) = next else {
                    break;
                };

                let _ = event_tx.send(RenderEvent::Started(file.clone()));
                match job(&file) {
                    Ok(()) => {
                        let _ = event_tx.send(RenderEvent::Finished(file));
                    }
                    Err(err) => {
                        let _ = event_tx.send(RenderEvent::Failed {
                            file,
                            error: err.to_string(),
                        });
                    }
                }
            }
        }));
    }
    drop(event_tx);

    let mut completed = 0usize;
    let mut active = std::collections::HashSet::new();
    let mut errors = Vec::new();
    let started_at = Instant::now();
    let mut current_file = String::from("-");
    let mut smoothed_secs_per_item: Option<f64> = None;
    let mut eta_anchor_secs: u64 = 0;
    let mut eta_anchor_at = Instant::now();

    if use_tty_progress {
        render_compact_progress_line(
            progress_layout.as_ref().expect("layout must exist"),
            &current_file,
            0,
            total,
            started_at.elapsed(),
            Duration::from_secs(0),
        )?;
    }

    while completed < total as usize {
        match event_rx.recv_timeout(Duration::from_millis(120)) {
            Ok(RenderEvent::Started(file)) => {
                current_file = file.clone();
                active.insert(file);
                if use_tty_progress {
                    let elapsed = started_at.elapsed();
                    let eta = eta_anchor_secs.saturating_sub(eta_anchor_at.elapsed().as_secs());
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta),
                    )?;
                }
            }
            Ok(RenderEvent::Finished(file)) => {
                active.remove(&file);
                completed += 1;
                current_file = if active.is_empty() {
                    file
                } else {
                    active.iter().next().cloned().unwrap_or_else(|| "-".to_string())
                };

                // ETA suavizado: EMA de segundos por item sobre throughput global,
                // y luego cuenta regresiva entre eventos para evitar dientes de sierra.
                let elapsed = started_at.elapsed();
                let current_secs_per_item = elapsed.as_secs_f64() / completed as f64;
                smoothed_secs_per_item = Some(match smoothed_secs_per_item {
                    Some(prev) => (0.22 * current_secs_per_item) + (0.78 * prev),
                    None => current_secs_per_item,
                });
                let remaining = (total as usize).saturating_sub(completed) as f64;
                let eta_estimated = smoothed_secs_per_item.unwrap_or(0.0) * remaining;
                eta_anchor_secs = eta_estimated.max(0.0).round() as u64;
                eta_anchor_at = Instant::now();

                if use_tty_progress {
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta_anchor_secs),
                    )?;
                }
            }
            Ok(RenderEvent::Failed { file, error }) => {
                active.remove(&file);
                completed += 1;
                current_file = if active.is_empty() {
                    file.clone()
                } else {
                    active.iter().next().cloned().unwrap_or_else(|| "-".to_string())
                };

                let elapsed = started_at.elapsed();
                let current_secs_per_item = elapsed.as_secs_f64() / completed as f64;
                smoothed_secs_per_item = Some(match smoothed_secs_per_item {
                    Some(prev) => (0.22 * current_secs_per_item) + (0.78 * prev),
                    None => current_secs_per_item,
                });
                let remaining = (total as usize).saturating_sub(completed) as f64;
                let eta_estimated = smoothed_secs_per_item.unwrap_or(0.0) * remaining;
                eta_anchor_secs = eta_estimated.max(0.0).round() as u64;
                eta_anchor_at = Instant::now();

                if use_tty_progress {
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta_anchor_secs),
                    )?;
                }
                errors.push(format!("{}: {}", file, error));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if use_tty_progress {
                    let elapsed = started_at.elapsed();
                    let eta = eta_anchor_secs.saturating_sub(eta_anchor_at.elapsed().as_secs());
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta),
                    )?;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if use_tty_progress {
        println!();
    }

    for handle in handles {
        let _ = handle.join();
    }

    if errors.is_empty() {
        println!("{} | completado", phase_label);
        return Ok(());
    }

    let total_errors = errors.len();
    let first_error = errors.remove(0);
    println!("{} | errores: {}", phase_label, total_errors);
    bail!(
        "{} fallo en {} archivo(s). Primer error: {}",
        phase_label,
        total_errors,
        first_error
    )
}

fn render_compact_progress_line(
    layout: &ProgressLineLayout,
    current_file: &str,
    completed: u64,
    total: u64,
    elapsed: Duration,
    eta: Duration,
) -> Result<()> {
    let counter = format!(
        "{:0width$}/{:0width$}",
        completed,
        total,
        width = layout.counter_digits
    );
    let elapsed_s = format_hhmmss(elapsed.as_secs());
    let eta_s = format_hhmmss(eta.as_secs());

    let filled = if total == 0 {
        0
    } else {
        ((completed as usize) * layout.bar_width) / (total as usize)
    };
    let bar = format!(
        "{}{}",
        "#".repeat(filled),
        "-".repeat(layout.bar_width.saturating_sub(filled))
    );
    let file_short = fit_file_field(current_file, layout.file_width);
    let mut line = format!(
        "{} [{}] {} [{}/{}]",
        file_short, bar, counter, elapsed_s, eta_s
    );

    // Reserva siempre 1 columna para evitar autowrap al borde derecho,
    // que provoca salto de linea visual en muchos terminales.
    if line.chars().count() > layout.max_cols {
        line = line.chars().take(layout.max_cols).collect::<String>();
    }

    // Limpia la linea actual y reescribe sobre la misma.
    print!("\r\x1b[2K{}", line);
    io::stdout().flush()?;
    Ok(())
}

fn terminal_columns() -> usize {
    match terminal_size() {
        Ok((w, _)) => w as usize,
        Err(_) => 100,
    }
}

fn build_progress_line_layout(total: u64) -> ProgressLineLayout {
    let cols = terminal_columns().max(40);
    let max_cols = cols.saturating_sub(1);
    let counter_digits = total.to_string().len().max(1);

    // "0000/1105" -> 2*digits + 1
    let counter_width = (2 * counter_digits) + 1;
    // "[00:00:00/00:00:00]" -> 19
    let time_block_width = 19usize;
    // Separadores fijos en: "<file> [<bar>] <counter> <time>"
    let separators_width = 5usize;

    let mut available_for_file_and_bar =
        max_cols.saturating_sub(counter_width + time_block_width + separators_width);

    // Minimos para mantener legibilidad.
    if available_for_file_and_bar < 16 {
        available_for_file_and_bar = 16;
    }

    let min_file_width = 8usize;
    let min_bar_width = 8usize;

    let mut file_width = (available_for_file_and_bar / 3).clamp(min_file_width, 36);
    let mut bar_width = available_for_file_and_bar.saturating_sub(file_width);

    if bar_width < min_bar_width {
        let delta = min_bar_width - bar_width;
        file_width = file_width.saturating_sub(delta);
        bar_width = available_for_file_and_bar.saturating_sub(file_width);
    }

    if file_width < min_file_width {
        file_width = min_file_width;
        bar_width = available_for_file_and_bar.saturating_sub(file_width).max(min_bar_width);
    }

    ProgressLineLayout {
        max_cols,
        file_width,
        bar_width,
        counter_digits,
    }
}

fn fit_file_field(name: &str, width: usize) -> String {
    let mut s: String = name.chars().take(width).collect();
    let len = s.chars().count();
    if len < width {
        s.push_str(&" ".repeat(width - len));
    }
    s
}

fn format_hhmmss(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn render_note_single_pass(paths: &WorkspacePaths, name: &str) -> Result<()> {
    let note_path = paths.notes_slipbox.join(format!("{name}.tex"));
    if !note_path.exists() {
        bail!("No such file: {}", note_path.display());
    }

    let output_dir = pdf_output_dir(paths);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    
    let file_name = note_path.file_name().unwrap().to_string_lossy();

    run_external_tool(
        "pdflatex",
        &[
            "--interaction=scrollmode",
            &format!("--jobname={name}"),
            "-shell-escape",
            &format!("-output-directory={}", output_dir.display()),
            file_name.as_ref(),
        ],
        Some(&paths.notes_slipbox),
    )
}

fn render_project_single_pass(paths: &WorkspacePaths, name: &str) -> Result<()> {
    let project_dir = paths.projects.join(name);
    let project_path = project_dir.join(format!("{name}.tex"));
    if !project_path.exists() {
        bail!("No such file: {}", project_path.display());
    }

    let output_dir = pdf_output_dir(paths);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    
    let file_name = project_path.file_name().unwrap().to_string_lossy();

    run_external_tool(
        "pdflatex",
        &[
            "--interaction=scrollmode",
            &format!("--jobname={name}"),
            "-shell-escape",
            &format!("-output-directory={}", output_dir.display()),
            file_name.as_ref(),
        ],
        Some(&project_dir),
    )
}

fn note_contains_citations(paths: &WorkspacePaths, name: &str) -> Result<bool> {
    let note_path = paths.notes_slipbox.join(format!("{name}.tex"));
    let content = fs::read_to_string(note_path)?;
    let parsed = parse_note(&content)?;
    Ok(!parsed.citations.is_empty())
}

fn run_biber_cmd(paths: &WorkspacePaths, name: &str, folder: Option<&str>) -> Result<()> {
    let output_dir = resolve_biber_folder(paths, folder);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    
    run_external_tool(
        "biber", 
        &[
            &format!("--output-directory={}", output_dir.display()),
            name
        ], 
        Some(&paths.notes_slipbox)
    )
}

fn run_biber_project_cmd(paths: &WorkspacePaths, name: &str, folder: Option<&str>) -> Result<()> {
    let output_dir = resolve_biber_folder(paths, folder);
    let project_dir = paths.projects.join(name);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    
    run_external_tool(
        "biber", 
        &[
            &format!("--output-directory={}", output_dir.display()),
            name
        ], 
        Some(&project_dir)
    )
}

fn resolve_biber_folder(paths: &WorkspacePaths, folder: Option<&str>) -> PathBuf {
    match folder {
        Some(raw) if !raw.is_empty() => {
            let candidate = PathBuf::from(raw);
            if candidate.is_absolute() {
                candidate
            } else {
                paths.root.join(candidate)
            }
        }
        _ => pdf_output_dir(paths),
    }
}

fn pdf_output_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    config
        .render
        .pdf_output_dir
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("pdf"))
}

fn run_external_tool(bin: &str, args: &[&str], cwd: Option<&Path>) -> Result<()> {
    let mut cmd = Command::new(bin);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    let output = match cmd.output() {
        Ok(out) => out,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            bail!("{bin} not found in PATH")
        }
        Err(err) => return Err(err.into()),
    };
    if !output.status.success() {
        let rendered_cmd = format!("{} {}", bin, args.join(" "));
        let cwd_display = cwd
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<current-dir>".to_string());
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let detail = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else if !stdout.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            format!("exit status {} (no stderr/stdout)", output.status)
        };

        bail!(
            "{} failed while running '{}' in {}: {}",
            bin,
            rendered_cmd,
            cwd_display,
            detail
        );
    }
    Ok(())
}

fn run_external_open_nonblocking_verified(bin: &str, args: &[&str], cwd: Option<&Path>) -> Result<()> {
    fn spawn_and_verify(mut cmd: Command, cwd: Option<&Path>) -> Result<()> {
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }

        let mut child = cmd.spawn()?;

        // Espera corta para detectar errores inmediatos sin bloquear la salida de fuzzy.
        let timeout = Duration::from_millis(350);
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if status.success() {
                    return Ok(());
                }
                bail!("open command failed with status {status}")
            }

            if start.elapsed() >= timeout {
                // Sigue vivo: consideramos que se lanzo correctamente.
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    // En Linux, setsid -f suele desacoplar mejor del terminal que un spawn directo.
    if command_exists("setsid") {
        let mut cmd = Command::new("setsid");
        cmd.arg("-f").arg(bin).args(args);
        if spawn_and_verify(cmd, cwd).is_ok() {
            return Ok(());
        }
    }

    // Fallback clasico para evitar SIGHUP al cerrar el terminal.
    if command_exists("nohup") {
        let mut cmd = Command::new("nohup");
        cmd.arg(bin).args(args);
        if spawn_and_verify(cmd, cwd).is_ok() {
            return Ok(());
        }
    }

    // Ultimo intento: lanzamiento directo.
    let mut cmd = Command::new(bin);
    cmd.args(args);
    spawn_and_verify(cmd, cwd)
}

fn remove_duplicate_citations_cmd(paths: &WorkspacePaths) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let removed = db.remove_duplicate_citations()?;
    if removed > 0 {
        println!("Removed {removed} duplicate citation(s)");
    } else {
        println!("No duplicate citations found");
    }
    Ok(())
}

fn edit_cmd(paths: &WorkspacePaths, filename: Option<&str>) -> Result<()> {
    let note_name = match filename {
        Some(name) if !name.trim().is_empty() => name.to_string(),
        _ => {
            let recent = recent_note_names(paths)?;
            recent
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No notes found to edit"))?
        }
    };

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    if !note_path.exists() {
        bail!("No such file: {}", note_path.display());
    }

    open_in_editor(&note_path)?;
    Ok(())
}

fn open_in_editor(file_path: &Path) -> Result<()> {
    let mut candidates = Vec::new();
    if let Ok(custom) = std::env::var("ZETTELTEX_EDITOR") {
        if !custom.trim().is_empty() {
            candidates.push(custom);
        }
    }
    candidates.push("code".to_string());
    candidates.push("/usr/bin/code".to_string());
    candidates.push("/usr/local/bin/code".to_string());
    candidates.push("/snap/bin/code".to_string());
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(Path::new(&home).join(".local/bin/code").to_string_lossy().to_string());
    }
    candidates.push("xdg-open".to_string());

    for cmd_name in candidates {
        match Command::new(&cmd_name).arg(file_path).status() {
            Ok(status) if status.success() => return Ok(()),
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    bail!("Could not open editor for {}", file_path.display())
}

fn fuzzy_cmd(
    paths: &WorkspacePaths,
    inline: bool,
    action: Option<&str>,
    query: Option<&str>,
    item: Option<&str>,
    clipboard_text: Option<String>,
) -> Result<()> {
    if let Some(action_name) = action {
        return run_fuzzy_scripted_action(paths, action_name, query, item, clipboard_text);
    }

    if inline {
        return run_fuzzy_inline(paths);
    }

    launch_fuzzy_in_new_terminal(paths)
}

fn run_fuzzy_scripted_action(
    paths: &WorkspacePaths,
    action_name: &str,
    query: Option<&str>,
    item: Option<&str>,
    clipboard_text: Option<String>,
) -> Result<()> {
    let index = build_fuzzy_index(paths)?;

    let action = match action_name {
        "copy-exhyperref" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::CopyExhyperref { item }
        }
        "copy-excref" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::CopyExcref { item }
        }
        "open-editor" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::OpenEditor { item }
        }
        "open-pdf" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::OpenPdf { item }
        }
        "create-from-query" => FuzzyUiAction::CreateFromQuery {
            query: query.unwrap_or_default().to_string(),
        },
        "create-from-clipboard" => FuzzyUiAction::CreateFromClipboard,
        other => {
            bail!(
                "Accion fuzzy no reconocida: {} (usa copy-exhyperref|copy-excref|open-editor|open-pdf|create-from-query|create-from-clipboard)",
                other
            )
        }
    };

    run_fuzzy_action(paths, &index, action, clipboard_text)
}

fn resolve_scripted_item(index: &FuzzyIndex, query: Option<&str>, item: Option<&str>) -> Result<FuzzyItem> {
    if let Some(target) = item {
        if let Some(found) = index
            .items
            .iter()
            .find(|i| i.display == target || i.name == target)
        {
            return Ok(found.clone());
        }
        bail!("No se encontro item fuzzy: {}", target)
    }

    if let Some(q) = query {
        let results = fuzzy_search(index, q, 1);
        if let Some((first, _)) = results.into_iter().next() {
            return Ok(first.clone());
        }
        bail!("No hay resultados fuzzy para query: {}", q)
    }

    bail!("Debes pasar --item o --query para acciones fuzzy scripted")
}

fn run_fuzzy_inline(paths: &WorkspacePaths) -> Result<()> {
    let index = build_fuzzy_index(paths)?;

    if index.items.is_empty() {
        println!("No hay notas ni proyectos para fuzzy.");
        return Ok(());
    }

    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        let action = run_fuzzy_tui(paths, &index)?;
        if let Some(action) = action {
            run_fuzzy_action(paths, &index, action, None)?;
        }
        return Ok(());
    }

    println!("Fuzzy inline (motor nativo Rust - fase de indexado)");
    println!("Escribe un termino y presiona Enter (linea vacia para salir).\n");

    loop {
        print!("fuzzy> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let query = line.trim();

        if query.is_empty() {
            break;
        }

        let results = fuzzy_search(&index, query, index.settings.max_results);
        if results.is_empty() {
            println!("Sin resultados.\n");
            continue;
        }

        for (idx, (item, score)) in results.iter().take(10).enumerate() {
            println!("{}. {} ({:.1})", idx + 1, item.display, score);
        }
        println!();
    }

    Ok(())
}

fn run_fuzzy_tui(paths: &WorkspacePaths, index: &FuzzyIndex) -> Result<Option<FuzzyUiAction>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    struct UiGuard;
    impl Drop for UiGuard {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
            let mut stdout = io::stdout();
            let _ = execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture);
        }
    }
    let _guard = UiGuard;

    let mut query = String::new();
    let mut cursor_pos = 0usize;
    let mut selected = 0usize;
    let mut preview_scroll = 0u16;
    let mut status_line: Option<String> = None;
    let history = load_fuzzy_history(paths, &index.items).unwrap_or_default();

    loop {
        let results = fuzzy_results_for_ui(index, &query, index.settings.max_results, &history);
        if selected >= results.len() {
            selected = results.len().saturating_sub(1);
        }

        terminal.draw(|f| {
            render_fuzzy_frame(
                f,
                index,
                &query,
                &results,
                selected,
                preview_scroll,
                status_line.as_deref(),
                cursor_pos,
            );
        })?;

        if !event::poll(Duration::from_millis(16))? {
            continue;
        }

        let Event::Key(KeyEvent {
            code,
            modifiers,
            ..
        }) = event::read()?
        else {
            continue;
        };

        match (code, modifiers) {
            (KeyCode::Esc, _) => return Ok(None),
            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => return Ok(None),
            (KeyCode::Backspace, m) if m.contains(KeyModifiers::CONTROL) => {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyExhyperref { item: (*item).clone() }));
                }
            }
            (KeyCode::Left, m) if m.contains(KeyModifiers::CONTROL) => {
                let chars: Vec<char> = query.chars().collect();
                if cursor_pos > 0 {
                    let mut p = cursor_pos - 1;
                    while p > 0 && chars[p].is_whitespace() { p -= 1; }
                    while p > 0 && !chars[p - 1].is_whitespace() { p -= 1; }
                    cursor_pos = p;
                }
            }
            (KeyCode::Right, m) if m.contains(KeyModifiers::CONTROL) => {
                let chars: Vec<char> = query.chars().collect();
                if cursor_pos < chars.len() {
                    let mut p = cursor_pos;
                    while p < chars.len() && chars[p].is_whitespace() { p += 1; }
                    while p < chars.len() && !chars[p].is_whitespace() { p += 1; }
                    cursor_pos = p;
                }
            }
            (KeyCode::Left, _) => {
                cursor_pos = cursor_pos.saturating_sub(1);
            }
            (KeyCode::Right, _) => {
                if cursor_pos < query.chars().count() {
                    cursor_pos += 1;
                }
            }
            (KeyCode::Home, _) => {
                cursor_pos = 0;
            }
            (KeyCode::End, _) => {
                cursor_pos = query.chars().count();
            }
            (KeyCode::Up, _) => {
                selected = selected.saturating_sub(1);
                preview_scroll = 0;
                status_line = None;
            }
            (KeyCode::Down, _) => {
                if selected + 1 < results.len() {
                    selected += 1;
                }
                preview_scroll = 0;
                status_line = None;
            }
            (KeyCode::Backspace, _) => {
                if cursor_pos > 0 {
                    let mut chars: Vec<char> = query.chars().collect();
                    chars.remove(cursor_pos - 1);
                    query = chars.into_iter().collect();
                    cursor_pos -= 1;
                    selected = 0;
                    preview_scroll = 0;
                    status_line = None;
                }
            }
            (KeyCode::Delete, _) => {
                let mut chars: Vec<char> = query.chars().collect();
                if cursor_pos < chars.len() {
                    chars.remove(cursor_pos);
                    query = chars.into_iter().collect();
                    selected = 0;
                    preview_scroll = 0;
                    status_line = None;
                }
            }
            (KeyCode::PageDown, _) => {
                preview_scroll = preview_scroll.saturating_add(5);
                status_line = None;
            }
            (KeyCode::PageUp, _) => {
                preview_scroll = preview_scroll.saturating_sub(5);
                status_line = None;
            }
            (KeyCode::Enter, _) => {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyExhyperref { item: (*item).clone() }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'h') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyExhyperref { item: (*item).clone() }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'r') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyExcref { item: (*item).clone() }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'e') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::OpenEditor { item: (*item).clone() }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'p') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::OpenPdf { item: (*item).clone() }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'o') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::OpenPdf { item: (*item).clone() }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL)
                    && m.contains(KeyModifiers::ALT)
                    && ch.eq_ignore_ascii_case(&'n') =>
            {
                if query.trim().is_empty() {
                    return Ok(Some(FuzzyUiAction::CreateFromClipboard));
                }
                status_line = Some("Ctrl+Alt+N requiere barra de busqueda vacia".to_string());
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'n') =>
            {
                return Ok(Some(FuzzyUiAction::CreateFromQuery {
                    query: query.clone(),
                }));
            }
            (KeyCode::Char(ch), m) if m.is_empty() || m == KeyModifiers::SHIFT => {
                let mut chars: Vec<char> = query.chars().collect();
                chars.insert(cursor_pos, ch);
                query = chars.into_iter().collect();
                cursor_pos += 1;
                selected = 0;
                preview_scroll = 0;
                status_line = None;
            }
            _ => {}
        }
    }
}

fn fuzzy_results_for_ui<'a>(
    index: &'a FuzzyIndex,
    query: &str,
    max_results: usize,
    history: &[String],
) -> Vec<(&'a FuzzyItem, f64)> {
    if !query.trim().is_empty() {
        return fuzzy_search(index, query, max_results);
    }

    // Paridad con fuzzy.py: historial en orden de recencia y luego populares.
    let target = 10usize;
    let mut out = Vec::new();
    for entry in history.iter().take(target) {
        if let Some(item) = index.items.iter().find(|i| &i.display == entry) {
            let score = if item.kind == FuzzyItemKind::Note {
                index
                    .note_popularity
                    .get(&item.name)
                    .map(|p| p.total)
                    .unwrap_or(0.0)
                    + 1000.0
            } else {
                1000.0
            };
            out.push((item, score));
        }
    }

    // Completar con populares (sin alterar el orden del historial ya cargado).
    let mut popular_candidates = Vec::new();
    for item in &index.items {
        if history.iter().any(|h| h == &item.display) {
            continue;
        }
        let score = if item.kind == FuzzyItemKind::Note {
            index
                .note_popularity
                .get(&item.name)
                .map(|p| p.total)
                .unwrap_or(0.0)
        } else {
            0.0
        };
        popular_candidates.push((item, score));
    }

    popular_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (item, score) in popular_candidates {
        if out.len() >= target {
            break;
        }
        out.push((item, score));
    }

    out.truncate(target.min(max_results));
    out
}

fn render_fuzzy_frame(
    f: &mut Frame,
    index: &FuzzyIndex,
    query: &str,
    results: &[(&FuzzyItem, f64)],
    selected: usize,
    preview_scroll: u16,
    status_line: Option<&str>,
    cursor_pos: usize,
) {
    f.render_widget(Clear, f.area());

    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer_chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(main_chunks[0]);

    render_search_bar(f, query, left_chunks[0], index.settings.accent_color, cursor_pos);
    render_results_list(f, results, selected, left_chunks[1], index.settings.accent_color);
    render_preview_panel(
        f,
        index,
        query,
        results,
        selected,
        preview_scroll,
        main_chunks[1],
    );
    render_help_bar(f, outer_chunks[1], status_line, index.settings.accent_color);
}

fn render_search_bar(f: &mut Frame, query: &str, area: Rect, accent_color: Color, cursor_pos: usize) {
    let paragraph = Paragraph::new(query.to_string())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Buscar ")
                .border_style(Style::default().fg(accent_color)),
        );
    f.render_widget(paragraph, area);
    
    // Set terminal cursor position
    let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    let max_x = inner_area.right().saturating_sub(1);
    let target_x = inner_area.x + cursor_pos as u16;
    f.set_cursor_position(ratatui::layout::Position::new(target_x.min(max_x), inner_area.y));
}

fn render_results_list(
    f: &mut Frame,
    results: &[(&FuzzyItem, f64)],
    selected: usize,
    area: Rect,
    accent_color: Color,
) {
    let items = results
        .iter()
        .map(|(item, _)| ListItem::new(item.display.clone()).style(Style::default().fg(Color::White)))
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Resultados ({}) ", results.len()))
                .border_style(Style::default().fg(accent_color)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(":: ");

    let mut state = ListState::default();
    if !results.is_empty() {
        state.select(Some(selected));
    }
    f.render_stateful_widget(list, area, &mut state);
}

fn render_help_bar(f: &mut Frame, area: Rect, status_line: Option<&str>, accent_color: Color) {
    let help = "Ctrl+H: exhyperref | Ctrl+R: excref | Ctrl+E: VSCode | Ctrl+P: PDF | Ctrl+N: Nueva nota | Ctrl+Alt+N: Portapapeles | Esc: salir";
    let (text, style) = if let Some(msg) = status_line {
        (msg, Style::default().fg(accent_color).add_modifier(Modifier::BOLD))
    } else {
        (help, Style::default().fg(Color::Gray))
    };
    let paragraph = Paragraph::new(text).style(style).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_preview_panel(
    f: &mut Frame,
    index: &FuzzyIndex,
    query: &str,
    results: &[(&FuzzyItem, f64)],
    selected: usize,
    preview_scroll: u16,
    area: Rect,
) {
    let search_term = query.to_lowercase();
    let preview = results
        .get(selected)
        .map(|(item, _)| preview_lines_for_item(index, item, area.height.saturating_sub(2) as usize))
        .unwrap_or_else(|| vec!["No hay resultados".to_string()]);

    let lines = preview
        .iter()
        .map(|line| highlight_latex_line(line, &search_term, index.settings.accent_color))
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Vista Previa ")
                .border_style(Style::default().fg(index.settings.accent_color)),
        )
        .wrap(Wrap { trim: false })
        .scroll((preview_scroll, 0));

    f.render_widget(paragraph, area);
}

fn latex_highlight_regexes() -> &'static [Regex; 5] {
    static RE: OnceLock<[Regex; 5]> = OnceLock::new();
    RE.get_or_init(|| {
        [
            Regex::new(r"%.*$").expect("regex comentario valida"),
            Regex::new(r"\\(begin|end)\{[^}]*\}").expect("regex entorno valida"),
            Regex::new(r"\\[a-zA-Z]+\*?").expect("regex comando valida"),
            Regex::new(r"\$[^\$]+\$").expect("regex math valida"),
            Regex::new(r"[{}\[\]]").expect("regex delimitador valida"),
        ]
    })
}

fn highlight_latex_line(line: &str, search_term: &str, accent_color: Color) -> Line<'static> {
    let mut spans = Vec::new();
    let mut marks = Vec::new();

    for (idx, re) in latex_highlight_regexes().iter().enumerate() {
        let color = match idx {
            0 => Color::Gray,
            1 | 2 => accent_color,
            _ => Color::White,
        };
        for m in re.find_iter(line) {
            marks.push((m.start(), m.end(), color, false));
        }
    }

    if !search_term.is_empty() {
        let lower = line.to_lowercase();
        let mut start = 0usize;
        while let Some(pos) = lower[start..].find(search_term) {
            let s = start + pos;
            let e = s + search_term.len();
            marks.push((s, e, Color::Black, true));
            start = e;
        }
    }

    marks.sort_by_key(|m| m.0);

    let mut last = 0usize;
    for (s, e, color, is_search) in marks {
        if s < last {
            continue;
        }
        if s > last {
            spans.push(Span::raw(line[last..s].to_string()));
        }
        let text = line[s..e].to_string();
        if is_search {
            spans.push(Span::styled(
                text,
                Style::default()
                    .bg(accent_color)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(text, Style::default().fg(color)));
        }
        last = e;
    }
    if last < line.len() {
        spans.push(Span::raw(line[last..].to_string()));
    }
    if spans.is_empty() {
        spans.push(Span::raw(line.to_string()));
    }
    Line::from(spans)
}

fn run_fuzzy_action(
    paths: &WorkspacePaths,
    index: &FuzzyIndex,
    action: FuzzyUiAction,
    clipboard_override: Option<String>,
) -> Result<()> {
    match action {
        FuzzyUiAction::CopyExhyperref { item } => {
            let text = build_exhyperref_for_item(paths, index, &item)?;
            write_xclip_clipboard(&text)?;
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::CopyExcref { item } => {
            let text = build_excref_for_item(paths, index, &item)?;
            write_xclip_clipboard(&text)?;
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::OpenEditor { item } => {
            if item.kind == FuzzyItemKind::Project {
                let path = paths.projects.join(&item.name);
                open_in_editor(&path)?;
            } else {
                let path = paths.notes_slipbox.join(format!("{}.tex", item.name));
                open_in_editor(&path)?;
            }
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::OpenPdf { item } => {
            open_pdf_best_effort(paths, &item.name)?;
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::CreateFromQuery { query } => {
            let name = normalize_new_note_name(&query)?;
            create_note(paths, &name)?;
            let note_path = paths.notes_slipbox.join(format!("{}.tex", name));
            open_in_editor(&note_path)?;
            save_history_entry(paths, &name)?;
        }
        FuzzyUiAction::CreateFromClipboard => {
            let content = clipboard_override.unwrap_or(read_xclip_clipboard()?);
            let name = note_name_from_clipboard_label(&content)?;
            create_note(paths, &name)?;
            let note_path = paths.notes_slipbox.join(format!("{}.tex", name));
            inject_clipboard_into_note_template(&note_path, &content)?;
            open_in_editor(&note_path)?;
            write_xclip_clipboard(&format!(r"\transclude{{{}}}", name))?;
            save_history_entry(paths, &name)?;
        }
    }
    Ok(())
}

fn fuzzy_pdf_candidate_paths(paths: &WorkspacePaths, item_name: &str) -> Vec<PathBuf> {
    vec![
        pdf_output_dir(paths).join(format!("{}.pdf", item_name)),
        paths
            .root
            .join("jabberwocky")
            .join("adjuntos")
            .join("pdf")
            .join(format!("{}.pdf", item_name)),
    ]
}

fn open_pdf_best_effort(paths: &WorkspacePaths, item_name: &str) -> Result<()> {
    let candidates = fuzzy_pdf_candidate_paths(paths, item_name);

    let chosen = candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone());

    if !chosen.exists() {
        bail!("PDF not found: {}", chosen.display());
    }

    let chosen_str = chosen.to_string_lossy();
    if let Ok(custom) = std::env::var("ZETTELTEX_PDF_OPENER") {
        let trimmed = custom.trim();
        if !trimmed.is_empty()
            && run_external_open_nonblocking_verified(trimmed, &[chosen_str.as_ref()], None).is_ok()
        {
            return Ok(());
        }
    }

    let direct_openers = [
        "qpdfview",
        "zathura",
        "okular",
        "evince",
        "atril",
        "mupdf",
        "/usr/bin/qpdfview",
        "/usr/bin/zathura",
        "/usr/bin/okular",
        "/usr/bin/evince",
        "/usr/bin/atril",
        "/usr/bin/mupdf",
    ];
    for opener in direct_openers {
        if run_external_open_nonblocking_verified(opener, &[chosen_str.as_ref()], None).is_ok() {
            return Ok(());
        }
    }

    if run_external_open_nonblocking_verified("xdg-open", &[chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }
    if run_external_open_nonblocking_verified("/usr/bin/xdg-open", &[chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }
    if run_external_open_nonblocking_verified("gio", &["open", chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }
    if run_external_open_nonblocking_verified("/usr/bin/gio", &["open", chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }

    bail!(
        "No se pudo abrir el PDF con ningun visor candidato (custom/directo/xdg-open/gio): {}",
        chosen.display()
    )
}

fn normalize_new_note_name(raw: &str) -> Result<String> {
    let mut name = raw.trim().to_string();
    if name.to_lowercase().ends_with(".tex") {
        name.truncate(name.len() - 4);
    }
    name = name.replace(':', "-").replace(' ', "-");
    if name.is_empty() {
        bail!("No se puede crear una nota sin nombre")
    }
    Ok(name)
}

fn note_name_from_clipboard_label(content: &str) -> Result<String> {
    let re = Regex::new(r"\\label\{([^}]+)\}")?;
    let caps = re
        .captures(content)
        .ok_or_else(|| anyhow::anyhow!("No se encontro ninguna etiqueta \\label{{...}} en el portapapeles"))?;
    let mut label = caps
        .get(1)
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();
    if let Some(rest) = label.strip_prefix("defn:") {
        label = rest.to_string();
    }
    label = label.replace(':', "-");
    if label.is_empty() {
        bail!("Etiqueta de portapapeles invalida")
    }
    Ok(label)
}

fn inject_clipboard_into_note_template(note_path: &Path, clipboard_content: &str) -> Result<()> {
    let original = fs::read_to_string(note_path)?;
    let indented = clipboard_content
        .lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let marker = "        %Write Note here";
    let updated = if original.contains(marker) {
        original.replace(marker, &indented)
    } else if let Some(pos) = original.find("\\end{document}") {
        format!("{}\n{}\n{}", &original[..pos], indented, &original[pos..])
    } else {
        format!("{}\n{}\n", original, indented)
    };

    fs::write(note_path, updated)?;
    Ok(())
}

fn fuzzy_state_path(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    config
        .fuzzy
        .state_file
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join(".fuzzy_state.json"))
}

fn fuzzy_legacy_history_path(paths: &WorkspacePaths) -> PathBuf {
    paths.root.join(".fuzzy_history")
}

fn fuzzy_legacy_search_history_json_path(paths: &WorkspacePaths) -> PathBuf {
    paths.root.join(".search_history.json")
}

fn fuzzy_legacy_popularity_tsv_path(paths: &WorkspacePaths) -> PathBuf {
    paths.root.join(".fuzzy_popularity_cache.tsv")
}

fn fuzzy_legacy_popularity_json_path(paths: &WorkspacePaths) -> PathBuf {
    paths.root.join(".fuzzy_popularity_cache.json")
}

fn load_fuzzy_history(paths: &WorkspacePaths, items: &[FuzzyItem]) -> Result<Vec<String>> {
    let state = read_or_migrate_fuzzy_state(paths)?;
    let available = items.iter().map(|i| i.display.clone()).collect::<std::collections::HashSet<_>>();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for entry in state.history {
        let trimmed = entry.trim();
        if !trimmed.is_empty() && available.contains(trimmed) && seen.insert(trimmed.to_string()) {
            out.push(trimmed.to_string());
        }
    }

    Ok(out)
}

fn save_history_entry(paths: &WorkspacePaths, item_display: &str) -> Result<()> {
    let mut state = read_or_migrate_fuzzy_state(paths)?;
    state.history.retain(|e| e != item_display);
    state.history.insert(0, item_display.to_string());
    state.history.truncate(FUZZY_HISTORY_LIMIT);
    write_fuzzy_state_file(&fuzzy_state_path(paths), &state)
}

fn read_or_migrate_fuzzy_state(paths: &WorkspacePaths) -> Result<FuzzyStateFile> {
    let state_path = fuzzy_state_path(paths);
    if state_path.exists() {
        let mut state = read_fuzzy_state_file(&state_path)?;
        if state.history.len() > FUZZY_HISTORY_LIMIT {
            state.history.truncate(FUZZY_HISTORY_LIMIT);
        }
        cleanup_legacy_fuzzy_files(paths);
        return Ok(state);
    }

    let mut state = FuzzyStateFile::default();

    let legacy_history = fuzzy_legacy_history_path(paths);
    if legacy_history.exists() {
        let content = fs::read_to_string(&legacy_history)?;
        for line in content.lines() {
            let entry = line.trim();
            if !entry.is_empty() {
                state.history.push(entry.to_string());
            }
        }
    } else {
        let legacy_search_json = fuzzy_legacy_search_history_json_path(paths);
        if legacy_search_json.exists() {
            let content = fs::read_to_string(&legacy_search_json)?;
            state.history = parse_legacy_history_json(&content)?;
        }
    }

    if state.history.len() > FUZZY_HISTORY_LIMIT {
        state.history.truncate(FUZZY_HISTORY_LIMIT);
    }

    let legacy_pop_tsv = fuzzy_legacy_popularity_tsv_path(paths);
    if legacy_pop_tsv.exists() {
        state.popularity_cache = parse_popularity_cache_tsv_file(&legacy_pop_tsv)?;
    }

    if !state.history.is_empty() || !state.popularity_cache.is_empty() {
        write_fuzzy_state_file(&state_path, &state)?;
    }

    cleanup_legacy_fuzzy_files(paths);
    Ok(state)
}

fn read_fuzzy_state_file(path: &Path) -> Result<FuzzyStateFile> {
    let content = fs::read_to_string(path)?;
    match serde_json::from_str::<FuzzyStateFile>(&content) {
        Ok(state) => Ok(state),
        Err(err) => {
            warn!("No se pudo parsear estado fuzzy en {}: {err}", path.display());
            Ok(FuzzyStateFile::default())
        }
    }
}

fn write_fuzzy_state_file(path: &Path, state: &FuzzyStateFile) -> Result<()> {
    let serialized = serde_json::to_string_pretty(state)?;
    fs::write(path, serialized + "\n")?;
    Ok(())
}

fn parse_legacy_history_json(content: &str) -> Result<Vec<String>> {
    let trimmed = content.trim();
    if !trimmed.starts_with('[') {
        return Ok(Vec::new());
    }

    let re = Regex::new(r#"\"([^\"]+)\""#)?;
    let mut out = Vec::new();
    for caps in re.captures_iter(trimmed) {
        if let Some(m) = caps.get(1) {
            let entry = m.as_str().trim();
            if !entry.is_empty() {
                out.push(entry.to_string());
            }
        }
    }
    Ok(out)
}

fn cleanup_legacy_fuzzy_files(paths: &WorkspacePaths) {
    let legacy_files = [
        fuzzy_legacy_history_path(paths),
        fuzzy_legacy_search_history_json_path(paths),
        fuzzy_legacy_popularity_tsv_path(paths),
        fuzzy_legacy_popularity_json_path(paths),
    ];

    for file in legacy_files {
        let _ = fs::remove_file(file);
    }
}

fn write_xclip_clipboard(text: &str) -> Result<()> {
    fn try_clipboard_write(bin: &str, args: &[&str], text: &str) -> Result<bool> {
        let mut cmd = Command::new(bin);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(_) => return Ok(false),
        };

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(err) = stdin.write_all(text.as_bytes()) {
                // Algunos binarios/fakes de clipboard cierran stdin de inmediato;
                // si el proceso finaliza con exito, tratamos EPIPE como no fatal.
                if err.kind() != std::io::ErrorKind::BrokenPipe {
                    return Err(err.into());
                }
            }
            drop(stdin);
        } else {
            return Ok(false);
        }

        // Verificacion corta: si falla de inmediato devolvemos false para activar fallback.
        // wl-copy suele salir rapido tras recibir stdin; xclip/xsel pueden quedarse vivos
        // para mantener ownership del clipboard.
        let timeout = Duration::from_millis(150);
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(status.success());
            }
            if start.elapsed() >= timeout {
                // xclip/xsel vivos tras timeout suele ser exito; wl-copy colgado no.
                if bin == "wl-copy" {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(false);
                }
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    // Comprobamos disponibilidad antes de spawnear para evitar tirones de D-Bus en binarios faltantes pero registrados.
    if command_exists("wl-copy") && std::env::var("WAYLAND_DISPLAY").is_ok() {
        if try_clipboard_write("wl-copy", &[], text)? {
            return Ok(());
        }
    }
    if command_exists("xclip") && try_clipboard_write("xclip", &["-selection", "clipboard"], text)? {
        return Ok(());
    }
    if command_exists("xsel") && try_clipboard_write("xsel", &["--clipboard", "--input"], text)? {
        return Ok(());
    }

    bail!("No se pudo copiar al portapapeles (wl-copy/xclip/xsel)")
}

fn read_xclip_clipboard() -> Result<String> {
    let readers: [(&str, &[&str]); 3] = [
        ("wl-paste", &["--no-newline"]),
        ("xclip", &["-selection", "clipboard", "-o"]),
        ("xsel", &["--clipboard", "--output"]),
    ];

    for (bin, args) in readers {
        if !command_exists(bin) { continue; }
        if bin == "wl-paste" && std::env::var("WAYLAND_DISPLAY").is_err() { continue; }
        
        let output = match Command::new(bin).args(args).output() {
            Ok(out) => out,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => continue,
        };
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }

    bail!("Error leyendo portapapeles (wl-paste/xclip/xsel)")
}

fn build_exhyperref_for_item(paths: &WorkspacePaths, index: &FuzzyIndex, item: &FuzzyItem) -> Result<String> {
    if item.kind == FuzzyItemKind::Project {
        return Ok(item.name.clone());
    }
    let label = best_label_for_note(paths, index, &item.name)
        .unwrap_or_else(|| format!("defn:{}", item.name));
    Ok(format!(r"\exhyperref[{}]{{{}}}", label, item.name))
}

fn build_excref_for_item(paths: &WorkspacePaths, index: &FuzzyIndex, item: &FuzzyItem) -> Result<String> {
    if item.kind == FuzzyItemKind::Project {
        return Ok(item.name.clone());
    }
    let label = best_label_for_note(paths, index, &item.name)
        .unwrap_or_else(|| format!("defn:{}", item.name));
    Ok(format!(r"\excref[{}]{{{}}}", label, item.name))
}

fn best_label_for_note(paths: &WorkspacePaths, index: &FuzzyIndex, note_name: &str) -> Option<String> {
    let mut labels = Vec::new();
    if let Some(content) = index.note_content_original.get(note_name) {
        let re = Regex::new(r"\\label\{([^}]+)\}").ok()?;
        labels.extend(
            re.captures_iter(content)
                .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string())),
        );
    }

    // Fallback a DB para paridad con fuzzy.py cuando faltan labels en el archivo indexado.
    if let Ok(db) = init_database(&paths.root.join("slipbox.db")) {
        if let Ok(db_labels) = db.labels_for_note(note_name) {
            labels.extend(db_labels);
        }
    }

    let mut seen = std::collections::HashSet::new();
    labels.retain(|l| !l.trim().is_empty() && seen.insert(l.clone()));

    if labels.is_empty() {
        return None;
    }

    if let Some(exact) = labels
        .iter()
        .find(|label| label.to_lowercase().contains(&note_name.to_lowercase()))
    {
        return Some(exact.clone());
    }

    labels
        .into_iter()
        .max_by(|a, b| {
            normalized_levenshtein(a, note_name)
                .partial_cmp(&normalized_levenshtein(b, note_name))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn preview_lines_for_item(index: &FuzzyIndex, item: &FuzzyItem, max_lines: usize) -> Vec<String> {
    if item.kind == FuzzyItemKind::Project {
        if let Some(lines) = index.project_preview.get(&item.name) {
            return lines.iter().take(max_lines).cloned().collect();
        }
        return vec![format!("Proyecto: {}", item.name)];
    }

    let content = index
        .note_content_original
        .get(&item.name)
        .cloned()
        .unwrap_or_default();
    content
        .lines()
        .take(max_lines)
        .map(|line| line.to_string())
        .collect()
}

fn launch_fuzzy_in_new_terminal(paths: &WorkspacePaths) -> Result<()> {
    let exe = std::env::current_exe()?;
    let exe_arg = exe.to_string_lossy().to_string();
    let root_arg = paths.root.to_string_lossy().to_string();

    let launchers = terminal_launchers(&exe_arg, &root_arg);

    for launcher in launchers {
        if !command_exists(&launcher.program) {
            continue;
        }

        // Importante: abrir en background y no depender del codigo de salida del emulador.
        // Esto evita que el proceso padre falle al cerrar la ventana fuzzy.
        let spawned = Command::new(&launcher.program)
            .args(launcher.args)
            .spawn();

        match spawned {
            Ok(_) => return Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err.into()),
        }
    }

    bail!(
        "No se pudo abrir una terminal nueva. Instala/configura uno de: x-terminal-emulator, gnome-terminal, konsole, kitty, alacritty"
    )
}

struct TerminalLauncher {
    program: String,
    args: Vec<String>,
}

fn terminal_launchers(exe_arg: &str, root_arg: &str) -> Vec<TerminalLauncher> {
    vec![
        TerminalLauncher {
            program: "alacritty".to_string(),
            args: vec![
                "-e".to_string(),
                exe_arg.to_string(),
                "--workspace-root".to_string(),
                root_arg.to_string(),
                "fuzzy".to_string(),
                "--inline".to_string(),
            ],
        },
        TerminalLauncher {
            program: "x-terminal-emulator".to_string(),
            args: vec![
                "-e".to_string(),
                exe_arg.to_string(),
                "--workspace-root".to_string(),
                root_arg.to_string(),
                "fuzzy".to_string(),
                "--inline".to_string(),
            ],
        },
        TerminalLauncher {
            program: "gnome-terminal".to_string(),
            args: vec![
                "--".to_string(),
                exe_arg.to_string(),
                "--workspace-root".to_string(),
                root_arg.to_string(),
                "fuzzy".to_string(),
                "--inline".to_string(),
            ],
        },
        TerminalLauncher {
            program: "konsole".to_string(),
            args: vec![
                "-e".to_string(),
                exe_arg.to_string(),
                "--workspace-root".to_string(),
                root_arg.to_string(),
                "fuzzy".to_string(),
                "--inline".to_string(),
            ],
        },
        TerminalLauncher {
            program: "kitty".to_string(),
            args: vec![
                "-e".to_string(),
                exe_arg.to_string(),
                "--workspace-root".to_string(),
                root_arg.to_string(),
                "fuzzy".to_string(),
                "--inline".to_string(),
            ],
        },
    ]
}

fn command_exists(program: &str) -> bool {
    if program.contains('/') {
        return Path::new(program).is_file();
    }

    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };

    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return true;
        }
    }

    false
}

fn build_fuzzy_index(paths: &WorkspacePaths) -> Result<FuzzyIndex> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let settings = load_fuzzy_settings(paths);
    let mut notes = db.list_notes()?;
    let mut projects = db.list_projects()?;

    // Si la DB aun no fue poblada, intentar sincronizar para reflejar el filesystem.
    if notes.is_empty() && projects.is_empty() {
        let _ = synchronize_notes(paths)?;
        let _ = synchronize_projects(paths)?;
        notes = db.list_notes()?;
        projects = db.list_projects()?;
    }

    let popularity = load_or_compute_popularity_cache(paths, &db)?;

    let mut index = FuzzyIndex {
        settings: settings.clone(),
        ..FuzzyIndex::default()
    };

    for note in notes {
        let note_name = note.filename;
        index.items.push(FuzzyItem {
            display: note_name.clone(),
            name: note_name.clone(),
            name_lower: note_name.to_lowercase(),
            kind: FuzzyItemKind::Note,
        });

        let note_path = paths.notes_slipbox.join(format!("{}.tex", note_name));
        let content = fs::read_to_string(note_path).unwrap_or_default();
        index
            .note_content_original
            .insert(note_name.clone(), content.clone());
        index
            .note_content_lower
            .insert(note_name, content.to_lowercase());
    }

    for project in projects {
        let project_name = project.name;
        index.items.push(FuzzyItem {
            display: format!("[PROJECT] {}", project_name),
            name: project_name.clone(),
            name_lower: project_name.to_lowercase(),
            kind: FuzzyItemKind::Project,
        });

        let mut preview = Vec::new();
        if let Some(meta) = db.project_metadata_by_name(&project_name)? {
            preview.push(format!("Proyecto: {}", meta.name));
            preview.push(String::new());
            preview.push(format!("Archivo principal: {}", meta.filename));
            preview.push(format!(
                "Creado: {}",
                meta.created.unwrap_or_else(|| "N/A".to_string())
            ));
            preview.push(format!(
                "Ultima edicion: {}",
                meta.last_edit_date.unwrap_or_else(|| "N/A".to_string())
            ));
            preview.push(format!(
                "Ultima compilacion PDF: {}",
                meta.last_build_date_pdf.unwrap_or_else(|| "N/A".to_string())
            ));
        } else {
            preview.push(format!("Proyecto: {}", project_name));
            preview.push(String::new());
            preview.push(format!("Archivo principal: {}.tex", project_name));
        }

        preview.push(String::new());
        preview.push("Notas incluidas:".to_string());
        let inclusions = db.list_project_inclusions_by_name(&project_name)?;
        if inclusions.is_empty() {
            preview.push("  (sin inclusiones)".to_string());
        } else {
            for inc in inclusions {
                let mut line = format!("  - {}", inc.note_filename);
                if !inc.tag.trim().is_empty() {
                    line.push_str(&format!(" [{}]", inc.tag));
                }
                line.push_str(&format!(" (desde {})", inc.source_file));
                preview.push(line);
            }
        }
        index.project_preview.insert(project_name.clone(), preview);
    }

    for p in popularity {
        let in_refs = p.in_refs as f64;
        let out_refs = p.out_refs as f64;
        let total = in_refs * settings.in_refs_weight + out_refs * settings.out_refs_weight;
        index.note_popularity.insert(
            p.filename,
            NotePopularity {
                in_refs,
                out_refs,
                total,
            },
        );
    }

    Ok(index)
}

fn load_fuzzy_settings(paths: &WorkspacePaths) -> FuzzySettings {
    let mut settings = FuzzySettings {
        max_results: FUZZY_MAX_RESULTS_DEFAULT,
        in_refs_weight: FUZZY_IN_REFS_WEIGHT_DEFAULT,
        out_refs_weight: FUZZY_OUT_REFS_WEIGHT_DEFAULT,
        accent_color: FUZZY_ACCENT_COLOR_DEFAULT,
    };

    fn parse_hex_color(s: &str) -> Option<Color> {
        let hex = s.trim();
        let hex = hex.strip_prefix('#')?;
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    }

    let config = load_zetteltex_config(paths);

    if let Some(v) = config.fuzzy.max_results {
        if v > 0 {
            settings.max_results = v;
        }
    }
    if let Some(v) = config.fuzzy.in_refs_weight {
        settings.in_refs_weight = v;
    }
    if let Some(v) = config.fuzzy.out_refs_weight {
        settings.out_refs_weight = v;
    }
    if let Some(raw) = config.fuzzy.selection_color.as_deref() {
        if let Some(color) = parse_hex_color(raw) {
            settings.accent_color = color;
        }
    }

    settings
}

fn load_zetteltex_config(paths: &WorkspacePaths) -> ZetteltexConfig {
    let config_path = paths.root.join("zetteltex.toml");
    let Ok(content) = fs::read_to_string(config_path) else {
        return ZetteltexConfig::default();
    };

    match toml::from_str::<ZetteltexConfig>(&content) {
        Ok(config) => config,
        Err(err) => {
            warn!("No se pudo parsear zetteltex.toml: {err}");
            ZetteltexConfig::default()
        }
    }
}

fn resolve_config_path(root: &Path, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw.trim());
    if candidate.is_absolute() {
        candidate
    } else {
        root.join(candidate)
    }
}

fn load_or_compute_popularity_cache(
    paths: &WorkspacePaths,
    db: &zetteltex_db::Database,
) -> Result<Vec<zetteltex_db::NotePopularityRecord>> {
    let state_path = fuzzy_state_path(paths);
    let db_path = paths.root.join("slipbox.db");
    let mut state = read_or_migrate_fuzzy_state(paths)?;

    if db_path.exists() {
        let db_mtime_ms = fs::metadata(&db_path)?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if state.db_mtime_unix_ms.unwrap_or(0) >= db_mtime_ms && !state.popularity_cache.is_empty() {
            let rows = state
                .popularity_cache
                .iter()
                .map(|row| zetteltex_db::NotePopularityRecord {
                    filename: row.filename.clone(),
                    in_refs: row.in_refs,
                    out_refs: row.out_refs,
                })
                .collect::<Vec<_>>();
            return Ok(rows);
        }

        let computed = db.note_popularity_stats()?;
        state.popularity_cache = computed
            .iter()
            .map(|row| FuzzyPopularityRow {
                filename: row.filename.clone(),
                in_refs: row.in_refs,
                out_refs: row.out_refs,
            })
            .collect();
        state.db_mtime_unix_ms = Some(db_mtime_ms);
        write_fuzzy_state_file(&state_path, &state)?;
        return Ok(computed);
    }

    let computed = db.note_popularity_stats()?;
    state.popularity_cache = computed
        .iter()
        .map(|row| FuzzyPopularityRow {
            filename: row.filename.clone(),
            in_refs: row.in_refs,
            out_refs: row.out_refs,
        })
        .collect();
    state.db_mtime_unix_ms = None;
    write_fuzzy_state_file(&state_path, &state)?;
    Ok(computed)
}

fn parse_popularity_cache_tsv_file(path: &Path) -> Result<Vec<FuzzyPopularityRow>> {
    let content = fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 3 {
            continue;
        }
        let Ok(in_refs) = parts[1].parse::<i64>() else {
            continue;
        };
        let Ok(out_refs) = parts[2].parse::<i64>() else {
            continue;
        };
        out.push(FuzzyPopularityRow {
            filename: parts[0].to_string(),
            in_refs,
            out_refs,
        });
    }
    Ok(out)
}

fn fuzzy_search<'a>(index: &'a FuzzyIndex, query: &str, max_results: usize) -> Vec<(&'a FuzzyItem, f64)> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let max_popularity = index
        .note_popularity
        .values()
        .map(|p| p.total)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let mut scored = Vec::new();

    for item in &index.items {
        let mut score = 0.0_f64;

        if item.name_lower.contains(&q) {
            score += 100.0;
        } else if q.contains(&item.name_lower) {
            score += 80.0;
        }

        let name_ratio = normalized_levenshtein(&q, &item.name_lower);
        score += name_ratio * 50.0;

        if item.kind == FuzzyItemKind::Note {
            if let Some(content) = index.note_content_lower.get(&item.name) {
                if content.contains(&q) {
                    let occurrences = content.matches(&q).count() as f64;
                    score += (occurrences * 5.0).min(40.0);

                    if let Some(first_pos) = content.find(&q) {
                        if first_pos < 500 {
                            score += 20.0;
                        }
                    }
                }
            }

            if let Some(pop) = index.note_popularity.get(&item.name) {
                let _ = pop.in_refs + pop.out_refs;
                let popularity_points = (pop.total / max_popularity) * 40.0;
                score += popularity_points;
            }
        }

        if score > 0.0 {
            scored.push((item, score));
        }
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max_results);
    scored
}

fn extract_tagged_block(content: &str, tag: &str) -> Result<Option<String>> {
    let pat = Regex::new(&format!(
        r"(?s)%<\*{}>(.*?)%</{}>",
        regex::escape(tag),
        regex::escape(tag)
    ))?;
    Ok(pat
        .captures(content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string())))
}

fn resolve_workspace_path(paths: &WorkspacePaths, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        paths.root.join(p)
    }
}

fn title_from_name(name: &str) -> String {
    name.split('_')
        .filter(|s| !s.is_empty())
        .map(capitalize_first)
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_first(token: &str) -> String {
    let mut chars = token.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut out = String::new();
    out.extend(first.to_uppercase());
    out.push_str(chars.as_str());
    out
}

fn replace_title(template: &str, new_title: &str) -> String {
    let token = "\\title{";
    let Some(start) = template.find(token) else {
        return template.to_string();
    };

    let content_start = start + token.len();
    let Some(relative_end) = template[content_start..].find('}') else {
        return template.to_string();
    };
    let end = content_start + relative_end;

    let mut out = String::with_capacity(template.len() + new_title.len());
    out.push_str(&template[..content_start]);
    out.push_str(new_title);
    out.push_str(&template[end..]);
    out
}

fn rename_file(paths: &WorkspacePaths, old_name: &str, new_name: &str) -> Result<()> {
    rename_file_impl(paths, old_name, new_name, true)
}

fn rename_file_impl(
    paths: &WorkspacePaths,
    old_name: &str,
    new_name: &str,
    emit_logs: bool,
) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let old_path = paths.notes_slipbox.join(format!("{old_name}.tex"));
    let new_path = paths.notes_slipbox.join(format!("{new_name}.tex"));

    if new_path.exists() {
        bail!("File {new_name}.tex already exists");
    }
    if !db.note_exists(old_name)? {
        bail!("Note {old_name} not found in database");
    }
    if !old_path.exists() {
        bail!("Note file {} does not exist", old_path.display());
    }

    fs::rename(&old_path, &new_path)?;
    db.rename_note_filename(old_name, new_name)?;
    update_documents_externaldocument(&paths.root.join("notes/documents.tex"), old_name, new_name)?;

    replace_references_in_folder(&paths.notes_slipbox, old_name, new_name)?;
    replace_references_in_folder(&paths.projects, old_name, new_name)?;
    cleanup_renamed_note_exports(paths, old_name)?;

    if emit_logs {
        println!("Renaming {old_name} -> {new_name}");
        println!("Successfully renamed {old_name} to {new_name}");
    }
    Ok(())
}

fn rename_label(
    paths: &WorkspacePaths,
    note_name: &str,
    old_label: &str,
    new_label: &str,
) -> Result<()> {
    rename_label_impl(paths, note_name, old_label, new_label, true)
}

fn rename_label_impl(
    paths: &WorkspacePaths,
    note_name: &str,
    old_label: &str,
    new_label: &str,
    emit_logs: bool,
) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!("Note {note_name} not found in database");
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let original = fs::read_to_string(&note_path)?;
    
    // Replace the label definition
    let own_label_pat = Regex::new(&format!(r"\\label\{{{}\}}", regex::escape(old_label)))?;
    let mut updated_note = own_label_pat
        .replace_all(&original, format!(r"\label{{{new_label}}}"))
        .to_string();
    
    // Replace internal references (without note prefix) in the same note
    let internal_patterns = vec![
        // \ref{old_label} -> \ref{new_label}
        (
            Regex::new(&format!(r"\\ref\{{{}\}}", regex::escape(old_label)))?,
            format!(r"\ref{{{new_label}}}"),
        ),
        // \hyperref[old_label] -> \hyperref[new_label]
        (
            Regex::new(&format!(r"\\hyperref\[{}\]", regex::escape(old_label)))?,
            format!(r"\hyperref[{new_label}]"),
        ),
        // \excref[old_label]{note_name} -> \excref[new_label]{note_name}
        (
            Regex::new(&format!(
                r"\\excref\[{}\]\{{{}\}}",
                regex::escape(old_label),
                regex::escape(note_name)
            ))?,
            format!(r"\excref[{new_label}]{{{note_name}}}"),
        ),
        // \exhyperref[old_label]{note_name}{...} -> \exhyperref[new_label]{note_name}{...}
        (
            Regex::new(&format!(
                r"\\exhyperref\[{}\]\{{{}\}}\{{([^}}]+)\}}",
                regex::escape(old_label),
                regex::escape(note_name)
            ))?,
            format!(r"\exhyperref[{new_label}]{{{note_name}}}{{$1}}"),
        ),
    ];
    
    for (re, replacement) in internal_patterns {
        updated_note = re.replace_all(&updated_note, replacement.as_str()).to_string();
    }
    
    fs::write(&note_path, updated_note)?;

    replace_label_references_in_folder(&paths.notes_slipbox, note_name, old_label, new_label)?;
    replace_label_references_in_folder(&paths.projects, note_name, old_label, new_label)?;

    let _ = synchronize_notes(paths)?;

    if emit_logs {
        println!("Successfully renamed label {old_label} to {new_label} in {note_name}");
    }
    Ok(())
}

fn rename_interactive(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let _ = synchronize_notes(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!("Note {note_name} not found in database");
    }

    let all_labels = db.labels_for_note(note_name)?;
    let labels: Vec<String> = all_labels
        .iter()
        .filter(|label| label.as_str() != "note")
        .cloned()
        .collect();

    let mut current_note_name = note_name.to_string();
    let chosen_note_name = prompt_user("Change note name to", note_name)?;
    if chosen_note_name != note_name {
        rename_file_impl(paths, note_name, &chosen_note_name, false)?;
        current_note_name = chosen_note_name;
    }

    let mut label_changes: Vec<(String, String)> = Vec::new();
    for (idx, old_label) in labels.iter().enumerate() {
        let prompt = format!("Change label #{} ({}) to", idx + 1, old_label);
        let chosen = prompt_user(&prompt, old_label)?;
        if chosen != *old_label {
            if chosen == "note" {
                bail!("Label name 'note' is reserved and cannot be assigned");
            }
            label_changes.push((old_label.clone(), chosen));
        }
    }

    if label_changes.is_empty() {
        if current_note_name == note_name {
            println!("No changes made");
        }
        return Ok(());
    }

    // Validate resulting labels to avoid duplicate final names.
    let mut final_labels = Vec::new();
    for old in &all_labels {
        let mapped = label_changes
            .iter()
            .find(|(from, _)| from == old)
            .map(|(_, to)| to.clone())
            .unwrap_or_else(|| old.clone());
        final_labels.push(mapped);
    }
    let mut seen = HashSet::new();
    for label in final_labels {
        if !seen.insert(label.clone()) {
            bail!("Duplicate target label after rename: {label}");
        }
    }

    // Two-phase rename through temporary labels so cycles/collisions are safe.
    let mut reserved: HashSet<String> = all_labels.iter().cloned().collect();
    let mut staged: Vec<(String, String, String)> = Vec::new();
    for (idx, (old_label, new_label)) in label_changes.iter().enumerate() {
        let mut candidate = format!("__ztx_tmp_rename_{}__", idx + 1);
        let mut salt = 1usize;
        while reserved.contains(&candidate) {
            salt += 1;
            candidate = format!("__ztx_tmp_rename_{}_{}__", idx + 1, salt);
        }
        reserved.insert(candidate.clone());
        staged.push((old_label.clone(), candidate, new_label.clone()));
    }

    for (old_label, temp_label, _) in &staged {
        rename_label_impl(paths, &current_note_name, old_label, temp_label, false)?;
    }
    for (_, temp_label, new_label) in &staged {
        rename_label_impl(paths, &current_note_name, temp_label, new_label, false)?;
    }

    println!("Rename summary:");
    if current_note_name == note_name {
        println!("- note: unchanged ({current_note_name})");
    } else {
        println!("- note: {note_name} -> {current_note_name}");
    }
    if label_changes.is_empty() {
        println!("- labels: no changes");
    } else {
        println!("- labels changed: {}", label_changes.len());
        for (old_label, new_label) in &label_changes {
            println!("  {old_label} -> {new_label}");
        }
    }
    Ok(())
}

fn cleanup_renamed_note_exports(paths: &WorkspacePaths, old_name: &str) -> Result<()> {
    let pdf_candidates = fuzzy_pdf_candidate_paths(paths, old_name);
    for candidate in pdf_candidates {
        if candidate.exists() {
            fs::remove_file(&candidate)?;
        }
    }

    let markdown_path = export_notes_dir(paths).join(format!("{old_name}.md"));
    if markdown_path.exists() {
        fs::remove_file(markdown_path)?;
    }

    Ok(())
}

#[derive(Debug, Default)]
struct CleanStats {
    pdf_removed: usize,
    markdown_removed: usize,
}

fn clean_generated_note_artifacts(paths: &WorkspacePaths) -> Result<CleanStats> {
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let note_names: std::collections::HashSet<String> = db
        .list_notes()?
        .into_iter()
        .map(|note| note.filename)
        .collect();
    let project_names: std::collections::HashSet<String> = db
        .list_projects()?
        .into_iter()
        .map(|project| project.name)
        .collect();
    let mut allowed_names = note_names.clone();
    allowed_names.extend(project_names.iter().cloned());

    let mut stats = CleanStats::default();
    let pdf_dirs = unique_existing_dirs([
        pdf_output_dir(paths),
        paths.root.join("jabberwocky/adjuntos/pdf"),
    ]);
    for dir in pdf_dirs {
        stats.pdf_removed += remove_orphan_files_with_extension(&dir, "pdf", &allowed_names)?;
    }

    let markdown_dirs = unique_existing_dirs([
        export_notes_dir(paths),
        export_projects_dir(paths),
        paths.root.join("markdown"),
    ]);
    for dir in markdown_dirs {
        stats.markdown_removed += remove_orphan_files_with_extension(&dir, "md", &allowed_names)?;
    }

    Ok(stats)
}

fn unique_existing_dirs<const N: usize>(dirs: [PathBuf; N]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        if seen.insert(dir.clone()) {
            out.push(dir);
        }
    }
    out
}

fn remove_orphan_files_with_extension(
    dir: &Path,
    extension: &str,
    note_names: &HashSet<String>,
) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut removed = 0usize;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some(extension) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if note_names.contains(stem) {
            continue;
        }
        fs::remove_file(&path)?;
        removed += 1;
    }

    Ok(removed)
}

fn remove_note(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    if note_path.exists() {
        fs::remove_file(&note_path)?;
    }

    remove_externaldocument_line(&paths.root.join("notes/documents.tex"), note_name)?;
    db.delete_note_by_filename(note_name)?;

    println!("Removed note {note_name}");
    Ok(())
}

fn update_documents_externaldocument(
    documents_path: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<()> {
    if !documents_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(documents_path)?;
    let pat = Regex::new(&format!(
        r"\\externaldocument\[{}-\]\{{{}\}}",
        regex::escape(old_name),
        regex::escape(old_name)
    ))?;
    let replaced = pat
        .replace_all(
            &content,
            format!(r"\externaldocument[{new_name}-]{{{new_name}}}"),
        )
        .to_string();
    fs::write(documents_path, replaced)?;
    Ok(())
}

fn remove_externaldocument_line(documents_path: &Path, note_name: &str) -> Result<()> {
    if !documents_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(documents_path)?;
    let mut out = Vec::new();
    for line in content.lines() {
        if line.contains(&format!("{{{note_name}}}")) && line.contains("\\externaldocument[") {
            continue;
        }
        out.push(line);
    }
    let mut rebuilt = out.join("\n");
    if !rebuilt.is_empty() {
        rebuilt.push('\n');
    }
    fs::write(documents_path, rebuilt)?;
    Ok(())
}

fn replace_references_in_folder(root: &Path, old_name: &str, new_name: &str) -> Result<()> {
    let patterns = vec![
        (
            Regex::new(&format!(r"\\transclude\{{{}\}}", regex::escape(old_name)))?,
            format!(r"\transclude{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\transclude\[([^\]]+)\]\{{{}\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\transclude[$1]{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exref\[([^\]]+)\]\{{{}\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\exref[$1]{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\excref\[([^\]]+)\]\{{{}\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\excref[$1]{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exhyperref\[([^\]]+)\]\{{{}\}}\{{([^}}]+)\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\exhyperref[$1]{{{new_name}}}{{$2}}"),
        ),
        (
            Regex::new(&format!(r"\\ref\{{{}-([^}}]+)\}}", regex::escape(old_name)))?,
            format!(r"\ref{{{new_name}-$1}}"),
        ),
        (
            Regex::new(&format!(r"\\hyperref\[{}-", regex::escape(old_name)))?,
            format!(r"\hyperref[{new_name}-"),
        ),
    ];

    rewrite_tex_files_recursive(root, &patterns)
}

fn replace_label_references_in_folder(
    root: &Path,
    note_name: &str,
    old_label: &str,
    new_label: &str,
) -> Result<()> {
    let full_old = format!("{note_name}-{old_label}");
    let full_new = format!("{note_name}-{new_label}");

    let patterns = vec![
        (
            Regex::new(&format!(r"\\ref\{{{}\}}", regex::escape(&full_old)))?,
            format!(r"\ref{{{full_new}}}"),
        ),
        (
            Regex::new(&format!(r"\\hyperref\[{}\]", regex::escape(&full_old)))?,
            format!(r"\hyperref[{full_new}]"),
        ),
        (
            Regex::new(&format!(
                r"\\excref\[{}\]\{{{}\}}",
                regex::escape(old_label),
                regex::escape(note_name)
            ))?,
            format!(r"\excref[{new_label}]{{{note_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exhyperref\[{}\]\{{{}\}}\{{([^}}]+)\}}",
                regex::escape(old_label),
                regex::escape(note_name)
            ))?,
            format!(r"\exhyperref[{new_label}]{{{note_name}}}{{$1}}"),
        ),
    ];

    rewrite_tex_files_recursive(root, &patterns)
}

fn rewrite_tex_files_recursive(root: &Path, patterns: &[(Regex, String)]) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            rewrite_tex_files_recursive(&path, patterns)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("tex") {
            continue;
        }

        let original = fs::read_to_string(&path)?;
        let mut updated = original.clone();
        for (re, replacement) in patterns {
            updated = re.replace_all(&updated, replacement.as_str()).to_string();
        }

        if updated != original {
            fs::write(&path, updated)?;
        }
    }
    Ok(())
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

fn init_config_interactive(paths: &WorkspacePaths) -> anyhow::Result<std::process::ExitCode> {
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
# Color de acento de la interfaz (en ANSI, por ejemplo 'blue', 'green', 'magenta')
selection_color = "{}"
"#, pdf_output_dir, obsidian_vault, notes_subdir, projects_subdir, max_results, selection_color);

    std::fs::write(&config_path, config_content)?;
    println!("\n¡Archivo de configuración guardado exitosamente en {}!", config_path.display());
    
    Ok(std::process::ExitCode::SUCCESS)
}
