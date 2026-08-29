use std::collections::HashMap;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzyItemKind {
    Note,
    Project,
}

#[derive(Debug, Clone)]
pub struct FuzzyItem {
    pub display: String,
    pub name: String,
    pub name_lower: String,
    pub kind: FuzzyItemKind,
}

#[derive(Debug, Clone)]
pub struct NotePopularity {
    pub in_refs: f64,
    pub out_refs: f64,
    pub total: f64,
}

#[derive(Debug, Default)]
pub struct FuzzyIndex {
    pub items: Vec<FuzzyItem>,
    pub note_content_lower: HashMap<String, String>,
    pub note_content_original: HashMap<String, String>,
    pub note_popularity: HashMap<String, NotePopularity>,
    pub project_preview: HashMap<String, Vec<String>>,
    pub settings: FuzzySettings,
}

#[derive(Debug, Clone)]
pub struct FuzzySettings {
    pub max_results: usize,
    pub history_results: usize,
    pub in_refs_weight: f64,
    pub out_refs_weight: f64,
    pub accent_color: Color,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ZetteltexConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub render: RenderConfig,
    #[serde(default)]
    pub export: ExportConfig,
    #[serde(default)]
    pub fuzzy: FuzzyConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GeneralConfig {
    /// Idioma de la interfaz: `en` (por defecto) o `es`.
    pub lang: Option<String>,
    /// Editor configurado para el comando `edit`.
    pub editor: Option<String>,
}

impl ZetteltexConfig {
    pub fn lang(&self) -> zetteltex_core::i18n::Lang {
        self.general
            .lang
            .as_deref()
            .map(zetteltex_core::i18n::Lang::parse)
            .unwrap_or_default()
    }

