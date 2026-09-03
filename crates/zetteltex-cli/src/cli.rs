use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Pdf,
    Html,
}

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Html => "html",
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "zetteltex")]
#[command(about = "Rust CLI to manage ZettelTeX")]
pub struct Cli {
    /// Workspace root directory.
    #[arg(long, default_value = ".")]
    pub workspace_root: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new note in notes/slipbox and register it in the database.
    #[command(name = "newnote")]
    Newnote {
        /// Note name (without the .tex extension).
        name: String,
    },
    /// Create a minimal workspace structure (notes/slipbox, projects, template) and its templates.
    #[command(name = "init")]
    Init,
    /// Create zetteltex.toml interactively at the workspace root.
    #[command(name = "init_config")]
    InitConfig,
    /// Interactively rename a note (file and associated labels).
    #[command(name = "rename_note")]
    RenameNote {
        /// Name of the note to rename.
        name: String,
    },
    /// Remove a note from the workspace and from supporting indexes.
    #[command(name = "remove_note")]
    RemoveNote {
        /// Name of the note to remove.
        name: String,
    },
    /// List the most recent notes, with configurable limit.
    #[command(name = "list_recent_files")]
    ListRecentFiles {
        /// Number of notes to list (default: configuration value).
        n: Option<usize>,
    },
    /// Show notes with no incoming references or detected usage.
    #[command(name = "list_unreferenced")]
    ListUnreferenced,
    /// Rename the nth most recent note by recency.
    #[command(name = "rename_recent")]
    RenameRecent {
        /// Index (1-based) of the recent note to rename.
        n: Option<usize>,
    },
    /// Add a note to notes/documents.tex for LaTeX cross-references.
    #[command(name = "addtodocuments")]
    AddToDocuments {
        /// Name of the note to add.
        name: String,
    },
    /// List citations detected in a note.
    #[command(name = "list_citations")]
    ListCitations {
        /// Name of the note to inspect.
        name: String,
    },

    /// Create a new project in projects/ with its base .tex file.
    #[command(name = "newproject")]
    Newproject {
        /// Project name (without the .tex extension).
        name: String,
    },
    /// List the projects known to the database.
    #[command(name = "list_projects")]
    ListProjects,
    /// Show the notes included in a project according to transclude.
    #[command(name = "list_project_inclusions")]
    ListProjectInclusions {
        /// Project name.
        project: String,
    },
    /// List the projects where a note appears via inclusion.
    #[command(name = "list_note_projects")]
    ListNoteProjects {
        /// Note name.
        note: String,
    },
    /// List notes and/or projects carrying a given keyword (or any keyword).
    #[command(name = "list_keywords")]
    ListKeywords {
        /// Keyword to filter by (name without trailing colon). If omitted, list everything.
        keyword: Option<String>,
        /// List only notes.
        #[arg(long, short = 'n')]
        notes: bool,
        /// List only projects.
        #[arg(long, short = 'p')]
        projects: bool,
    },
    /// Export a project to an output folder.
    #[command(name = "export_project")]
    ExportProject {
        /// Folder of the project to export.
        folder: String,
        /// Main .tex file inside the project (default: <folder>.tex).
        texfile: Option<String>,
    },
    /// Convert/dump an input file to an output draft.
    #[command(name = "export_draft")]
    ExportDraft {
        /// Input file.
        input_file: String,
        /// Output draft file.
        output_file: String,
    },

    /// Export a note or project to Markdown using the export configuration.
    #[command(name = "export_markdown")]
    ExportMarkdown {
        /// Name of the note or project to export.
        note: String,
        /// Force treating the name as a project.
        #[arg(long)]
        project: bool,
    },
    /// Export notes and projects to Markdown in a single step.
    #[command(name = "export_all_markdown")]
    ExportAllMarkdown {
        /// Export only notes.
        #[arg(long, short = 'n')]
        notes: bool,
        /// Export only projects.
        #[arg(long, short = 'p')]
        projects: bool,
    },

