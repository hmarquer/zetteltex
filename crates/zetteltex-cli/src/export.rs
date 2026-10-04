use super::*;
use crate::i18n::tr;
use zetteltex_core::validate_component_name;

pub(crate) fn export_projects_dir(paths: &WorkspacePaths) -> PathBuf {
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

pub(crate) fn export_notes_dir(paths: &WorkspacePaths) -> PathBuf {
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

pub(crate) fn subject_tags_for_note(
    paths: &WorkspacePaths,
    note_name: &str,
) -> Result<Vec<String>> {
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

pub(crate) fn export_note_markdown_file(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!(tr!(
            "Nota {note_name} no encontrada en la base de datos",
            "Note {note_name} not found in database"
        ));
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let content = fs::read_to_string(&note_path)?;
    let parsed = parse_note(&content);

    let out_dir = export_notes_dir(paths);
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{note_name}.md"));

    let meta = db.note_metadata_by_filename(note_name)?;
    let title = meta
        .as_ref()
        .and_then(|m| m.title.as_deref())
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
        .or_else(|| extract_title_from_tex_content(&content))
        .unwrap_or_else(|| note_name.to_string());
    let tags = subject_tags_for_note(paths, note_name)?;
    let references = parsed
        .references
        .iter()
        .map(|r| r.target_note.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let backlinks = db.notes_referencing_note(note_name)?;
    let citations = db.citations_for_note(note_name)?;
    let labels = db.labels_for_note(note_name)?;
    let projects = db
        .list_note_projects(note_name)?
        .into_iter()
        .map(|p| p.project_name)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let keywords = db.note_keywords(note_name)?;

    let mut out = String::new();
    out.push_str("---\n");
    push_frontmatter_str(&mut out, "title", Some(&title));
    push_frontmatter_str(&mut out, "filename", Some(note_name));
    if let Some(m) = &meta {
        push_frontmatter_str(&mut out, "created", m.created.as_deref());
        push_frontmatter_str(&mut out, "last_edit_date", m.last_edit_date.as_deref());
        push_frontmatter_str(
            &mut out,
            "last_build_date_pdf",
            m.last_build_date_pdf.as_deref(),
        );
        push_frontmatter_str(
            &mut out,
            "last_build_date_html",
            m.last_build_date_html.as_deref(),
        );
    }
    push_frontmatter_list(&mut out, "labels", &labels);
    push_frontmatter_list(&mut out, "references", &references);
    push_frontmatter_list(&mut out, "backlinks", &backlinks);
    push_frontmatter_list(&mut out, "citations", &citations);
    push_frontmatter_list(&mut out, "projects", &projects);
    if !tags.is_empty() {
        out.push_str("tags:\n");
        for tag in &tags {
            out.push_str(&format!("  - {tag}\n"));
        }
    }
    out.push_str("---\n\n");

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

pub(crate) fn export_markdown(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    println!(
        "{}: {}='{}' | sync=true | salida={} ",
        tr("Plan export_markdown", "Export markdown plan"),
        tr("nota", "note"),
        note_name,
        export_notes_dir(paths).display()
    );
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    export_note_markdown_file(paths, note_name)
}

pub(crate) fn export_project_markdown_file(
    paths: &WorkspacePaths,
    project_name: &str,
) -> Result<()> {
    let project_dir = paths.projects.join(project_name);
    let main_tex = project_dir.join(format!("{project_name}.tex"));
    if !main_tex.exists() {
        bail!(
            "{}: {}",
            tr(
                "Archivo principal del proyecto no encontrado",
                "Project main tex not found"
            ),
            main_tex.display()
        );
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
    let keywords = db.project_keywords(project_name)?;
    let meta = db.project_metadata_by_name(project_name)?;
    let inclusion_names = inclusions
        .iter()
        .map(|inc| inc.note_filename.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut out = String::new();
    out.push_str("---\n");
    push_frontmatter_str(&mut out, "title", Some(&title));
    push_frontmatter_str(&mut out, "name", Some(project_name));
    if let Some(m) = &meta {
        push_frontmatter_str(&mut out, "created", m.created.as_deref());
        push_frontmatter_str(&mut out, "last_edit_date", m.last_edit_date.as_deref());
        push_frontmatter_str(
            &mut out,
            "last_build_date_pdf",
            m.last_build_date_pdf.as_deref(),
        );
        push_frontmatter_str(
            &mut out,
            "last_build_date_html",
            m.last_build_date_html.as_deref(),
        );
    }
    push_frontmatter_list(&mut out, "inclusions", &inclusion_names);
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

pub(crate) fn export_project_markdown(paths: &WorkspacePaths, project_name: &str) -> Result<()> {
    println!(
        "{}: {}='{}' | sync=true | salida={}",
        tr(
            "Plan export_project_markdown",
            "Export project markdown plan"
        ),
        tr("proyecto", "project"),
        project_name,
        export_projects_dir(paths).display()
    );
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    export_project_markdown_file(paths, project_name)
}

pub(crate) fn export_all_notes_markdown(paths: &WorkspacePaths) -> Result<()> {
    let note_names: Vec<String> = fs::read_dir(&paths.notes_slipbox)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| note_stem_from_path(&path))
        .collect();

    println!(
        "{}: {}={} | sync=true | salida={}",
        tr("Plan export_all (notas)", "Export all plan (notes)"),
        tr("notas", "notes"),
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
        "{} {} {} {}",
        tr("Exportadas", "Exported"),
        tr!("{} nota(s)", "{} note(s)", count),
        tr("a", "to"),
        export_notes_dir(paths).display()
    );
    Ok(())
}

pub(crate) fn export_all_projects_markdown(paths: &WorkspacePaths) -> Result<()> {
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;

    let projects = db.list_projects()?;
    println!(
        "{}: {}={} | sync=true | salida={}",
        tr("Plan export_all (proyectos)", "Export all plan (projects)"),
        tr("proyectos", "projects"),
        projects.len(),
        export_projects_dir(paths).display()
    );

    let mut count = 0usize;
    for p in projects {
        export_project_markdown_file(paths, &p.name)?;
        count += 1;
    }

    println!(
        "{} {} {} {}",
        tr("Exportados", "Exported"),
        tr!("{} proyecto(s)", "{} project(s)", count),
        tr("a", "to"),
        export_projects_dir(paths).display()
    );
    Ok(())
}

pub(crate) fn export_all_markdown(
    paths: &WorkspacePaths,
    notes: bool,
    projects: bool,
) -> Result<()> {
    if notes {
        export_all_notes_markdown(paths)?;
    }
    if projects {
        export_all_projects_markdown(paths)?;
    }
    Ok(())
}

fn clean_project_tag(project_name: &str) -> String {
    let without_prefix = project_name
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim_start_matches('-');
    without_prefix.to_string()
}

fn push_frontmatter_str(out: &mut String, key: &str, value: Option<&str>) {
    if let Some(v) = value {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            out.push_str(&format!("{key}: '{}'\n", trimmed.replace('\'', "''")));
        }
    }
}

fn push_frontmatter_list(out: &mut String, key: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    out.push_str(&format!("{key}:\n"));
    for v in values {
        out.push_str(&format!("  - {v}\n"));
    }
}

const REF_FALLBACK_COMMANDS: &str = r#"% Comandos de referencia con respaldo para el fichero standalone.
% Si la etiqueta <nota>-<tag> está definida (la nota está incrustada en este fichero)
% se enlaza con normalidad (sin mostrar el nombre de la nota); si no, se muestra
% el nombre de la nota en tipo monoespaciado.
\makeatletter
\renewcommand{\exhyperref}[3][note]{%
    \ifcsname r@#2-#1\endcsname
        \ztxmaybehyperlink{\hyperref[#2-#1]{#3}}{#3}%
    \else
        #3%
    \fi
}
\renewcommand{\excref}[2][note]{%
    \ifcsname r@#2-#1\endcsname
        \ifthenelse{\equal{#1}{note}}
            {\ztxmaybehyperlink{\hyperref[#2-note]{\cref{#2-note}}}{\cref{#2-note}}}%
            {\ztxmaybehyperlink{\hyperref[#2-#1]{\cref*{#2-#1}}}{\cref*{#2-#1}}}%
    \else
        \texttt{#2}%
    \fi
}
\renewcommand{\exref}[2][note]{%
    \ifcsname r@#2-#1\endcsname
        \ifthenelse{\equal{#1}{note}}
            {\ztxmaybehyperlink{\hyperref[#2-note]{\ref{#2-note}}}{\ref{#2-note}}}%
            {\ztxmaybehyperlink{\hyperref[#2-#1]{\ref*{#2-#1}}}{\ref*{#2-#1}}}%
    \else
        \texttt{#2}%
    \fi
}
\renewcommand{\transclude}[2][note]{%
    \PackageWarning{standalone}{Transclusion no expandida: #2/#1}%
}
\makeatother
"#;

const ZTX_STYLE_HOOK: &str = "__ZTX_STYLE__";
const ZTX_BIB_RES_HOOK: &str = "__ZTX_BIB_RES__\n";

fn read_template_raw(paths: &WorkspacePaths, fname: &str) -> Result<String> {
    let path = paths.template.join(fname);
    if !path.exists() {
        bail!(
            "{}: {}",
            tr("Plantilla no encontrada", "Template not found"),
            path.display()
        );
    }
    Ok(fs::read_to_string(&path)?)
}

/// Elimina líneas que cumplen el predicado (diagnóstico de paquetes/clases).
fn strip_lines(text: &str, pred: impl Fn(&str) -> bool) -> String {
    text.lines()
        .filter(|l| !pred(l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convierte `\LoadClass[opts]{base}` en `\documentclass[opts]{base}` y quita
/// las líneas propias de una clase. Devuelve (línea \documentclass, resto).
fn process_class(raw: &str, fallback_base: &str) -> Result<(String, String)> {
    let cleaned = strip_lines(raw, |l| {
        let t = l.trim_start();
        t.starts_with("\\NeedsTeXFormat")
            || t.starts_with("\\ProvidesClass")
            || t.starts_with("\\ProcessOptions")
            || (t.starts_with("\\subimport") && t.contains("documents.tex"))
    });
    let docclass_re = Regex::new(r"\\LoadClass\s*(?:\[([^\]]*)\])?\s*\{([^}]*)\}")?;
    let docclass = if let Some(cap) = docclass_re.captures(&cleaned) {
        let opts = cap
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        let base = cap.get(2).map(|m| m.as_str()).unwrap_or(fallback_base);
        match opts {
            Some(o) => format!("\\documentclass[{o}]{{{base}}}"),
            None => format!("\\documentclass{{{base}}}"),
        }
    } else {
        format!("\\documentclass{{{fallback_base}}}")
    };
    let rest = docclass_re.replace_all(&cleaned, "").to_string();
    Ok((docclass, rest))
}

/// Sustituye cada `\addbibresource{...}` por el gancho `__ZTX_BIB_RES__`.
fn hook_bib_resource(text: &str) -> String {
    Regex::new(r"(?m)^\s*\\addbibresource\{[^}]*\}\s*\n?")
        .unwrap()
        .replace_all(text, ZTX_BIB_RES_HOOK)
        .to_string()
}

fn process_ztxbase(raw: &str) -> String {
    let cleaned = strip_lines(raw, |l| {
        let t = l.trim_start();
        t.starts_with("\\NeedsTeXFormat") || t.starts_with("\\ProvidesPackage")
    });
    let with_style = Regex::new(r"(?m)^\s*\\usepackage\{[^}]*template/style\}\s*\n?")
        .unwrap()
        .replace_all(&cleaned, &format!("\\makeatletter\n{ZTX_STYLE_HOOK}\n\\makeatother\n"))
        .to_string();
    hook_bib_resource(&with_style)
}

fn process_style(raw: &str) -> String {
    let cleaned = strip_lines(raw, |l| {
        let t = l.trim_start();
        t.starts_with("\\NeedsTeXFormat") || t.starts_with("\\ProvidesPackage")
    });
    hook_bib_resource(&cleaned)
}

/// Extrae el argumento balanceado `{...}` del primer `token` (p. ej. `\title{`).
fn extract_braced_arg(content: &str, token: &str) -> Option<String> {
    let start = content.find(token)? + token.len();
    let mut depth = 1usize;
    let bytes = content.as_bytes();
    let mut i = start;
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

/// Antepone `<prefix>-` a las etiquetas y referencias internas de `text`, igual
/// que `xr-hyper` prefija los labels de cada nota en la compilación normal.
/// Cubre `\label{...}`, `\ref`/`\eqref`/`\cref`/`\Cref` (con o sin `*`) y los
/// destinos de `\hyperref[...]`. Así `\exref[tag]{nota}`, `\excref[tag]{nota}` y
/// `\exhyperref[tag]{nota}{...}` resuelven cuando la nota está incrustada en el
/// fichero y solo muestran el nombre de la nota cuando su etiqueta no está.
fn prefix_labels_and_refs(text: &str, prefix: &str) -> String {
    let out = Regex::new(r"\\label\{([^}]*)\}")
        .expect("label regex")
        .replace_all(text, |caps: &regex::Captures<'_>| {
            format!("\\label{{{}-{}}}", prefix, &caps[1])
        });
    let out = Regex::new(r"\\((?:eq)?ref|cref|Cref)(\*?)\{([^}]*)\}")
        .expect("ref regex")
        .replace_all(&out, |caps: &regex::Captures<'_>| {
            format!("\\{}{}{{{}-{}}}", &caps[1], &caps[2], prefix, &caps[3])
        });
    Regex::new(r"\\hyperref\[([^\]]*)\]")
        .expect("hyperref regex")
        .replace_all(&out, |caps: &regex::Captures<'_>| {
            format!("\\hyperref[{}-{}]", prefix, &caps[1])
        })
        .to_string()
}

/// Expande recursivamente `\transclude[tag]{note}` y `\input`/`\subimport`
/// sobre el contenido dado. `base_dir` indica dónde se resuelven los nombres
/// relativos (carpeta de la nota o del proyecto según el fichero que se procesa).
/// `anchors` garantiza que el ancla `<nota>-note` de cada nota incrustada se
/// inserte una sola vez; `stack` detecta ciclos de transclusión/input.
fn expand_source(
    paths: &WorkspacePaths,
    text: &str,
    base_dir: &Path,
    stack: &mut Vec<String>,
    anchors: &mut BTreeSet<String>,
) -> Result<String> {
    let trans_re = Regex::new(r"\\transclude(?:\[([^\]]+)\])?\{([^}]+)\}")?;
    let input_re = Regex::new(r"\\subimport\{([^}]*)\}\{([^}]+)\}|\\input\s*\{([^}]+)\}")?;

    let tran = trans_re.find(text);
    let inp = input_re.find(text);
    let m = match (tran, inp) {
        (None, None) => return Ok(text.to_string()),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (Some(a), Some(b)) => {
            if a.start() <= b.start() {
                a
            } else {
                b
            }
        }
    };

    let (start, end, expanded) = if let Some(caps) = trans_re.captures_at(text, m.start()) {
        let whole = caps.get(0).expect("group 0");
        let tag = caps
            .get(1)
            .map(|x| x.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "note".to_string());
        let note = caps.get(2).map(|x| x.as_str().trim()).unwrap_or_default();
        validate_component_name(note)?;
        let key = format!("trans:{note}:{tag}");
        if stack.contains(&key) {
            let chain = stack
                .iter()
                .cloned()
                .chain(std::iter::once(key))
                .collect::<Vec<_>>()
                .join(" → ");
            bail!(
                "{}: {}",
                tr("Ciclo detectado en las transclusiones", "Transclusion cycle detected"),
                chain
            );
        }
        let note_path = paths.notes_slipbox.join(format!("{note}.tex"));
        let note_content = match fs::read_to_string(&note_path) {
            Ok(s) => s,
            Err(_) => bail!(
                "{}: {}",
                tr("Nota no encontrada", "Note not found"),
                note_path.display()
            ),
        };
        let block = extract_tagged_block(&note_content, &tag)?;
        let block = match block {
            Some(b) => b,
            None => bail!(
                "{} <*{}>...</{}> {}: {}",
                tr!("Etiqueta", "Tag"),
                tag,
                tag,
                tr!("no encontrada en", "not found in"),
                note_path.display()
            ),
        };
        let mut block = prefix_labels_and_refs(&block, note);
        if anchors.insert(note.to_string()) {
            block = format!("\\phantomsection\\label{{{note}-note}}\n{block}");
        }
        stack.push(key);
        let expanded = expand_source(paths, &block, &paths.notes_slipbox, stack, anchors)?;
        stack.pop();
        (whole.start(), whole.end(), expanded)
    } else if let Some(caps) = input_re.captures_at(text, m.start()) {
        let whole = caps.get(0).expect("group 0");
        let name = caps
            .get(3)
            .map(|x| x.as_str())
            .or_else(|| caps.get(2).map(|x| x.as_str()))
            .expect("input name")
            .trim();
        let subdir = caps.get(1).map(|x| x.as_str().trim()).unwrap_or_default();
        let relative = if subdir.is_empty() {
            name.to_string()
        } else {
            format!("{subdir}/{name}")
        };
        let resolved = if relative.ends_with(".tex") {
            base_dir.join(&relative)
        } else {
            base_dir.join(format!("{relative}.tex"))
        };
        if !resolved.exists() {
            bail!(
                "{}: {}",
                tr("No se encontró el fichero incluido", "Included file not found"),
                resolved.display()
            );
        }
        let key = format!("input:{}", resolved.display());
        if stack.contains(&key) {
            bail!(
                "{}: {}",
                tr("Ciclo detectado en las inclusiones", "Inclusion cycle detected"),
                key
            );
        }
        let content = fs::read_to_string(&resolved)?;
        let new_base = resolved.parent().unwrap_or(base_dir);
        stack.push(key);
        let expanded = expand_source(paths, &content, new_base, stack, anchors)?;
        stack.pop();
        (whole.start(), whole.end(), expanded)
    } else {
        unreachable!("regex match without captures");
    };

    let mut out = String::with_capacity(text.len() + expanded.len());
    out.push_str(&text[..start]);
    out.push_str(&expanded);
    out.push_str(&expand_source(paths, &text[end..], base_dir, stack, anchors)?);
    Ok(out)
}

/// Recoge las claves citadas (cite, parencite, textcite, autocite, ...).
fn extract_citations(text: &str) -> BTreeSet<String> {
    let re = Regex::new(r"\\[A-Za-z]*cite(?:\s*(?:\[[^\]]*\]){1,2})?\s*\{([^}]*)\}").unwrap();
    let mut keys = BTreeSet::new();
    for caps in re.captures_iter(text) {
        if let Some(m) = caps.get(1) {
            for key in m.as_str().split(',').map(|s| s.trim()) {
                if !key.is_empty() {
                    keys.insert(key.to_string());
                }
            }
        }
    }
    keys
}

/// Extrae las entradas `@tipo{clave, ...}` de un `.bib`.
fn extract_bib_entries(content: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"@(?P<kind>[A-Za-z]+)\s*\{").unwrap();
    let mut entries = Vec::new();
    for caps in re.captures_iter(content) {
        let kind = caps.name("kind").expect("kind").as_str();
        if matches!(kind, "comment" | "string" | "preamble") {
            continue;
        }
        let m = caps.get(0).expect("group 0");
        let open = m.end();
        let rest = &content[open..];
        let key_end = rest.find(',').or_else(|| rest.find('}')).unwrap_or(rest.len());
        let key = rest[..key_end].trim().to_string();
        if key.is_empty() {
            continue;
        }
        let mut depth = 1usize;
        let mut close = None;
        for (i, ch) in content[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(open + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = close.map(|c| c + 1).unwrap_or(content.len());
        entries.push((key, content[m.start()..end].to_string()));
    }
    entries
}

/// Devuelve el texto `.bib` con únicamente las entradas citadas.
fn filter_bibliography(paths: &WorkspacePaths, cited: &BTreeSet<String>) -> Result<String> {
    let bib_path = paths.root.join("bibliography.bib");
    let content = fs::read_to_string(&bib_path)?;
    let entries = extract_bib_entries(&content);
    let found = entries
        .iter()
        .map(|(k, _)| k.as_str())
        .collect::<BTreeSet<_>>();
    let missing = cited
        .iter()
        .map(String::as_str)
        .filter(|k| !found.contains(k))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        bail!(
            "{}: {} ({})",
            tr("Cita no encontrada en", "Citation not found in"),
            bib_path.display(),
            missing.join(", ")
        );
    }
    let mut out = String::new();
    for (key, text) in &entries {
        if cited.contains(key) {
            out.push_str(&format!("{text}\n"));
        }
    }
    Ok(out)
}

/// Genera un `.tex` standalone autocontenido a partir de una nota o proyecto:
/// incrusta la clase, ztxbase.sty y style.sty, expande las transclusiones
/// (`\transclude`) recursivamente, incrusta solo las citas usadas y redefine los
/// comandos de referencia para que muestren el nombre de la nota cuando su
/// etiqueta no está presente en el fichero.
pub(crate) fn export_standalone(
    paths: &WorkspacePaths,
    name: &str,
    output: &str,
    project: bool,
) -> Result<()> {
    let kind = resolve_note_or_project(paths, name, project)?;
    let (src_path, class_fname, base_class) = match kind {
        TargetKind::Note => (
            paths.notes_slipbox.join(format!("{name}.tex")),
            "texnote.cls",
            "article",
        ),
        TargetKind::Project => (
            paths.projects.join(name).join(format!("{name}.tex")),
            "texbook.cls",
            "report",
        ),
    };
    if !src_path.exists() {
        bail!(
            "{}: {}",
            tr("Archivo no encontrado", "File not found"),
            src_path.display()
        );
    }

    let src = fs::read_to_string(&src_path)?;
    let (preamble_src, body_src) = match src.split_once("\\begin{document}") {
        Some((pre, rest)) => {
            let body = match rest.rfind("\\end{document}") {
                Some(i) => &rest[..i],
                None => rest,
            };
            (pre.trim().to_string(), body.to_string())
        }
        None => bail!(
            "{}: {}",
            tr("No se encontró \\\\begin{{document}} en", "No \\begin{document} found in"),
            src_path.display()
        ),
    };

    let title = extract_braced_arg(&preamble_src, "\\title{");
    let author = extract_braced_arg(&preamble_src, "\\author{");
    let date = extract_braced_arg(&preamble_src, "\\date{");

    let (docclass, class_body) = process_class(&read_template_raw(paths, class_fname)?, base_class)?;
    let style_inlined = match read_template_raw(paths, "style.sty") {
        Ok(s) => process_style(&s),
        Err(_) => String::new(),
    };
    let ztxbase_inlined = process_ztxbase(&read_template_raw(paths, "ztxbase.sty")?)
        .replace(ZTX_STYLE_HOOK, &style_inlined);
    let preamble_src = format!(
        "{docclass}\n\\makeatletter\n{}\n\\makeatother\n",
        class_body.replace(
            "\\RequirePackage{../../template/ztxbase}",
            &format!(
                "\\makeatletter\n{ztxbase_inlined}\n\\makeatother\n\\makeatletter"
            )
        )
    );

    // Cuerpo: expandir transclusiones e inclusiones (input/subimport).
    let project_dir = if kind == TargetKind::Project {
        Some(paths.projects.join(name))
    } else {
        None
    };
    let base_dir = match kind {
        TargetKind::Note => paths.notes_slipbox.as_path(),
        TargetKind::Project => project_dir.as_deref().expect("project dir"),
    };
    let mut anchors = BTreeSet::new();
    anchors.insert(name.to_string());
    let mut stack = Vec::new();
    let root_src = if kind == TargetKind::Note {
        prefix_labels_and_refs(&body_src, name)
    } else {
        body_src
    };
    let mut body = expand_source(paths, &root_src, base_dir, &mut stack, &mut anchors)?;

    // Citas: incrustar solo las entradas usadas.
    let citations = extract_citations(&body);
    let bib_block = if citations.is_empty() {
        String::new()
    } else {
        let entries = filter_bibliography(paths, &citations)?;
        format!(
            "\\begin{{filecontents*}}{{\\jobname.bib}}\n{entries}\n\\end{{filecontents*}}\n\\addbibresource{{\\jobname.bib}}\n"
        )
    };
    if citations.is_empty() {
        body = body.replace(
            "\\printbibliography",
            "% \\printbibliography: la exportación no contiene citas",
        );
    }

    let mut out = String::with_capacity(preamble_src.len() + body.len() + 512);
    out.push_str(&preamble_src);
    out.push('\n');
    if let Some(t) = title {
        out.push_str(&format!("\\title{{{t}}}\n"));
    }
    if let Some(a) = author {
        out.push_str(&format!("\\author{{{a}}}\n"));
    }
    if let Some(d) = date {
        out.push_str(&format!("\\date{{{d}}}\n"));
    }
    out.push('\n');
    out.push_str(REF_FALLBACK_COMMANDS);
    out = out.replace(ZTX_BIB_RES_HOOK, &bib_block);
    out.push_str("\\begin{document}\n");
    out.push_str(&format!("\\phantomsection\\label{{{name}-note}}\n"));
    out.push_str(body.trim());
    out.push('\n');
    out.push_str("\\end{document}\n");

    let out_path = resolve_workspace_path(paths, output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, out)?;
    println!("{}: {}", tr("Exportado a", "Exported to"), out_path.display());
    Ok(())
}