    /// Devuelve el comando del editor configurado, o `None` si no se configuró.
    pub fn editor_cmd(&self) -> Option<&str> {
        self.general.editor.as_deref().filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RenderConfig {
    pub pdf_output_dir: Option<String>,
    pub html_output_dir: Option<String>,
    /// Habilitar `-shell-escape` para pdflatex/make4ht. Apagado por defecto:
    /// permite que las notas LaTeX ejecuten comandos de sistema via `\write18`.
    #[serde(default)]
    pub allow_shell_escape: bool,
    /// Tiempo maximo (segundos) para cada invocacion de una herramienta externa
    /// (pdflatex/make4ht/biber). Por defecto 120s; `null` o ausente usa el default.
    #[serde(default)]
    pub render_timeout_secs: Option<u64>,
}

impl RenderConfig {
    /// Tiempo limite por invocacion de herramienta externa, con default de 120s.
    pub fn tool_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.render_timeout_secs.unwrap_or(120))
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExportConfig {
    pub obsidian_vault: Option<String>,
    pub notes_subdir: Option<String>,
    pub projects_subdir: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FuzzyConfig {
    pub max_results: Option<usize>,
    pub history_results: Option<usize>,
    pub in_refs_weight: Option<f64>,
    pub out_refs_weight: Option<f64>,
    pub selection_color: Option<String>,
    pub state_file: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FuzzyStateFile {
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub popularity_cache: Vec<FuzzyPopularityRow>,
    pub db_mtime_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyPopularityRow {
    pub filename: String,
    pub in_refs: i64,
    pub out_refs: i64,
}

impl Default for FuzzySettings {
    fn default() -> Self {
        Self {
            max_results: FUZZY_MAX_RESULTS_DEFAULT,
            history_results: FUZZY_HISTORY_RESULTS_DEFAULT,
            in_refs_weight: FUZZY_IN_REFS_WEIGHT_DEFAULT,
            out_refs_weight: FUZZY_OUT_REFS_WEIGHT_DEFAULT,
            accent_color: FUZZY_ACCENT_COLOR_DEFAULT,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FuzzyUiAction {
    CopyExhyperref { item: FuzzyItem },
    CopyExcref { item: FuzzyItem },
    CopyTransclude { item: FuzzyItem },
    OpenEditor { item: FuzzyItem },
    OpenPdf { item: FuzzyItem },
    CreateFromQuery { query: String },
    CreateFromClipboard,
}

pub const FUZZY_MAX_RESULTS_DEFAULT: usize = 50;
pub const FUZZY_HISTORY_RESULTS_DEFAULT: usize = 10;
pub const FUZZY_IN_REFS_WEIGHT_DEFAULT: f64 = 1.5;
pub const FUZZY_OUT_REFS_WEIGHT_DEFAULT: f64 = 1.0;
pub const FUZZY_HISTORY_LIMIT: usize = 20;
pub const FUZZY_ACCENT_COLOR_DEFAULT: Color = Color::LightMagenta;

use anyhow::{bail, Result};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::{fs, process::Command};
use strsim::normalized_levenshtein;
use tracing::warn;
use zetteltex_core::WorkspacePaths;
use zetteltex_db::init_database;

pub fn build_exhyperref_for_item(
    paths: &WorkspacePaths,
    index: &FuzzyIndex,
    item: &FuzzyItem,
) -> Result<String> {
    if item.kind == FuzzyItemKind::Project {
        return Ok(item.name.clone());
    }
    let label = best_label_for_note(paths, index, &item.name)
        .unwrap_or_else(|| format!("defn:{}", item.name));
    Ok(format!(r"\exhyperref[{}]{{{}}}", label, item.name))
}

pub fn build_excref_for_item(
    paths: &WorkspacePaths,
    index: &FuzzyIndex,
    item: &FuzzyItem,
) -> Result<String> {
    if item.kind == FuzzyItemKind::Project {
        return Ok(item.name.clone());
    }
    let label = best_label_for_note(paths, index, &item.name)
        .unwrap_or_else(|| format!("defn:{}", item.name));
    Ok(format!(r"\excref[{}]{{{}}}", label, item.name))
}

pub fn best_label_for_note(
    paths: &WorkspacePaths,
    index: &FuzzyIndex,
    note_name: &str,
) -> Option<String> {
    let mut labels = Vec::new();
    if let Some(content) = index.note_content_original.get(note_name) {
        let re = Regex::new(r"\\label\{([^}]+)\}").ok()?;
        labels.extend(
            re.captures_iter(content)
                .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string())),
        );
    }

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

    labels.into_iter().max_by(|a, b| {
        normalized_levenshtein(a, note_name)
            .partial_cmp(&normalized_levenshtein(b, note_name))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

pub fn preview_lines_for_item(
    index: &FuzzyIndex,
    item: &FuzzyItem,
    max_lines: usize,
) -> Vec<String> {
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

pub fn launch_fuzzy_in_new_terminal(paths: &WorkspacePaths) -> Result<()> {
    let exe = std::env::current_exe()?;
    let exe_arg = exe.to_string_lossy().to_string();
    let root_arg = paths.root.to_string_lossy().to_string();

    let launchers = terminal_launchers(&exe_arg, &root_arg);

    for launcher in launchers {
        if !command_exists(&launcher.program) {
            continue;
        }

        let spawned = Command::new(&launcher.program).args(launcher.args).spawn();

        match spawned {
            Ok(_) => return Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err.into()),
        }
    }

    bail!(
        "{}",
        crate::i18n::tr(
            "No se pudo abrir una terminal nueva. Instala/configura uno de: x-terminal-emulator, gnome-terminal, konsole, kitty, alacritty",
            "Could not open a new terminal. Install/configure one of: x-terminal-emulator, gnome-terminal, konsole, kitty, alacritty"
        )
    )
}

pub struct TerminalLauncher {
    pub program: String,
    pub args: Vec<String>,
}

pub fn terminal_launchers(exe_arg: &str, root_arg: &str) -> Vec<TerminalLauncher> {
    vec![
        TerminalLauncher {
            program: "alacritty".to_string(),
            args: vec![
                "-e".to_string(),
                exe_arg.to_string(),
                "--workspace-root".to_string(),
                root_arg.to_string(),
                "fuzzy".to_string(),
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
            ],
        },
    ]
}

pub fn command_exists(program: &str) -> bool {
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

pub fn build_fuzzy_index(paths: &WorkspacePaths) -> Result<FuzzyIndex> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let settings = load_fuzzy_settings(paths);
    let mut notes = db.list_notes()?;
    let mut projects = db.list_projects()?;

    if notes.is_empty() && projects.is_empty() {
        let _ = crate::sync::synchronize_notes(paths);
        let _ = crate::sync::synchronize_projects(paths);
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
                meta.last_build_date_pdf
                    .unwrap_or_else(|| "N/A".to_string())
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

pub fn load_fuzzy_settings(paths: &WorkspacePaths) -> FuzzySettings {
    let mut settings = FuzzySettings {
        max_results: FUZZY_MAX_RESULTS_DEFAULT,
        history_results: FUZZY_HISTORY_RESULTS_DEFAULT,
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
    if let Some(v) = config.fuzzy.history_results {
        if v > 0 {
            settings.history_results = v;
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

pub fn load_zetteltex_config(paths: &WorkspacePaths) -> ZetteltexConfig {
    let config_path = paths.root.join("zetteltex.toml");
    let Ok(content) = fs::read_to_string(config_path) else {
        return ZetteltexConfig::default();
    };

    match toml::from_str::<ZetteltexConfig>(&content) {
        Ok(config) => config,
        Err(err) => {
            warn!("No se pudo parsear zetteltex.toml: {}", err);
            ZetteltexConfig::default()
        }
    }
}

pub fn resolve_config_path(root: &Path, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw.trim());
    if candidate.is_absolute() {
        candidate
    } else {
        root.join(candidate)
    }
}

pub fn load_or_compute_popularity_cache(
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

        if state.db_mtime_unix_ms.unwrap_or(0) >= db_mtime_ms && !state.popularity_cache.is_empty()
        {
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

pub fn parse_popularity_cache_tsv_file(path: &Path) -> Result<Vec<FuzzyPopularityRow>> {
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

pub fn fuzzy_search<'a>(
    index: &'a FuzzyIndex,
    query: &str,
    max_results: usize,
) -> Vec<(&'a FuzzyItem, f64)> {
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

pub fn load_fuzzy_history(paths: &WorkspacePaths, items: &[FuzzyItem]) -> Result<Vec<String>> {
    let state = read_or_migrate_fuzzy_state(paths)?;
    let available = items
        .iter()
        .map(|i| i.display.clone())
        .collect::<std::collections::HashSet<_>>();
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

pub fn save_history_entry(paths: &WorkspacePaths, item_display: &str) -> Result<()> {
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
            warn!(
                "No se pudo parsear estado fuzzy en {}: {err}",
                path.display()
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zetteltex_core::WorkspacePaths;

    #[test]
    fn test_preview_lines_for_item_note() {
        let mut index = FuzzyIndex::default();
        index
            .note_content_original
            .insert("note1".to_string(), "Line1\nLine2\nLine3\n".to_string());
        let item = FuzzyItem {
            display: "note1".into(),
            name: "note1".into(),
            name_lower: "note1".into(),
            kind: FuzzyItemKind::Note,
        };
        let lines = preview_lines_for_item(&index, &item, 2);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Line1");
        assert_eq!(lines[1], "Line2");
    }

    #[test]
    fn test_preview_lines_for_item_project() {
        let mut index = FuzzyIndex::default();
        index.project_preview.insert(
            "proj1".to_string(),
            vec!["P1".to_string(), "P2".to_string()],
        );
        let item = FuzzyItem {
            display: "[PROJECT] proj1".into(),
            name: "proj1".into(),
            name_lower: "proj1".into(),
            kind: FuzzyItemKind::Project,
        };
        let lines = preview_lines_for_item(&index, &item, 5);
        assert!(!lines.is_empty());
        assert_eq!(lines[0], "P1");
    }

    #[test]
    fn test_best_label_and_exhyperref() {
        let tmp = tempdir().unwrap();
        let wp = WorkspacePaths {
            root: tmp.path().to_path_buf(),
            notes_slipbox: tmp.path().join("notes/slipbox"),
            projects: tmp.path().join("projects"),
            template: tmp.path().join("template"),
        };
        let mut index = FuzzyIndex::default();
        index.note_content_original.insert(
            "mynote".to_string(),
            "Some content\n\\label{mynote-sec}\n".to_string(),
        );
        let item = FuzzyItem {
            display: "mynote".into(),
            name: "mynote".into(),
            name_lower: "mynote".into(),
            kind: FuzzyItemKind::Note,
        };
        let label = best_label_for_note(&wp, &index, "mynote");
        assert_eq!(label.unwrap(), "mynote-sec");
        let ex = build_exhyperref_for_item(&wp, &index, &item).unwrap();
        assert!(ex.contains("mynote-sec"));
    }

    #[test]
    fn test_parse_popularity_cache_tsv_file() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("pop.tsv");
        let mut f = File::create(&p).unwrap();
        writeln!(f, "note1\t3\t1").unwrap();
        writeln!(f, "note2\t0\t2").unwrap();
        drop(f);
        let rows = parse_popularity_cache_tsv_file(&p).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].filename, "note1");
        assert_eq!(rows[0].in_refs, 3);
    }

    #[test]
    fn test_command_exists_with_path() {
        let tmp = tempdir().unwrap();
        let exe = tmp.path().join("mybin.sh");
        let mut f = File::create(&exe).unwrap();
        writeln!(f, "#!/bin/sh\necho hi").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = f.metadata().unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&exe, perms).unwrap();
        }
        assert!(command_exists(exe.to_str().unwrap()));
        assert!(!command_exists("/nonexistent/foobar"));
    }

    #[test]
    fn test_terminal_launchers_contains_exe() {
        let v = terminal_launchers("myexe", "root");
        assert!(!v.is_empty());
        assert!(v.iter().any(|t| t.args.iter().any(|a| a == "myexe")));
    }
}
