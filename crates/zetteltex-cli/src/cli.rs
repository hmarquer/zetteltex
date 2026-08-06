use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "zetteltex")]
#[command(about = "CLI Rust para gestionar ZettelTeX")]
pub struct Cli {
    #[arg(long, default_value = ".")]
    pub workspace_root: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "newnote")]
    Newnote { name: String },
    #[command(name = "init")]
    Init,
    #[command(name = "init_config")]
    InitConfig,
    #[command(name = "rename_note")]
    RenameNote { name: String },
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
    ValidateReferences {
        #[arg(long, default_value_t = false)]
        notes_only: bool,
        #[arg(long, default_value_t = false)]
        projects_only: bool,
    },
    #[command(name = "clean")]
    Clean,
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
