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
#[command(about = "CLI Rust para gestionar ZettelTeX")]
pub struct Cli {
    /// Directorio raiz del workspace.
    #[arg(long, default_value = ".")]
    pub workspace_root: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Crear una nueva nota en notes/slipbox y registrarla en la base de datos.
    #[command(name = "newnote")]
    Newnote {
        /// Nombre de la nota (sin extension .tex).
        name: String,
    },
    /// Crear la estructura minima de un workspace (notes/slipbox, projects, template) y sus plantillas.
    #[command(name = "init")]
    Init,
    /// Crear zetteltex.toml interactivamente en la raiz del workspace.
    #[command(name = "init_config")]
    InitConfig,
    /// Renombrar interactivamente una nota (archivo y etiquetas asociadas).
    #[command(name = "rename_note")]
    RenameNote {
        /// Nombre de la nota a renombrar.
        name: String,
    },
    /// Eliminar una nota del workspace y de los indices de soporte.
    #[command(name = "remove_note")]
    RemoveNote {
        /// Nombre de la nota a eliminar.
        name: String,
    },
    /// Listar las notas mas recientes, con limite configurable.
    #[command(name = "list_recent_files")]
    ListRecentFiles {
        /// Numero de notas a listar (default: valor de configuracion).
        n: Option<usize>,
    },
    /// Mostrar notas sin referencias entrantes o uso detectado.
    #[command(name = "list_unreferenced")]
    ListUnreferenced,
    /// Renombrar la nota reciente numero n segun orden de recencia.
    #[command(name = "rename_recent")]
    RenameRecent {
        /// Indice (1-based) de la nota reciente a renombrar.
        n: Option<usize>,
    },
    /// Agregar una nota a notes/documents.tex para referencias cruzadas de LaTeX.
    #[command(name = "addtodocuments")]
    AddToDocuments {
        /// Nombre de la nota a agregar.
        name: String,
    },
    /// Listar citas detectadas en una nota.
    #[command(name = "list_citations")]
    ListCitations {
        /// Nombre de la nota a inspeccionar.
        name: String,
    },

    /// Crear un nuevo proyecto en projects/ con su archivo .tex base.
    #[command(name = "newproject")]
    Newproject {
        /// Nombre del proyecto (sin extension .tex).
        name: String,
    },
    /// Listar los proyectos conocidos por la base de datos.
    #[command(name = "list_projects")]
    ListProjects,
    /// Mostrar las notas incluidas en un proyecto segun transclude.
    #[command(name = "list_project_inclusions")]
    ListProjectInclusions {
        /// Nombre del proyecto.
        project: String,
    },
    /// Listar los proyectos donde aparece una nota via inclusion.
    #[command(name = "list_note_projects")]
    ListNoteProjects {
        /// Nombre de la nota.
        note: String,
    },
    /// Exportar un proyecto a una carpeta de salida.
    #[command(name = "export_project")]
    ExportProject {
        /// Carpeta del proyecto a exportar.
        folder: String,
        /// Archivo .tex principal dentro del proyecto (default: <folder>.tex).
        texfile: Option<String>,
    },
    /// Convertir/volcar un archivo de entrada a un borrador de salida.
    #[command(name = "export_draft")]
    ExportDraft {
        /// Archivo de entrada.
        input_file: String,
        /// Archivo de salida del borrador.
        output_file: String,
    },

    /// Exportar una nota o proyecto a Markdown usando la configuracion de export.
    #[command(name = "export_markdown")]
    ExportMarkdown {
        /// Nombre de la nota o proyecto a exportar.
        note: String,
        /// Fuerza a tratar el nombre como proyecto.
        #[arg(long)]
        project: bool,
    },
    /// Exportar notas y proyectos a Markdown en un solo paso.
    #[command(name = "export_all_markdown")]
    ExportAllMarkdown {
        /// Exportar solo notas.
        #[arg(long)]
        notes: bool,
        /// Exportar solo proyectos.
        #[arg(long)]
        projects: bool,
    },

    /// Renderizar una nota o un proyecto, por defecto en PDF.
    #[command(name = "render")]
    Render {
        /// Nombre de la nota o proyecto a renderizar.
        name: String,
        /// Fuerza a tratar el nombre como proyecto.
        #[arg(long)]
        project: bool,
        /// Formato de salida (pdf|html).
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,
        /// Fuerza la ejecucion de biber para la bibliografia.
        #[arg(long)]
        biber: bool,
    },
    /// Renderizar todas las notas y proyectos con concurrencia configurable.
    #[command(name = "render_all")]
    RenderAll {
        /// Formato de salida (pdf|html).
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,
        /// Numero de trabajos en paralelo.
        #[arg(long, short = 'j')]
        workers: Option<usize>,
        /// Renderizar solo notas.
        #[arg(long)]
        notes_only: bool,
        /// Renderizar solo proyectos.
        #[arg(long)]
        projects_only: bool,
    },
    /// Renderizar solo los elementos desactualizados segun timestamps de la base de datos.
    #[command(name = "render_updates")]
    RenderUpdates {
        /// Formato de salida (pdf|html).
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,
        /// Numero de trabajos en paralelo.
        #[arg(long, short = 'j')]
        workers: Option<usize>,
    },
    /// Ejecutar biber para una nota o proyecto concreto.
    #[command(name = "biber")]
    Biber {
        /// Nombre de la nota o proyecto.
        name: String,
        /// Fuerza a tratar el nombre como proyecto.
        #[arg(long)]
        project: bool,
        /// Carpeta de salida (default: directorio de exportacion).
        folder: Option<String>,
    },

    /// Sincronizar notas y proyectos contra la base de datos.
    #[command(name = "synchronize")]
    Synchronize,
    /// Forzar sincronizacion completa de notas y proyectos.
    #[command(name = "force_synchronize")]
    ForceSynchronize {
        /// Forzar sincronizacion solo de notas.
        #[arg(long)]
        notes_only: bool,
        /// Forzar sincronizacion solo de proyectos e inclusiones.
        #[arg(long)]
        projects_only: bool,
    },
    /// Validar referencias entre notas y reportar enlaces rotos.
    #[command(name = "validate_references")]
    ValidateReferences {
        /// Validar solo notas.
        #[arg(long, default_value_t = false)]
        notes_only: bool,
        /// Validar solo proyectos.
        #[arg(long, default_value_t = false)]
        projects_only: bool,
    },
    /// Eliminar pdf y markdown huerfanos de los directorios de exportacion.
    #[command(name = "clean")]
    Clean,
    /// Eliminar citas duplicadas detectadas durante el procesamiento de notas.
    #[command(name = "remove_duplicate_citations")]
    RemoveDuplicateCitations,

    /// Abrir una nota en el editor externo.
    #[command(name = "edit")]
    Edit {
        /// Nombre de la nota a editar (default: ultima nota).
        name: Option<String>,
    },

    /// Abrir la interfaz fuzzy para busqueda y acciones rapidas.
    #[command(name = "fuzzy")]
    Fuzzy {
        /// Ejecuta la sesion en la terminal actual.
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