    /// Render a note or project, PDF by default.
    #[command(name = "render")]
    Render {
        /// Name of the note or project to render.
        name: String,
        /// Force treating the name as a project.
        #[arg(long)]
        project: bool,
        /// Output format (pdf|html).
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,
        /// Force running biber for the bibliography.
        #[arg(long)]
        biber: bool,
    },
    /// Render all notes and projects with configurable concurrency.
    #[command(name = "render_all")]
    RenderAll {
        /// Output format (pdf|html).
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,
        /// Number of parallel jobs.
        #[arg(long, short = 'j')]
        workers: Option<usize>,
        /// Render only notes.
        #[arg(long, short = 'n')]
        notes: bool,
        /// Render only projects.
        #[arg(long, short = 'p')]
        projects: bool,
    },
    /// Render only the items that are out of date according to database timestamps.
    #[command(name = "render_updates")]
    RenderUpdates {
        /// Output format (pdf|html).
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,
        /// Number of parallel jobs.
        #[arg(long, short = 'j')]
        workers: Option<usize>,
    },
    /// Watch for changes to LaTeX files and recompile the affected notes/projects.
    #[command(name = "watch")]
    Watch {
        /// Note or project to watch. Omit to watch the whole workspace.
        name: Option<String>,
        /// Treat `name` as a project.
        #[arg(long)]
        project: bool,
        /// Output format (pdf|html).
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,
        /// Number of parallel jobs (whole-workspace mode).
        #[arg(long, short = 'j')]
        workers: Option<usize>,
        /// Poll interval in milliseconds.
        #[arg(long, default_value_t = 800)]
        poll: u64,
    },
    /// Run biber for a specific note or project.
    #[command(name = "biber")]
    Biber {
        /// Name of the note or project.
        name: String,
        /// Force treating the name as a project.
        #[arg(long)]
        project: bool,
        /// Output folder (default: export directory).
        folder: Option<String>,
    },

    /// Synchronize notes and projects against the database.
    #[command(name = "synchronize")]
    Synchronize,
    /// Force a full synchronization of notes and projects.
    #[command(name = "force_synchronize")]
    ForceSynchronize {
        /// Force synchronizing only notes.
        #[arg(long, short = 'n')]
        notes: bool,
        /// Force synchronizing only projects and inclusions.
        #[arg(long, short = 'p')]
        projects: bool,
    },
    /// Validate references between notes and report broken links.
    #[command(name = "validate_references")]
    ValidateReferences {
        /// Validate only notes.
        #[arg(long, short = 'n', default_value_t = false)]
        notes: bool,
        /// Validate only projects.
        #[arg(long, short = 'p', default_value_t = false)]
        projects: bool,
    },
    /// Remove orphaned pdf and markdown from the export directories.
    #[command(name = "clean")]
    Clean,
    /// Remove duplicate citations detected during note processing.
    #[command(name = "remove_duplicate_citations")]
    RemoveDuplicateCitations,

    /// Open a note or project in the external editor.
    #[command(name = "edit")]
    Edit {
        /// Name of the note to edit (default: last note).
        name: Option<String>,
        /// Treat `name` as a project.
        #[arg(long)]
        project: bool,
    },

    /// Open the fuzzy interface for search and quick actions.
    #[command(name = "fuzzy")]
    Fuzzy {
        /// Run the session in the current terminal.
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

    /// Run the Language Server Protocol (LSP) server over stdio.
    ///
    /// Used by the VS Code extension to provide contextual completion of note
    /// and label names inside `\excref`/`\exref`/`\exhyperref` as you type.
    #[command(name = "lsp")]
    Lsp {
        /// Accepted for editor compatibility (always served over stdio).
        /// `vscode-languageclient` appends this to the server invocation.
        #[arg(long, hide = true)]
        stdio: bool,
    },
}
