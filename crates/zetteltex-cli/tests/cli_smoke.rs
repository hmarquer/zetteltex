use assert_cmd::Command;
use predicates::str::contains;
use rusqlite::Connection;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn setup_workspace(root: &std::path::Path) {
    fs::create_dir_all(root.join("notes/slipbox")).expect("notes/slipbox");
    fs::create_dir_all(root.join("projects")).expect("projects");
    fs::create_dir_all(root.join("template")).expect("template");
    fs::write(root.join("notes/documents.tex"), "").expect("documents.tex");
    fs::write(
        root.join("template/note.tex"),
        "\\documentclass{texnote}\n\\title{Note Title}\n\\begin{document}\n\\currentdoc{note}\n\\end{document}\n",
    )
    .expect("template note");
    fs::write(
        root.join("template/project.tex"),
        "\\documentclass{texbook}\n\\title{Titulo}\n\\begin{document}\n\\end{document}\n",
    )
    .expect("template project");
    fs::write(
        root.join("zetteltex.toml"),
        "[general]\nlang = \"es\"\neditor = \"code\"\n",
    )
    .expect("zetteltex.toml");
}

fn install_fake_tool(bin_dir: &Path, name: &str, log_file: &Path) {
    let script = format!(
        "#!/bin/sh\necho \"{} $@\" >> \"{}\"\nexit 0\n",
        name,
        log_file.display()
    );
    let path = bin_dir.join(name);
    fs::write(&path, script).expect("write fake tool");
    let mut perms = fs::metadata(&path).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod");
}

fn prepend_path(dir: &Path) -> String {
    let old = env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), old)
}

fn logs_contain_biber_for(logs: &str, name: &str) -> bool {
    logs.lines()
        .any(|line| line.starts_with("biber ") && line.ends_with(name))
}

#[test]
fn help_works() {
    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex debe existir");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Rust CLI to manage ZettelTeX"));
}

#[test]
fn invalid_note_name_is_rejected() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("newnote")
        .arg("../../evil")
        .assert()
        .failure()
        .stderr(contains("nombre inválido"));
}

#[test]
fn invalid_command_fails() {
    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex debe existir");
    cmd.arg("comando_que_no_existe")
        .assert()
        .failure()
        .stderr(contains("unrecognized subcommand"));
}

#[test]
fn workspace_error_returns_exit_code_2() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .assert()
        .code(2)
        .stderr(contains("Workspace error"));
}

#[test]
fn command_runtime_error_returns_exit_code_1() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("nota")
        .arg("--format")
        .arg("pdf")
        .assert()
        .code(1)
        .stderr(contains("No existe nota ni proyecto con nombre 'nota'"));
}

#[test]
fn invalid_format_is_rejected_at_parse_time() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("nota")
        .arg("--format")
        .arg("docx")
        .assert()
        .code(2)
        .stderr(contains("possible values"));
}

#[test]
fn synchronize_and_validate_success() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/proj-a")).expect("projects");

    fs::write(
        root.join("notes/slipbox/a.tex"),
        "\\currentdoc{note}\n\\label{defn:a}\n",
    )
    .expect("write a");
    fs::write(root.join("notes/slipbox/b.tex"), "\\excref[defn:a]{a}\n").expect("write b");
    fs::write(root.join("projects/proj-a/proj-a.tex"), "\\transclude{a}\n").expect("write project");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success()
        .stdout(contains("Sincronizacion completa"));

    let mut validate_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    validate_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .success()
        .stdout(contains("Todas las referencias son validas"));

    let conn = Connection::open(root.join("slipbox.db")).expect("db open");
    let inclusion_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM inclusion", [], |row| row.get(0))
        .expect("query inclusion count");
    assert_eq!(inclusion_count, 1);
}

#[test]
fn validate_references_detects_missing_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/only.tex"),
        "\\excref[defn:ghost]{missing-note}\n",
    )
    .expect("write note");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .failure()
        .stdout(contains("missing_note"));
}

#[test]
fn validate_references_detects_missing_label_in_excref() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/existing.tex"), "\\label{defn:a}\n").expect("existing note");
    fs::write(
        root.join("notes/slipbox/ref.tex"),
        "\\excref[defn:ghost]{existing}\n",
    )
    .expect("ref note");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .failure()
        .stdout(contains("missing_label"));
}

#[test]
fn validate_references_detects_missing_project_local_ref() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::create_dir_all(root.join("projects/libro")).expect("project dir");
    fs::write(
        root.join("projects/libro/libro.tex"),
        "\\label{cap:1}\n\\ref{cap:2}\n",
    )
    .expect("project main");
    fs::write(root.join("projects/libro/cap2.tex"), "\\label{cap:2}\n").expect("project extra");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .success()
        .stdout(contains("Todas las referencias son validas"));
}

#[test]
fn validate_references_detects_missing_internal_label() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/note.tex"),
        "\\label{defn:a}\n\\ref{defn:ghost}\n",
    )
    .expect("note");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .failure()
        .stdout(contains("missing_label"));
}

#[test]
fn validate_references_detects_missing_transclude_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::create_dir_all(root.join("projects/p1")).expect("project dir");
    fs::write(root.join("projects/p1/p1.tex"), "\\transclude{missing}\n").expect("project");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .failure()
        .stdout(contains("missing_note"))
        .stdout(contains("transclude"));
}

#[test]
fn synchronize_rejects_missing_transclude_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::create_dir_all(root.join("projects/p1")).expect("project dir");
    fs::write(root.join("projects/p1/p1.tex"), "\\transclude{missing}\n").expect("project");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .failure()
        .stderr(contains("Falta la referencia a la nota"))
        .stderr(contains("transclude"));
}

#[test]
fn validate_references_passes_when_all_are_valid() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{defn:a}\n").expect("note a");
    fs::write(
        root.join("notes/slipbox/b.tex"),
        "\\excref[defn:a]{a}\n\\label{defn:b}\n\\ref{defn:b}\n",
    )
    .expect("note b");
    fs::create_dir_all(root.join("projects/p1")).expect("project dir");
    fs::write(
        root.join("projects/p1/p1.tex"),
        "\\transclude{a}\n\\excref[defn:b]{b}\n",
    )
    .expect("project");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .success()
        .stdout(contains("Todas las referencias son validas"));
}

#[test]
fn validate_references_notes_only_skips_projects() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::create_dir_all(root.join("projects/p1")).expect("project dir");
    fs::write(root.join("projects/p1/p1.tex"), "\\transclude{missing}\n").expect("project");
    fs::write(
        root.join("notes/slipbox/only.tex"),
        "\\label{defn:ok}\n\\ref{defn:ok}\n",
    )
    .expect("note");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .arg("--notes")
        .assert()
        .success()
        .stdout(contains("Todas las referencias son validas"));
}

#[test]
fn validate_references_projects_only_skips_notes() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/only.tex"),
        "\\excref[defn:ghost]{missing}\n",
    )
    .expect("note");
    fs::create_dir_all(root.join("projects/p1")).expect("project dir");
    fs::write(
        root.join("projects/p1/p1.tex"),
        "\\label{cap:1}\n\\ref{cap:1}\n",
    )
    .expect("project");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .arg("--projects")
        .assert()
        .success()
        .stdout(contains("Todas las referencias son validas"));
}

#[test]
fn validate_references_detects_missing_project_local_label() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::create_dir_all(root.join("projects/libro")).expect("project dir");
    fs::write(
        root.join("projects/libro/libro.tex"),
        "\\label{cap:1}\n\\ref{cap:2}\n",
    )
    .expect("project main");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("validate_references")
        .assert()
        .failure()
        .stdout(contains("missing_label"))
        .stdout(contains("cap:2"));
}

#[test]
fn list_project_commands_work() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/proj-list")).expect("projects/proj-list");

    fs::write(
        root.join("notes/slipbox/topic-a.tex"),
        "\\label{defn:topic-a}\n",
    )
    .expect("write topic-a");
    fs::write(
        root.join("notes/slipbox/topic-b.tex"),
        "\\label{defn:topic-b}\n",
    )
    .expect("write topic-b");

    fs::write(
        root.join("projects/proj-list/proj-list.tex"),
        "\\transclude{topic-a}\n\\transclude[demo]{topic-b}\n",
    )
    .expect("write project");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut list_projects = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    list_projects
        .arg("--workspace-root")
        .arg(root)
        .arg("list_projects")
        .assert()
        .success()
        .stdout(contains("proj-list"));

    let mut list_inclusions = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    list_inclusions
        .arg("--workspace-root")
        .arg(root)
        .arg("list_project_inclusions")
        .arg("proj-list")
        .assert()
        .success()
        .stdout(contains("topic-a"))
        .stdout(contains("topic-b"));

    let mut list_note_projects = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    list_note_projects
        .arg("--workspace-root")
        .arg(root)
        .arg("list_note_projects")
        .arg("topic-b")
        .assert()
        .success()
        .stdout(contains("proj-list"));
}

#[test]
fn newproject_and_newnote_commands_work() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut newproject = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    newproject
        .arg("--workspace-root")
        .arg(root)
        .arg("newproject")
        .arg("teoria_de_grafos")
        .assert()
        .success()
        .stdout(contains("Proyecto teoria_de_grafos creado en"));

    let project_path = root.join("projects/teoria_de_grafos/teoria_de_grafos.tex");
    let project_content = fs::read_to_string(project_path).expect("project tex");
    assert!(project_content.contains("\\title{Teoria de grafos}"));

    let mut newnote = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    newnote
        .arg("--workspace-root")
        .arg(root)
        .arg("newnote")
        .arg("mi_nota")
        .assert()
        .success();

    let note_path = root.join("notes/slipbox/mi_nota.tex");
    let note_content = fs::read_to_string(note_path).expect("note tex");
    assert!(note_content.contains("\\title{Mi nota}"));

    let documents = fs::read_to_string(root.join("notes/documents.tex")).expect("documents");
    assert!(documents.contains("\\externaldocument[mi_nota-]{mi_nota}"));

    let conn = Connection::open(root.join("slipbox.db")).expect("db open");
    let notes_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note WHERE filename = 'mi_nota'",
            [],
            |row| row.get(0),
        )
        .expect("notes count");
    let projects_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM project WHERE name = 'teoria_de_grafos'",
            [],
            |row| row.get(0),
        )
        .expect("projects count");
    assert_eq!(notes_count, 1);
    assert_eq!(projects_count, 1);
}

#[test]
fn list_recent_files_and_list_citations_work() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/older.tex"), "\\cite{knuth1984}\n").expect("older note");
    thread::sleep(Duration::from_millis(20));
    fs::write(
        root.join("notes/slipbox/newer.tex"),
        "\\cite{lamport1994}\\cite{knuth1984}\n",
    )
    .expect("newer note");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut recent_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    recent_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("list_recent_files")
        .arg("1")
        .assert()
        .success()
        .stdout(contains("newer"));

    let mut citations_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    citations_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("list_citations")
        .arg("newer")
        .assert()
        .success()
        .stdout(contains("lamport1994"))
        .stdout(contains("knuth1984"));
}

#[test]
fn rename_file_updates_references_and_db() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/p")).expect("projects/p");

    fs::write(root.join("notes/slipbox/old.tex"), "\\label{defn:a}\n").expect("old note");
    fs::write(
        root.join("notes/slipbox/ref.tex"),
        "\\excref[defn:a]{old}\\n\\hyperref[old-defn:a]{ver}\\n",
    )
    .expect("ref note");
    fs::write(root.join("projects/p/p.tex"), "\\transclude{old}\\n").expect("project");
    fs::write(
        root.join("notes/documents.tex"),
        "\\externaldocument[old-]{old}\n",
    )
    .expect("documents");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("old")
        .write_stdin("new\n\n")
        .assert()
        .success()
        .stdout(contains("Renombrado exitosamente old a new"));

    assert!(!root.join("notes/slipbox/old.tex").exists());
    assert!(root.join("notes/slipbox/new.tex").exists());

    let ref_content = fs::read_to_string(root.join("notes/slipbox/ref.tex")).expect("ref read");
    assert!(ref_content.contains("\\excref[defn:a]{new}"));
    assert!(ref_content.contains("\\hyperref[new-defn:a]"));

    let project_content = fs::read_to_string(root.join("projects/p/p.tex")).expect("project read");
    assert!(project_content.contains("\\transclude{new}"));

    let docs = fs::read_to_string(root.join("notes/documents.tex")).expect("docs read");
    assert!(docs.contains("\\externaldocument[new-]{new}"));
    assert!(!docs.contains("\\externaldocument[old-]{old}"));

    let conn = Connection::open(root.join("slipbox.db")).expect("db open");
    let old_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note WHERE filename='old'",
            [],
            |row| row.get(0),
        )
        .expect("old count");
    let new_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note WHERE filename='new'",
            [],
            |row| row.get(0),
        )
        .expect("new count");
    assert_eq!(old_count, 0);
    assert_eq!(new_count, 1);
}

#[test]
fn rename_preserves_dollar_in_new_name() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/old.tex"), "\\label{defn:a}\n").expect("old note");
    fs::write(
        root.join("notes/slipbox/ref.tex"),
        "\\excref[defn:a]{old}\n",
    )
    .expect("ref note");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("old")
        .write_stdin("new$name\n\n")
        .assert()
        .success();

    let ref_content = fs::read_to_string(root.join("notes/slipbox/ref.tex")).expect("ref read");
    assert!(ref_content.contains("\\excref[defn:a]{new$name}"));
}

#[test]
fn rename_file_removes_stale_export_artifacts() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/old.tex"), "\\label{defn:a}\n").expect("old note");
    fs::create_dir_all(root.join("pdf")).expect("pdf dir");
    fs::write(root.join("pdf/old.pdf"), "old pdf").expect("old pdf");
    fs::create_dir_all(root.join("jabberwocky/latex/zettelkasten")).expect("markdown dir");
    fs::write(root.join("jabberwocky/latex/zettelkasten/old.md"), "old md").expect("old md");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("old")
        .write_stdin("new\n\n")
        .assert()
        .success();

    assert!(!root.join("pdf/old.pdf").exists());
    assert!(!root.join("jabberwocky/latex/zettelkasten/old.md").exists());
}

#[test]
fn clean_removes_orphan_note_exports() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/keep.tex"), "\\label{defn:keep}\n").expect("keep");
    fs::create_dir_all(root.join("projects/p1")).expect("project dir");
    fs::write(root.join("projects/p1/p1.tex"), "\\transclude{keep}\n").expect("project");

    let default_md_dir = root.join("jabberwocky/latex/zettelkasten");
    let project_md_dir = root.join("jabberwocky/latex/asignaturas");
    let legacy_pdf_dir = root.join("jabberwocky/adjuntos/pdf");
    fs::create_dir_all(&default_md_dir).expect("md dir");
    fs::create_dir_all(&project_md_dir).expect("project md dir");
    fs::create_dir_all(&legacy_pdf_dir).expect("legacy pdf dir");
    fs::create_dir_all(root.join("markdown")).expect("legacy markdown dir");
    fs::create_dir_all(root.join("pdf")).expect("pdf dir");

    fs::write(root.join("pdf/keep.pdf"), "keep pdf").expect("keep pdf");
    fs::write(root.join("pdf/orphan.pdf"), "orphan pdf").expect("orphan pdf");
    fs::write(root.join("pdf/orphan-project.pdf"), "orphan project pdf")
        .expect("orphan project pdf");
    fs::write(
        legacy_pdf_dir.join("orphan-legacy.pdf"),
        "legacy orphan pdf",
    )
    .expect("legacy pdf");

    fs::write(default_md_dir.join("keep.md"), "keep md").expect("keep md");
    fs::write(default_md_dir.join("orphan.md"), "orphan md").expect("orphan md");
    fs::write(project_md_dir.join("p1.md"), "project md").expect("project md");
    fs::write(
        project_md_dir.join("orphan-project.md"),
        "orphan project md",
    )
    .expect("orphan project md");
    fs::write(root.join("markdown/orphan.md"), "legacy orphan md").expect("legacy orphan md");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("clean")
        .assert()
        .success()
        .stdout(contains(
            "Resumen de limpieza: 3 pdf(s), 3 markdown(s) eliminado(s)",
        ));

    assert!(root.join("pdf/keep.pdf").exists());
    assert!(!root.join("pdf/orphan.pdf").exists());
    assert!(!root.join("pdf/orphan-project.pdf").exists());
    assert!(!legacy_pdf_dir.join("orphan-legacy.pdf").exists());
    assert!(default_md_dir.join("keep.md").exists());
    assert!(!default_md_dir.join("orphan.md").exists());
    assert!(project_md_dir.join("p1.md").exists());
    assert!(!project_md_dir.join("orphan-project.md").exists());
    assert!(!root.join("markdown/orphan.md").exists());
}

#[test]
fn rename_label_updates_references() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/target.tex"), "\\label{l1}\\n").expect("target");
    fs::write(
        root.join("notes/slipbox/consumer.tex"),
        "\\excref[l1]{target}\\n\\ref{target-l1}\\n\\hyperref[target-l1]{X}\\n",
    )
    .expect("consumer");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("target")
        .write_stdin("\nl2\n")
        .assert()
        .success()
        .stdout(contains(
            "Etiqueta renombrada exitosamente de l1 a l2 en target",
        ));

    let target_content =
        fs::read_to_string(root.join("notes/slipbox/target.tex")).expect("target read");
    assert!(target_content.contains("\\label{l2}"));
    assert!(!target_content.contains("\\label{l1}"));

    let consumer_content =
        fs::read_to_string(root.join("notes/slipbox/consumer.tex")).expect("consumer read");
    assert!(consumer_content.contains("\\excref[l2]{target}"));
    assert!(consumer_content.contains("\\ref{target-l2}"));
    assert!(consumer_content.contains("\\hyperref[target-l2]"));
}

#[test]
fn rename_label_with_colon_updates_references() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    // Create target note with label containing colon (like defn:teoria)
    fs::write(
        root.join("notes/slipbox/teoria-semantica.tex"),
        "\\label{defn:teoria}\n",
    )
    .expect("target");

    // Create consumer note with references using the old label
    fs::write(
        root.join("notes/slipbox/consumer.tex"),
        "\\excref[defn:teoria]{teoria-semantica}\n\\ref{teoria-semantica-defn:teoria}\n\\hyperref[teoria-semantica-defn:teoria]{X}\n",
    )
    .expect("consumer");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("teoria-semantica")
        .write_stdin("\ndefn:teoria-semantica\n")
        .assert()
        .success()
        .stdout(contains(
            "Etiqueta renombrada exitosamente de defn:teoria a defn:teoria-semantica en teoria-semantica",
        ));

    let target_content =
        fs::read_to_string(root.join("notes/slipbox/teoria-semantica.tex")).expect("target read");
    assert!(target_content.contains("\\label{defn:teoria-semantica}"));
    assert!(!target_content.contains("\\label{defn:teoria}"));

    let consumer_content =
        fs::read_to_string(root.join("notes/slipbox/consumer.tex")).expect("consumer read");
    assert!(
        consumer_content.contains("\\excref[defn:teoria-semantica]{teoria-semantica}"),
        "excref not updated correctly. Content: {}",
        consumer_content
    );
    assert!(
        consumer_content.contains("\\ref{teoria-semantica-defn:teoria-semantica}"),
        "ref not updated correctly. Content: {}",
        consumer_content
    );
    assert!(
        consumer_content.contains("\\hyperref[teoria-semantica-defn:teoria-semantica]"),
        "hyperref not updated correctly. Content: {}",
        consumer_content
    );
}

#[test]
fn rename_label_in_project_folder() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    // Create target note
    fs::write(root.join("notes/slipbox/target.tex"), "\\label{defn:key}\n").expect("target");

    // Create project directory with files that reference the note
    fs::create_dir_all(root.join("projects/myproject")).expect("project dir");
    fs::write(
        root.join("projects/myproject/myproject.tex"),
        "\\documentclass{book}\n\\begin{document}\n\\excref[defn:key]{target}\n\\end{document}\n",
    )
    .expect("project");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("target")
        .write_stdin("\ndefn:new-key\n")
        .assert()
        .success()
        .stdout(contains(
            "Etiqueta renombrada exitosamente de defn:key a defn:new-key en target",
        ));

    let target_content =
        fs::read_to_string(root.join("notes/slipbox/target.tex")).expect("target read");
    assert!(target_content.contains("\\label{defn:new-key}"));

    let project_content =
        fs::read_to_string(root.join("projects/myproject/myproject.tex")).expect("project read");
    assert!(
        project_content.contains("\\excref[defn:new-key]{target}"),
        "Project file not updated. Content: {}",
        project_content
    );
}

#[test]
fn rename_label_multiple_refs_same_file() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    // Create notes with multiple references to the same label in one file
    fs::write(
        root.join("notes/slipbox/teoria-semantica.tex"),
        "\\label{defn:teoria}\n",
    )
    .expect("target");

    fs::write(
        root.join("notes/slipbox/consumer.tex"),
        "First ref:\\excref[defn:teoria]{teoria-semantica}\nSecond ref:\\ref{teoria-semantica-defn:teoria}\nThird ref:\\exhyperref[defn:teoria]{teoria-semantica}{text}\n",
    )
    .expect("consumer");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("teoria-semantica")
        .write_stdin("\ndefn:teoria-semantica\n")
        .assert()
        .success();

    let consumer_content =
        fs::read_to_string(root.join("notes/slipbox/consumer.tex")).expect("consumer read");

    eprintln!("Consumer content after rename:\n{}", consumer_content);

    assert!(
        consumer_content.contains("\\excref[defn:teoria-semantica]{teoria-semantica}"),
        "First excref not updated"
    );
    assert!(
        consumer_content.contains("\\ref{teoria-semantica-defn:teoria-semantica}"),
        "ref not updated"
    );
    assert!(
        consumer_content.contains("\\exhyperref[defn:teoria-semantica]{teoria-semantica}"),
        "exhyperref not updated"
    );
}

#[test]
fn rename_label_with_internal_references() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    // Create note with INTERNAL references (without note prefix)
    fs::write(
        root.join("notes/slipbox/theory.tex"),
        "\\label{defn:base}\nSection with first ref:\\ref{defn:base}\nAnother ref:\\hyperref[defn:base]{link}\n",
    )
    .expect("theory");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("theory")
        .write_stdin("\ndefn:extended\n")
        .assert()
        .success();

    let theory_content =
        fs::read_to_string(root.join("notes/slipbox/theory.tex")).expect("theory read");

    eprintln!("Theory content after rename:\n{}", theory_content);

    // Check if internal references (without note prefix) are updated
    assert!(
        !theory_content.contains("\\label{defn:base}"),
        "Label not renamed"
    );
    assert!(
        theory_content.contains("\\label{defn:extended}"),
        "New label not found"
    );

    // This might fail if internal references aren't handled
    assert!(
        !theory_content.contains("\\ref{defn:base}"),
        "Internal ref with old label still present"
    );
    assert!(
        theory_content.contains("\\ref{defn:extended}"),
        "Internal ref not updated to new label"
    );
}

#[test]
fn rename_interactive_renames_note_and_all_labels_in_one_shot() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/alpha.tex"),
        "\\label{defn:a}\n\\label{thm:a}\n",
    )
    .expect("alpha");
    fs::write(
        root.join("notes/slipbox/consumer.tex"),
        "\\excref[defn:a]{alpha}\n\\exhyperref[thm:a]{alpha}{Ver}\n\\ref{alpha-defn:a}\n\\hyperref[alpha-thm:a]{X}\n",
    )
    .expect("consumer");
    fs::write(
        root.join("notes/documents.tex"),
        "\\externaldocument[alpha-]{alpha}\n",
    )
    .expect("documents");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("alpha")
        .write_stdin("beta\ndefn:b\nthm:b\n")
        .assert()
        .success()
        .stdout(contains("Renombrado exitosamente alpha a beta"));

    assert!(!root.join("notes/slipbox/alpha.tex").exists());
    let beta_content = fs::read_to_string(root.join("notes/slipbox/beta.tex")).expect("beta");
    assert!(beta_content.contains("\\label{defn:b}"));
    assert!(beta_content.contains("\\label{thm:b}"));

    let consumer = fs::read_to_string(root.join("notes/slipbox/consumer.tex")).expect("consumer");
    assert!(consumer.contains("\\excref[defn:b]{beta}"));
    assert!(consumer.contains("\\exhyperref[thm:b]{beta}{Ver}"));
    assert!(
        consumer.contains("\\ref{beta-defn:b}"),
        "consumer after interactive rename:\n{}",
        consumer
    );
    assert!(consumer.contains("\\hyperref[beta-thm:b]{X}"));

    let docs = fs::read_to_string(root.join("notes/documents.tex")).expect("docs");
    assert!(docs.contains("\\externaldocument[beta-]{beta}"));
}

#[test]
fn rename_interactive_skips_reserved_note_label() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/logic.tex"),
        "\\currentdoc{note}\n\\label{defn:logic}\n",
    )
    .expect("logic");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("logic")
        .write_stdin("\ndefn:logic-updated\n")
        .assert()
        .success()
        .stdout(contains(
            "Etiqueta renombrada exitosamente de defn:logic a defn:logic-updated en logic",
        ));
}

#[test]
fn rename_interactive_enter_keeps_values() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/note.tex"),
        "\\label{defn:one}\n\\label{defn:two}\n",
    )
    .expect("note");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("note")
        .write_stdin("\n\n\n")
        .assert()
        .success()
        .stdout(contains("No se realizaron cambios"));

    assert!(root.join("notes/slipbox/note.tex").exists());
    let content = fs::read_to_string(root.join("notes/slipbox/note.tex")).expect("note read");
    assert!(content.contains("\\label{defn:one}"));
    assert!(content.contains("\\label{defn:two}"));
}

#[test]
fn remove_note_removes_file_documents_and_db() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/killme.tex"), "\\label{x}\\n").expect("killme");
    fs::write(
        root.join("notes/slipbox/refnote.tex"),
        "\\excref[killme]{x}\\n",
    )
    .expect("refnote");
    fs::create_dir_all(root.join("projects/proj-killme")).expect("project dir");
    fs::write(
        root.join("projects/proj-killme/proj-killme.tex"),
        "\\transclude{killme}\\n",
    )
    .expect("project tex");
    fs::write(
        root.join("notes/documents.tex"),
        "\\externaldocument[killme-]{killme}\n",
    )
    .expect("documents");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("remove_note")
        .arg("killme")
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(contains("La nota 'killme' esta referenciada desde:"))
        .stdout(contains("Nota eliminada killme"));

    assert!(!root.join("notes/slipbox/killme.tex").exists());
    let docs = fs::read_to_string(root.join("notes/documents.tex")).expect("docs");
    assert!(!docs.contains("killme"));

    let conn = Connection::open(root.join("slipbox.db")).expect("db open");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note WHERE filename='killme'",
            [],
            |row| row.get(0),
        )
        .expect("count");
    assert_eq!(count, 0);
}

#[test]
fn addtodocuments_adds_line_once() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd1 = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd1.arg("--workspace-root")
        .arg(root)
        .arg("addtodocuments")
        .arg("alpha")
        .assert()
        .success();

    let mut cmd2 = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd2.arg("--workspace-root")
        .arg(root)
        .arg("addtodocuments")
        .arg("alpha")
        .assert()
        .success();

    let docs = fs::read_to_string(root.join("notes/documents.tex")).expect("docs");
    let count = docs.matches("\\externaldocument[alpha-]{alpha}").count();
    assert_eq!(count, 1);
}

#[test]
fn list_unreferenced_lists_notes_without_incoming_links() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{defn:a}\n").expect("a");
    fs::write(
        root.join("notes/slipbox/b.tex"),
        "\\label{defn:b}\\n\\excref[defn:a]{a}\n",
    )
    .expect("b");
    fs::write(root.join("notes/slipbox/c.tex"), "\\label{defn:c}\n").expect("c");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("list_unreferenced")
        .assert()
        .success()
        .stdout(contains("b"))
        .stdout(contains("c"));
}

#[test]
fn rename_recent_renames_selected_recent_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/older.tex"), "\\label{x}\n").expect("older");
    thread::sleep(Duration::from_millis(20));
    fs::write(root.join("notes/slipbox/newer.tex"), "\\label{y}\n").expect("newer");
    fs::write(
        root.join("notes/documents.tex"),
        "\\externaldocument[older-]{older}\n\\externaldocument[newer-]{newer}\n",
    )
    .expect("documents");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_recent")
        .arg("1")
        .write_stdin("renamed\n")
        .assert()
        .success()
        .stdout(contains("Renombrado exitosamente newer a renamed"));

    assert!(!root.join("notes/slipbox/newer.tex").exists());
    assert!(root.join("notes/slipbox/renamed.tex").exists());

    let docs = fs::read_to_string(root.join("notes/documents.tex")).expect("docs");
    assert!(docs.contains("\\externaldocument[renamed-]{renamed}"));
}

#[test]
fn export_project_expands_transcludes() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/p1")).expect("projects/p1");

    fs::write(
        root.join("notes/slipbox/n1.tex"),
        "start\n%<*note>\nBody completo\n%</note>\n%<*part>\nSolo parte\n%</part>\n",
    )
    .expect("note");
    fs::write(
        root.join("projects/p1/p1.tex"),
        "Intro\n\\transclude{n1}\n\\transclude[part]{n1}\nFin\n",
    )
    .expect("project");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("export_project")
        .arg("p1")
        .assert()
        .success();

    let out = fs::read_to_string(root.join("projects/p1/standalone/p1.tex")).expect("out");
    assert!(out.contains("Intro"));
    assert!(out.contains("Body completo"));
    assert!(out.contains("Solo parte"));
    assert!(out.contains("Fin"));
}

#[test]
fn export_draft_expands_execute_metadata() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("draft")).expect("draft dir");
    fs::create_dir_all(root.join("inputs")).expect("inputs");

    fs::write(
        root.join("notes/slipbox/meta.tex"),
        "X\n%<*note>\nMeta bloque\n%</note>\n",
    )
    .expect("meta");
    fs::write(
        root.join("inputs/in.tex"),
        "A\n\\ExecuteMetaData[notes/slipbox/meta.tex]{note}\nB\n",
    )
    .expect("in");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("export_draft")
        .arg("inputs/in.tex")
        .arg("draft/out.tex")
        .assert()
        .success();

    let out = fs::read_to_string(root.join("draft/out.tex")).expect("out");
    assert!(out.contains("A"));
    assert!(out.contains("Meta bloque"));
    assert!(out.contains("B"));
}

#[test]
fn export_markdown_commands_generate_obsidian_files() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/materias")).expect("projects/materias");

    fs::write(
        root.join("notes/slipbox/note-a.tex"),
        "\\title{Titulo A}\n\\label{defn:a}\n",
    )
    .expect("note-a");
    fs::write(
        root.join("notes/slipbox/note-b.tex"),
        "\\excref[defn:a]{note-a}\nTODO: revisar ejemplo\n",
    )
    .expect("note-b");
    fs::write(
        root.join("projects/materias/materias.tex"),
        "\\title{Curso de Prueba}\n\\transclude{note-a}\n\\transclude{note-b}\n",
    )
    .expect("project tex");

    let mut export_note = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    export_note
        .arg("--workspace-root")
        .arg(root)
        .arg("export_markdown")
        .arg("note-b")
        .assert()
        .success();

    let mut export_project = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    export_project
        .arg("--workspace-root")
        .arg(root)
        .arg("export_markdown")
        .arg("materias")
        .assert()
        .success();

    let note_md =
        fs::read_to_string(root.join("jabberwocky/latex/zettelkasten/note-b.md")).expect("note md");
    assert!(note_md.contains("[[note-b.pdf]]"));
    assert!(note_md.contains("## Referencias"));
    assert!(note_md.contains("[note-a](./note-a.md)"));
    assert!(note_md.contains("#TODO revisar ejemplo"));

    let db = rusqlite::Connection::open(root.join("slipbox.db")).expect("open db");
    let keywords: Vec<(String, String)> = db
        .prepare(
            "SELECT nk.keyword, nk.value FROM note_keyword nk JOIN note n ON n.id = nk.note_id WHERE n.filename = 'note-b'",
        )
        .expect("prepare")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(
        keywords,
        vec![("TODO".to_string(), "revisar ejemplo".to_string())]
    );

    let project_md = fs::read_to_string(root.join("jabberwocky/latex/asignaturas/materias.md"))
        .expect("project md");
    assert!(project_md.contains("[[materias.pdf]]"));
    assert!(project_md.contains("## Notas incluidas"));
    assert!(project_md.contains("[note-a](./note-a.md)"));
    assert!(project_md.contains("[note-b](./note-b.md)"));
}

#[test]
fn list_keywords_lists_notes_and_projects_with_filter() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/materias")).expect("projects/materias");

    fs::write(
        root.join("notes/slipbox/note-a.tex"),
        "\\title{Titulo A}\nTODO: revisar ejemplo\n",
    )
    .expect("note-a");
    fs::write(
        root.join("notes/slipbox/note-b.tex"),
        "\\title{Titulo B}\nFIXME: corregir defecto\n",
    )
    .expect("note-b");
    fs::write(
        root.join("projects/materias/materias.tex"),
        "\\title{Curso}\n\\transclude{note-a}\nTODO: ampliar curso\n",
    )
    .expect("project tex");
    fs::write(
        root.join("projects/materias/capitulo.tex"),
        "Seccion FIXME: pendiente future\n",
    )
    .expect("capitulo tex");

    let run = |args: &[&str]| {
        let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
        cmd.arg("--workspace-root")
            .arg(root)
            .arg("list_keywords")
            .args(args);
        let out = cmd.output().expect("run");
        String::from_utf8_lossy(&out.stdout).to_string()
    };

    // Any keyword, notes and projects (default keywords: TODO and FIXME)
    let all = run(&[]);
    assert!(all.contains("Notas con keyword"));
    assert!(all.contains("#TODO revisar ejemplo"));
    assert!(all.contains("#TODO revisar ejemplo  (note-a.tex:2)"));
    assert!(all.contains("#FIXME corregir defecto"));
    assert!(all.contains("Proyectos con keyword"));
    assert!(all.contains("#TODO ampliar curso"));
    assert!(all.contains("#FIXME pendiente future"));
    assert!(all.contains("#TODO ampliar curso  (materias.tex:3)"));
    assert!(all.contains("#FIXME pendiente future  (capitulo.tex:1)"));
    assert!(all.contains("#FIXME corregir defecto  (note-b.tex:2)"));

    // Filter by TODO
    let todo = run(&["TODO"]);
    assert!(todo.contains("#TODO revisar ejemplo"));
    assert!(todo.contains("#TODO ampliar curso"));
    assert!(!todo.contains("#FIXME corregir defecto"));

    // Notes only
    let notes_only = run(&["--notes"]);
    assert!(notes_only.contains("Notas con keyword"));
    assert!(notes_only.contains("#TODO revisar ejemplo"));
    assert!(!notes_only.contains("Proyectos con keyword"));
    assert!(!notes_only.contains("#TODO ampliar curso"));

    // Projects only
    let projects_only = run(&["--projects"]);
    assert!(projects_only.contains("Proyectos con keyword"));
    assert!(projects_only.contains("#TODO ampliar curso"));
    assert!(!projects_only.contains("Notas con keyword"));
    assert!(!projects_only.contains("#TODO revisar ejemplo"));
}

#[test]
fn export_markdown_frontmatter_includes_db_metadata() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/fp")).expect("projects/fp");

    fs::write(
        root.join("notes/slipbox/note-a.tex"),
        "\\title{Titulo A}\n\\label{defn:a}\n",
    )
    .expect("note-a");
    fs::write(
        root.join("notes/slipbox/note-b.tex"),
        "\\excref[defn:a]{note-a}\n\\cite{key:x}\n",
    )
    .expect("note-b");
    fs::write(
        root.join("projects/fp/fp.tex"),
        "\\title{Proyecto FP}\n\\transclude{note-a}\n\\transclude{note-b}\n",
    )
    .expect("fp tex");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("export_all_markdown")
        .assert()
        .success();

    let note_a = fs::read_to_string(root.join("jabberwocky/latex/zettelkasten/note-a.md"))
        .expect("note-a md");
    assert!(note_a.contains("title: 'Titulo A'"));
    assert!(note_a.contains("filename: 'note-a'"));
    assert!(note_a.contains("created: '"));
    assert!(note_a.contains("last_edit_date: '"));
    assert!(note_a.contains("labels:\n  - defn:a"));
    assert!(note_a.contains("backlinks:\n  - note-b"));
    assert!(note_a.contains("projects:\n  - fp"));

    let note_b = fs::read_to_string(root.join("jabberwocky/latex/zettelkasten/note-b.md"))
        .expect("note-b md");
    assert!(note_b.contains("references:\n  - note-a"));
    assert!(note_b.contains("citations:\n  - key:x"));

    let proj = fs::read_to_string(root.join("jabberwocky/latex/asignaturas/fp.md")).expect("fp md");
    assert!(proj.contains("title: 'Proyecto FP'"));
    assert!(proj.contains("name: 'fp'"));
    assert!(proj.contains("created: '"));
    assert!(proj.contains("last_edit_date: '"));
    assert!(proj.contains("inclusions:\n  - note-a\n  - note-b"));
}

#[test]
fn export_all_markdown_generates_all_files() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/pall")).expect("projects/pall");

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{la}\n").expect("a");
    fs::write(root.join("notes/slipbox/b.tex"), "\\label{lb}\n").expect("b");
    fs::write(
        root.join("projects/pall/pall.tex"),
        "\\transclude{a}\n\\transclude{b}\n",
    )
    .expect("pall tex");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("export_all_markdown")
        .assert()
        .success();

    assert!(root.join("jabberwocky/latex/zettelkasten/a.md").exists());
    assert!(root.join("jabberwocky/latex/zettelkasten/b.md").exists());
    assert!(root.join("jabberwocky/latex/asignaturas/pall.md").exists());
}

#[test]
fn newnote_fails_on_duplicate_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut first = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    first
        .arg("--workspace-root")
        .arg(root)
        .arg("newnote")
        .arg("dup")
        .assert()
        .success();

    let mut second = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    second
        .arg("--workspace-root")
        .arg(root)
        .arg("newnote")
        .arg("dup")
        .assert()
        .failure()
        .stderr(contains("Ya existe una nota con nombre"));
}

#[test]
fn newproject_fails_on_duplicate_project() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut first = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    first
        .arg("--workspace-root")
        .arg(root)
        .arg("newproject")
        .arg("dup_project")
        .assert()
        .success();

    let mut second = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    second
        .arg("--workspace-root")
        .arg(root)
        .arg("newproject")
        .arg("dup_project")
        .assert()
        .failure()
        .stderr(contains("Ya existe un proyecto con nombre"));
}

#[test]
fn rename_file_fails_for_missing_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_note")
        .arg("missing")
        .assert()
        .failure()
        .stderr(contains("no encontrada en la base de datos"));
}

#[test]
fn export_project_fails_when_main_file_missing() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/empty_project")).expect("projects/empty_project");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("export_project")
        .arg("empty_project")
        .assert()
        .failure()
        .stderr(contains("Archivo de proyecto no encontrado"));
}

#[test]
fn export_draft_fails_when_input_missing() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("export_draft")
        .arg("missing/in.tex")
        .arg("out.tex")
        .assert()
        .failure()
        .stderr(contains("Archivo de entrada no encontrado"));
}

#[test]
fn export_markdown_fails_when_note_missing_in_db() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("export_markdown")
        .arg("ghost")
        .assert()
        .failure()
        .stderr(contains("No existe nota ni proyecto con nombre 'ghost'"));
}

#[test]
fn render_and_biber_commands_invoke_external_tools() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/rp")).expect("projects/rp");

    fs::write(
        root.join("notes/slipbox/nr.tex"),
        "\\label{a}\n\\cite{key:a}\n",
    )
    .expect("nr");
    fs::write(
        root.join("projects/rp/rp.tex"),
        "\\chapter{X}\n\\cite{key:p}\n",
    )
    .expect("rp");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("tool.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut render_note = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_note
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("nr")
        .arg("--format")
        .arg("pdf")
        .arg("--biber")
        .assert()
        .success();

    let mut render_project = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_project
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("rp")
        .assert()
        .success();

    let logs_before_manual = fs::read_to_string(&log).expect("read log");

    let mut run_biber = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    run_biber
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("biber")
        .arg("nr")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read log");
    let pdflatex_passes_for = |jobname: &str| {
        logs.lines()
            .filter(|l| l.starts_with("pdflatex ") && l.contains(&format!("--jobname={jobname}")))
            .count()
    };
    let biber_calls_for = |jobname: &str| {
        logs.lines()
            .filter(|l| l.starts_with("biber ") && l.ends_with(jobname))
            .count()
    };

    // Note with citations: pdflatex -> biber -> pdflatex -> pdflatex (3 passes).
    assert_eq!(pdflatex_passes_for("nr"), 3);
    // One biber call from the render plus one manual `zetteltex biber nr`.
    assert_eq!(biber_calls_for("nr"), 2);

    // Project with citations: 3 passes and one biber call.
    assert_eq!(pdflatex_passes_for("rp"), 3);
    assert_eq!(biber_calls_for("rp"), 1);

    // biber must run between pdflatex passes of the same note (manual call
    // happens afterwards).
    let order: Vec<String> = logs_before_manual
        .lines()
        .filter(|l| {
            (l.starts_with("pdflatex ") && l.contains("--jobname=nr"))
                || (l.starts_with("biber ") && l.ends_with("nr"))
        })
        .map(|l| l.to_string())
        .collect();
    assert_eq!(order.len(), 4);
    assert!(order[0].starts_with("pdflatex"));
    assert!(order[1].starts_with("biber ") && order[1].ends_with("nr"));
    assert!(order[2].starts_with("pdflatex"));
    assert!(order[3].starts_with("pdflatex"));
}

#[test]
fn render_pdf_without_citations_runs_two_passes() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/nc.tex"), "\\label{x}\n\\ref{x}\n").expect("nc");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("tool-nocite.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut render_note = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_note
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("nc")
        .arg("--format")
        .arg("pdf")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read log");
    let passes = logs
        .lines()
        .filter(|l| l.starts_with("pdflatex ") && l.contains("--jobname=nc"))
        .count();
    assert_eq!(
        passes, 2,
        "note without citations must run exactly 2 pdflatex passes"
    );
    assert!(
        !logs.lines().any(|l| l.starts_with("biber ")),
        "biber must not run for notes without citations"
    );
}

#[test]
fn render_html_invokes_make4ht_and_biber() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/rp")).expect("projects/rp");

    fs::write(root.join("notes/slipbox/nr.tex"), "\\label{a}\n").expect("nr");
    fs::write(root.join("projects/rp/rp.tex"), "\\chapter{X}\n").expect("rp");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("tool-html.log");
    install_fake_tool(&fake_bin, "make4ht", &log);
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut render_note = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_note
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("nr")
        .arg("--format")
        .arg("html")
        .arg("--biber")
        .assert()
        .success();

    let mut render_project = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_project
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("rp")
        .arg("--format")
        .arg("html")
        .arg("--biber")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read log");
    assert!(logs.contains("make4ht --format html5+svg"));
    assert!(logs.contains("--jobname nr"));
    assert!(logs.contains("--jobname rp"));
    assert!(logs.contains(".zetteltex-render-nr.html.tex"));
    assert!(logs_contain_biber_for(&logs, "nr"));
    assert!(logs_contain_biber_for(&logs, "rp"));
}

#[test]
fn render_note_adds_referenced_in_only_to_temporary_tex() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/target.tex"),
        "\\documentclass{texnote}\n\\begin{document}\n\\currentdoc{note}\nContenido\n\\end{document}\n",
    )
    .expect("target note");
    fs::write(
        root.join("notes/slipbox/source_a.tex"),
        "\\title{Titulo A}\n\\excref[defn:x]{target}\n",
    )
    .expect("source a");
    fs::write(
        root.join("notes/slipbox/source_b.tex"),
        "\\title{Titulo B}\n\\excref{target}\n",
    )
    .expect("source b");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("render-note-referenced.log");

    let pdflatex_script = format!(
        "#!/bin/sh\n\
echo \"pdflatex $@\" >> \"{}\"\n\
last=\"\"\n\
for arg in \"$@\"; do last=\"$arg\"; done\n\
echo \"---BEGIN-SOURCE---\" >> \"{}\"\n\
cat \"$last\" >> \"{}\"\n\
echo \"---END-SOURCE---\" >> \"{}\"\n\
exit 0\n",
        log.display(),
        log.display(),
        log.display(),
        log.display()
    );
    let pdflatex_path = fake_bin.join("pdflatex");
    fs::write(&pdflatex_path, pdflatex_script).expect("write fake pdflatex");
    let mut perms = fs::metadata(&pdflatex_path).expect("meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&pdflatex_path, perms).expect("chmod");

    let path_env = prepend_path(&fake_bin);
    let mut render_note = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_note
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("target")
        .assert()
        .success();

    let original_target =
        fs::read_to_string(root.join("notes/slipbox/target.tex")).expect("target");
    assert!(!original_target.contains("Referenciado en"));

    let logs = fs::read_to_string(&log).expect("read log");
    assert!(logs.contains(".zetteltex-render-target.input"));
    assert!(logs.contains("\\section*{Referenciado en}"));
    assert!(logs.contains("\\item \\hyperref[source_a-note]{Titulo A}"));
    assert!(logs.contains("\\item \\hyperref[source_b-note]{Titulo B}"));

    // The referencing notes must be pre-rendered (raw, single pass) so their
    // .aux exists for the backlinks of the target note.
    assert!(
        logs.contains("--jobname=source_a"),
        "source_a must be pre-rendered"
    );
    assert!(
        logs.contains("--jobname=source_b"),
        "source_b must be pre-rendered"
    );

    let temp_tex = root.join("notes/slipbox/.zetteltex-render-target.input");
    assert!(!temp_tex.exists());
}

#[test]
fn render_note_ensures_referencing_sources_before_target() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    // a <-> b mutual references: a cycle that must not hang or recurse.
    fs::write(
        root.join("notes/slipbox/a.tex"),
        "\\label{defn:a}\n\\excref[defn:b]{b}\n",
    )
    .expect("a");
    fs::write(
        root.join("notes/slipbox/b.tex"),
        "\\label{defn:b}\n\\excref[defn:a]{a}\n",
    )
    .expect("b");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("ensure-cycle.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    let path_env = prepend_path(&fake_bin);

    let mut render_note = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_note
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("a")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read log");
    let a_passes = logs
        .lines()
        .filter(|l| l.starts_with("pdflatex ") && l.contains("--jobname=a"))
        .count();
    let b_passes = logs
        .lines()
        .filter(|l| l.starts_with("pdflatex ") && l.contains("--jobname=b"))
        .count();

    // `b` is pre-rendered exactly once (raw, to provide its .aux) and `a` gets
    // its two normal passes. The pre-render of `b` must come first.
    assert_eq!(a_passes, 2);
    assert_eq!(b_passes, 1);
    let first_a = logs
        .lines()
        .position(|l| l.starts_with("pdflatex ") && l.contains("--jobname=a"))
        .expect("a pass must exist");
    let first_b = logs
        .lines()
        .position(|l| l.starts_with("pdflatex ") && l.contains("--jobname=b"))
        .expect("b pre-render must exist");
    assert!(
        first_b < first_a,
        "b must be pre-rendered before a is compiled"
    );
}

#[test]
fn render_all_commands_invoke_batch_tools() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/pbatch")).expect("projects/pbatch");

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{a}\n\\cite{ka}\n").expect("a");
    fs::write(root.join("notes/slipbox/b.tex"), "\\label{b}\n").expect("b");
    fs::write(root.join("projects/pbatch/pbatch.tex"), "\\chapter{Y}\n").expect("project");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("tool-batch.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut render_all = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_all
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render_all")
        .assert()
        .success();

    let mut render_all_projects = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_all_projects
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render_all")
        .arg("--projects")
        .assert()
        .success();

    let mut render_updates = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_updates
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render_updates")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read log");
    assert!(logs.contains("pdflatex"));
    assert!(logs.contains("--jobname=a"));
    assert!(logs.contains("--jobname=b"));
    assert!(logs.contains("--jobname=pbatch"));
}

#[test]
fn render_updates_renders_only_stale_items() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/p-stale")).expect("projects/p-stale");
    fs::create_dir_all(root.join("projects/p-fresh")).expect("projects/p-fresh");

    fs::write(
        root.join("notes/slipbox/stale.tex"),
        "\\label{st}\n\\cite{k}\n",
    )
    .expect("stale note");
    fs::write(root.join("notes/slipbox/fresh.tex"), "\\label{fr}\n").expect("fresh note");
    fs::write(
        root.join("projects/p-stale/p-stale.tex"),
        "\\chapter{Stale}\n",
    )
    .expect("p-stale");
    fs::write(
        root.join("projects/p-fresh/p-fresh.tex"),
        "\\chapter{Fresh}\n",
    )
    .expect("p-fresh");

    let mut sync_cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("synchronize")
        .assert()
        .success();

    let conn = Connection::open(root.join("slipbox.db")).expect("open db");
    conn.execute(
        "UPDATE note SET last_build_date_pdf='1900-01-01T00:00:00+00:00' WHERE filename='stale'",
        [],
    )
    .expect("mark stale note");
    conn.execute(
        "UPDATE note SET last_build_date_pdf='9999-01-01T00:00:00+00:00' WHERE filename='fresh'",
        [],
    )
    .expect("mark fresh note");
    conn.execute(
        "UPDATE project SET last_build_date_pdf='1900-01-01T00:00:00+00:00' WHERE name='p-stale'",
        [],
    )
    .expect("mark stale project");
    conn.execute(
        "UPDATE project SET last_build_date_pdf='9999-01-01T00:00:00+00:00' WHERE name='p-fresh'",
        [],
    )
    .expect("mark fresh project");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("tool-updates.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render_updates")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read updates log");
    assert!(logs.contains("--jobname=stale"));
    assert!(!logs.contains("--jobname=fresh"));
    assert!(logs.contains("--jobname=p-stale"));
    assert!(!logs.contains("--jobname=p-fresh"));
}

#[test]
fn watch_recompiles_target_when_file_changes() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    let note_path = root.join("notes/slipbox/watchable.tex");
    fs::write(&note_path, "\\label{defn:watchable}\n").expect("note");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("tool-watch.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    // Spawn `watch`; it renders once up front, then recompiles on changes.
    let bin = assert_cmd::cargo::cargo_bin!("zetteltex");
    let mut child = std::process::Command::new(bin)
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("watch")
        .arg("watchable")
        .arg("--format")
        .arg("pdf")
        .arg("--poll")
        .arg("100")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn watch");

    // Wait for the initial render triggered by watch (fresh note = pending).
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let initial_count = loop {
        let n = count_pdflatex_jobnames(&log, "watchable");
        if n >= 2 || std::time::Instant::now() > deadline {
            break n;
        }
        thread::sleep(Duration::from_millis(100));
    };
    assert!(
        initial_count >= 2,
        "initial render should run >= 2 pdflatex passes"
    );

    // Touch the note and expect watch to recompile it.
    fs::write(&note_path, "\\label{defn:watchable}\n% edit\n").expect("rewrite note");

    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut final_count = initial_count;
    while final_count <= initial_count {
        thread::sleep(Duration::from_millis(150));
        final_count = count_pdflatex_jobnames(&log, "watchable");
        if std::time::Instant::now() > deadline {
            break;
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        final_count >= initial_count + 2,
        "edit should trigger a fresh 2-pass render (was {initial_count}, now {final_count})"
    );
}

fn count_pdflatex_jobnames(log: &Path, jobname: &str) -> usize {
    fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .filter(|l| l.starts_with("pdflatex ") && l.contains(&format!("--jobname={jobname}")))
        .count()
}

#[test]
fn force_synchronize_notes_updates_note_db_state() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{defn:a}\n").expect("note a");
    fs::write(
        root.join("notes/slipbox/b.tex"),
        "\\excref[defn:a]{a}\n\\cite{key:b}\n",
    )
    .expect("note b");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("force_synchronize")
        .arg("--notes")
        .assert()
        .success()
        .stdout(contains("Fuerza sincronizacion de notas:"));

    let conn = Connection::open(root.join("slipbox.db")).expect("open db");
    let notes_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM note", [], |row| row.get(0))
        .expect("note count");
    assert_eq!(notes_count, 2);
}

#[test]
fn force_synchronize_projects_updates_project_inclusions() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/curso")).expect("projects/curso");

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{defn:a}\n").expect("note a");
    fs::write(root.join("projects/curso/curso.tex"), "\\transclude{a}\n").expect("project main");

    let mut sync_notes = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    sync_notes
        .arg("--workspace-root")
        .arg(root)
        .arg("force_synchronize")
        .arg("--notes")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("force_synchronize")
        .arg("--projects")
        .assert()
        .success()
        .stdout(contains("Fuerza sincronizacion de proyectos:"));

    let conn = Connection::open(root.join("slipbox.db")).expect("open db");
    let inclusion_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM inclusion", [], |row| row.get(0))
        .expect("inclusion count");
    assert_eq!(inclusion_count, 1);
}

#[test]
fn force_synchronize_runs_both_notes_and_projects() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/alg")).expect("projects/alg");

    fs::write(root.join("notes/slipbox/n.tex"), "\\label{ln}\n").expect("note n");
    fs::write(root.join("projects/alg/alg.tex"), "\\transclude{n}\n").expect("project alg");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("force_synchronize")
        .assert()
        .success()
        .stdout(contains("Fuerza sincronizacion de proyectos:"))
        .stdout(contains("Fuerza sincronizacion de notas:"));

    let conn = Connection::open(root.join("slipbox.db")).expect("open db");
    let projects_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM project", [], |row| row.get(0))
        .expect("project count");
    assert_eq!(projects_count, 1);
}

#[test]
fn render_all_defaults_to_pdf_pipeline() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{a}\n\\cite{k}\n").expect("note a");
    fs::write(root.join("notes/slipbox/b.tex"), "\\label{b}\n").expect("note b");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("render-all-pdf.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render_all")
        .arg("--format")
        .arg("pdf")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read render_all pdf log");
    assert!(logs.contains("--jobname=a"));
    assert!(logs.contains("--jobname=b"));
}

#[test]
fn biber_auto_detects_project_by_name() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/proyecto-demo")).expect("projects/proyecto-demo");
    fs::write(
        root.join("projects/proyecto-demo/proyecto-demo.tex"),
        "\\chapter{X}\n",
    )
    .expect("proyecto-demo main tex");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("biber-project.log");
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("biber")
        .arg("proyecto-demo")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read biber project log");
    assert!(logs_contain_biber_for(&logs, "proyecto-demo"));
}

#[test]
fn render_ambiguity_between_note_and_project_requires_flag() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/dual.tex"), "\\label{a}\n").expect("dual note");
    fs::create_dir_all(root.join("projects/dual")).expect("dual project dir");
    fs::write(root.join("projects/dual/dual.tex"), "\\chapter{X}\n").expect("dual project");

    let mut ambiguous = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    ambiguous
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("dual")
        .assert()
        .failure()
        .stderr(contains("existe como nota y como proyecto"));

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("render-ambiguous.log");
    install_fake_tool(&fake_bin, "pdflatex", &log);
    let path_env = prepend_path(&fake_bin);

    let mut with_flag = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    with_flag
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("dual")
        .arg("--project")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read log");
    assert!(logs.contains("--jobname=dual"));
}

#[test]
fn render_fails_when_pdflatex_missing() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/n1.tex"), "\\label{a}\n").expect("n1");

    let empty_bin = root.join("empty-bin");
    fs::create_dir_all(&empty_bin).expect("empty bin");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", empty_bin.display().to_string())
        .arg("--workspace-root")
        .arg(root)
        .arg("render")
        .arg("n1")
        .assert()
        .failure()
        .stderr(contains("pdflatex no encontrado en PATH"));
}

#[test]
fn biber_fails_when_biber_missing() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/n1.tex"), "\\label{a}\n").expect("n1");

    let empty_bin = root.join("empty-bin");
    fs::create_dir_all(&empty_bin).expect("empty bin");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", empty_bin.display().to_string())
        .arg("--workspace-root")
        .arg(root)
        .arg("biber")
        .arg("n1")
        .assert()
        .failure()
        .stderr(contains("biber no encontrado en PATH"));
}

#[test]
fn remove_duplicate_citations_removes_db_duplicates() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let conn = Connection::open(root.join("slipbox.db")).expect("open db");
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS note (
            id INTEGER PRIMARY KEY,
            filename TEXT NOT NULL UNIQUE,
            created TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS citation (
            id INTEGER PRIMARY KEY,
            note_id INTEGER NOT NULL,
            citationkey TEXT NOT NULL
        );
        "#,
    )
    .expect("schema");
    conn.execute(
        "INSERT INTO note (id, filename, created) VALUES (1, 'n1', '2026-01-01')",
        [],
    )
    .expect("insert note");
    conn.execute(
        "INSERT INTO citation (note_id, citationkey) VALUES (1, 'dup-key')",
        [],
    )
    .expect("insert c1");
    conn.execute(
        "INSERT INTO citation (note_id, citationkey) VALUES (1, 'dup-key')",
        [],
    )
    .expect("insert c2");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("remove_duplicate_citations")
        .assert()
        .success()
        .stdout(contains("Eliminada(s) 1 cita(s) duplicada(s)"));

    let remaining: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM citation WHERE citationkey='dup-key'",
            [],
            |row| row.get(0),
        )
        .expect("remaining");
    assert_eq!(remaining, 1);
}

#[test]
fn edit_command_opens_note_in_editor() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::write(root.join("notes/slipbox/openme.tex"), "\\label{o}\n").expect("openme");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("editor.log");
    install_fake_tool(&fake_bin, "code", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("edit")
        .arg("openme")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read log");
    assert!(logs.contains("openme.tex"));
}

#[test]
fn edit_without_name_opens_most_recent_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/older.tex"), "\\label{a}\n").expect("older");
    thread::sleep(Duration::from_millis(20));
    fs::write(root.join("notes/slipbox/newer.tex"), "\\label{b}\n").expect("newer");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("editor-noarg.log");
    install_fake_tool(&fake_bin, "code", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("edit")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read noarg edit log");
    assert!(logs.contains("newer.tex"));
}

#[test]
fn edit_fails_when_note_does_not_exist() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("edit")
        .arg("ghost")
        .assert()
        .failure()
        .stderr(contains("No existe nota ni proyecto con nombre 'ghost'"));
}

#[test]
fn edit_with_project_flag_opens_project_tex() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/myproj")).expect("project dir");
    fs::write(
        root.join("projects/myproj/myproj.tex"),
        "\\input{notes/slipbox/notes}\n",
    )
    .expect("project tex");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("editor-project.log");
    install_fake_tool(&fake_bin, "code", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("edit")
        .arg("myproj")
        .arg("--project")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read project log");
    assert!(
        logs.contains("projects/myproj/myproj.tex"),
        "el editor debe recibir el .tex del proyecto; log: {logs}"
    );
}

#[test]
fn edit_detects_project_without_flag_when_no_note_has_that_name() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::create_dir_all(root.join("projects/soloproj")).expect("project dir");
    fs::write(
        root.join("projects/soloproj/soloproj.tex"),
        "\\input{notes/slipbox/notes}\n",
    )
    .expect("project tex");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("editor-soloproj.log");
    install_fake_tool(&fake_bin, "code", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("edit")
        .arg("soloproj")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read soloproj log");
    assert!(
        logs.contains("projects/soloproj/soloproj.tex"),
        "debe abrir el proyecto aunque no se pase --project; log: {logs}"
    );
}

#[test]
fn edit_fails_when_project_flag_used_without_name() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("edit")
        .arg("--project")
        .assert()
        .failure()
        .stderr(contains(
            "Debes indicar el nombre del proyecto cuando usas --project",
        ));
}

#[test]
fn fuzzy_default_uses_terminal_launcher() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let alacritty_log = root.join("alacritty-launch.log");
    let xterm_log = root.join("x-terminal-launch.log");
    install_fake_tool(&fake_bin, "alacritty", &alacritty_log);
    install_fake_tool(&fake_bin, "x-terminal-emulator", &xterm_log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .assert()
        .success();

    let logs = fs::read_to_string(&alacritty_log).expect("read alacritty launch log");
    assert!(logs.contains("alacritty"));
    assert!(logs.contains("fuzzy"));
    assert!(!logs.contains("--inline"));
    assert!(!xterm_log.exists());
}

#[test]
fn fuzzy_inline_runs_native_index_and_search() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/analisis.tex"),
        "\\label{defn:analisis}\\n\\cite{key:a}\\n",
    )
    .expect("write analisis note");
    fs::write(
        root.join("notes/slipbox/topologia.tex"),
        "\\label{defn:topologia}\\n",
    )
    .expect("write topologia note");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .write_stdin("analisis\n\n")
        .assert()
        .success()
        .stdout(contains("motor nativo Rust"))
        .stdout(contains("analisis"));
}

#[test]
fn fuzzy_inline_reports_empty_index_when_no_items() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .assert()
        .success()
        .stdout(contains("No hay notas ni proyectos para fuzzy"));
}

#[test]
fn fuzzy_scripted_copy_exhyperref_updates_clipboard_and_history() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/nota-a.tex"),
        "\\label{defn:nota-a}\\n",
    )
    .expect("write note");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("xclip.log");
    install_fake_tool(&fake_bin, "xclip", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .arg("--action")
        .arg("copy-exhyperref")
        .arg("--item")
        .arg("nota-a")
        .assert()
        .success();

    let history = fs::read_to_string(root.join(".fuzzy_state.json")).expect("history state");
    assert!(history.contains("\"nota-a\""));

    let logs = fs::read_to_string(&log).expect("xclip log");
    assert!(logs.contains("xclip -selection clipboard"));
}

#[test]
fn fuzzy_scripted_copy_transclude_updates_clipboard_and_history() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/nota-a.tex"),
        "\\label{defn:nota-a}\\n",
    )
    .expect("write note");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("xclip.log");
    let clipboard_contents = root.join("clipboard.txt");
    fs::write(
        fake_bin.join("xclip"),
        format!(
            "#!/bin/sh\necho \"xclip $@\" >> \"{}\"\ncat > \"{}\"\nexit 0\n",
            log.display(),
            clipboard_contents.display()
        ),
    )
    .expect("write fake xclip");
    let mut perms = fs::metadata(fake_bin.join("xclip"))
        .expect("meta")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(fake_bin.join("xclip"), perms).expect("chmod xclip");
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .arg("--action")
        .arg("copy-transclude")
        .arg("--item")
        .arg("nota-a")
        .assert()
        .success();

    let history = fs::read_to_string(root.join(".fuzzy_state.json")).expect("history state");
    assert!(history.contains("\"nota-a\""));

    let logs = fs::read_to_string(&log).expect("xclip log");
    assert!(logs.contains("xclip -selection clipboard"));

    let clipboard = fs::read_to_string(&clipboard_contents).expect("clipboard contents");
    assert_eq!(clipboard, "\\transclude{nota-a}");
}

#[test]
fn fuzzy_scripted_open_editor_for_project_opens_project_root_workspace() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::create_dir_all(root.join("projects/algebra")).expect("projects/algebra");
    fs::write(
        root.join("projects/algebra/algebra.tex"),
        "\\chapter{Algebra}\\n",
    )
    .expect("write project main");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let editor_log = root.join("editor-project.log");
    install_fake_tool(&fake_bin, "code", &editor_log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .arg("--action")
        .arg("open-editor")
        .arg("--item")
        .arg("algebra")
        .assert()
        .success();

    let logs = fs::read_to_string(&editor_log).expect("read editor project log");
    let project_root = root.join("projects/algebra");

    assert!(logs.contains(&project_root.display().to_string()));
}

#[test]
fn fuzzy_scripted_open_pdf_uses_qpdfview_unique_mode() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(
        root.join("notes/slipbox/nota-a.tex"),
        "\\label{defn:nota-a}\\n",
    )
    .expect("write note");
    fs::create_dir_all(root.join("pdf")).expect("pdf dir");
    fs::write(root.join("pdf/nota-a.pdf"), b"%PDF-1.4\n").expect("write pdf");

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let qpdfview_log = root.join("qpdfview.log");
    install_fake_tool(&fake_bin, "qpdfview", &qpdfview_log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .arg("--action")
        .arg("open-pdf")
        .arg("--item")
        .arg("nota-a")
        .assert()
        .success();

    let logs = fs::read_to_string(&qpdfview_log).expect("qpdfview log");
    assert!(logs.contains("qpdfview --unique "));
    assert!(logs.contains("pdf/nota-a.pdf"));
}

#[test]
fn fuzzy_scripted_create_from_query_creates_note_and_documents_entry() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let editor_log = root.join("editor.log");
    install_fake_tool(&fake_bin, "code", &editor_log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .arg("--action")
        .arg("create-from-query")
        .arg("--query")
        .arg("mi nota")
        .assert()
        .success();

    assert!(root.join("notes/slipbox/mi-nota.tex").exists());
    let docs = fs::read_to_string(root.join("notes/documents.tex")).expect("documents");
    assert!(docs.contains("\\externaldocument[mi-nota-]{mi-nota}"));
}

#[test]
fn fuzzy_scripted_create_from_clipboard_injects_content_and_copies_transclude() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let xclip_log = root.join("xclip.log");
    let editor_log = root.join("editor.log");
    install_fake_tool(&fake_bin, "xclip", &xclip_log);
    install_fake_tool(&fake_bin, "code", &editor_log);
    let path_env = prepend_path(&fake_bin);

    let clipboard_text = "\\label{defn:compacto-secuencial}\\nContenido desde clipboard";

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("fuzzy")
        .arg("--inline")
        .arg("--action")
        .arg("create-from-clipboard")
        .arg("--clipboard-text")
        .arg(clipboard_text)
        .assert()
        .success();

    let note_path = root.join("notes/slipbox/compacto-secuencial.tex");
    assert!(note_path.exists());
    let content = fs::read_to_string(note_path).expect("new note");
    assert!(content.contains("Contenido desde clipboard"));

    let xclip_logs = fs::read_to_string(&xclip_log).expect("xclip log");
    assert!(xclip_logs.contains("xclip -selection clipboard"));
}

fn setup_minimal_workspace(root: &Path) {
    fs::create_dir_all(root.join("notes/slipbox")).expect("notes/slipbox");
    fs::create_dir_all(root.join("projects")).expect("projects");
    fs::create_dir_all(root.join("template")).expect("template");
}

#[test]
fn init_config_in_es_switches_prompts_and_writes_spanish_only_comments() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_minimal_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    let assert = cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("init_config")
        .write_stdin("es\n\n\n\n\n\n\n\n\n\n\n")
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(
        stdout.contains("Editor preferido"),
        "los prompts posteriores a elegir es deben ser en espanol; stdout: {stdout}"
    );

    let config = fs::read_to_string(root.join("zetteltex.toml")).expect("config read");
    assert!(config.contains("# Configuración de ZettelTeX"));
    assert!(config.contains("# Idioma de la interfaz: es o en"));
    assert!(!config.contains("Interface language: es or en"));
    assert!(!config.contains("# ZettelTeX configuration"));
    assert!(config.contains("lang = \"es\""));
    assert!(config.contains("author = \"\""));
}

#[test]
fn init_config_default_writes_english_only_comments() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_minimal_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    let assert = cmd
        .arg("--workspace-root")
        .arg(root)
        .arg("init_config")
        .write_stdin("\n\n\n\n\n\n\n\n\n\n\n")
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(
        stdout.contains("Preferred editor"),
        "el idioma por defecto debe ser ingles; stdout: {stdout}"
    );

    let config = fs::read_to_string(root.join("zetteltex.toml")).expect("config read");
    assert!(config.contains("# ZettelTeX configuration"));
    assert!(config.contains("# Interface language: es or en"));
    assert!(!config.contains("# Configuración de ZettelTeX"));
    assert!(config.contains("lang = \"en\""));
    assert!(config.contains("author = \"\""));
}

#[test]
fn newnote_and_newproject_apply_configured_author() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::write(
        root.join("template/note.tex"),
        "\\documentclass{texnote}\n\\title{Note Title}\n\\author{Hugo Marquerie}\n\\date{\\today}\n\\begin{document}\n\\end{document}\n",
    )
    .expect("template note");
    fs::write(
        root.join("template/project.tex"),
        "\\documentclass{texbook}\n\\title{Titulo}\n\\author{Hugo Marquerie}\n\\date{fecha}\n\\begin{document}\n\\end{document}\n",
    )
    .expect("template project");
    fs::write(
        root.join("zetteltex.toml"),
        "[general]\nlang = \"es\"\nauthor = \"Ada Lovelace\"\neditor = \"code\"\n",
    )
    .expect("zetteltex.toml");

    let mut newnote = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    newnote
        .arg("--workspace-root")
        .arg(root)
        .arg("newnote")
        .arg("mi_nota")
        .assert()
        .success();

    let note_content =
        fs::read_to_string(root.join("notes/slipbox/mi_nota.tex")).expect("note tex");
    assert!(
        note_content.contains("\\author{Ada Lovelace}"),
        "el autor configurado debe aplicarse a la nota; contenido: {note_content}"
    );

    let mut newproject = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    newproject
        .arg("--workspace-root")
        .arg(root)
        .arg("newproject")
        .arg("teoria-de-grafos")
        .assert()
        .success();

    let project_content =
        fs::read_to_string(root.join("projects/teoria-de-grafos/teoria-de-grafos.tex"))
            .expect("project tex");
    assert!(
        project_content.contains("\\author{Ada Lovelace}"),
        "el autor configurado debe aplicarse al proyecto; contenido: {project_content}"
    );
}

#[test]
fn newnote_without_author_keeps_template_author() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);
    fs::write(
        root.join("template/note.tex"),
        "\\documentclass{texnote}\n\\title{Note Title}\n\\author{Hugo Marquerie}\n\\date{\\today}\n\\begin{document}\n\\end{document}\n",
    )
    .expect("template note");

    let mut newnote = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    newnote
        .arg("--workspace-root")
        .arg(root)
        .arg("newnote")
        .arg("otra_nota")
        .assert()
        .success();

    let note_content =
        fs::read_to_string(root.join("notes/slipbox/otra_nota.tex")).expect("note tex");
    assert!(
        note_content.contains("\\author{Hugo Marquerie}"),
        "sin autor configurado se conserva el de la plantilla; contenido: {note_content}"
    );
}

#[test]
fn init_config_es_yes_adds_babel_to_style_sty() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_minimal_workspace(root);
    fs::write(root.join("template/style.sty"), "\\usepackage{amsmath}\n").expect("style sty");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("init_config")
        .write_stdin("es\ny\n\n\n\n\n\n\n\n\n\n\n")
        .assert()
        .success()
        .stdout(contains("\\usepackage[spanish]{babel}"));

    let style = fs::read_to_string(root.join("template/style.sty")).expect("style read");
    assert!(
        style.contains("\\usepackage[spanish]{babel}"),
        "babel debe insertarse al inicio de style.sty; contenido: {style}"
    );
    assert!(style.contains("\\usepackage{amsmath}"));
}

#[test]
fn init_config_es_no_keeps_style_sty_untouched() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_minimal_workspace(root);
    fs::write(
        root.join("template/style.sty"),
        "\\usepackage{amsmath}\n\\usepackage{geometry}\n",
    )
    .expect("style sty");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("init_config")
        .write_stdin("es\n\n\n\n\n\n\n\n\n\n\n\n")
        .assert()
        .success();

    let style = fs::read_to_string(root.join("template/style.sty")).expect("style read");
    assert!(
        !style.contains("babel"),
        "responder n no debe tocar style.sty; contenido: {style}"
    );
}

#[test]
fn init_config_es_updates_existing_babel_option() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_minimal_workspace(root);
    fs::write(
        root.join("template/style.sty"),
        "\\usepackage[english]{babel}\n\\usepackage{amsmath}\n",
    )
    .expect("style sty");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("init_config")
        .write_stdin("es\ny\n\n\n\n\n\n\n\n\n\n\n")
        .assert()
        .success();

    let style = fs::read_to_string(root.join("template/style.sty")).expect("style read");
    assert!(
        style.contains("\\usepackage[spanish]{babel}"),
        "la opcion de babel existente debe actualizarse; contenido: {style}"
    );
    assert!(!style.contains("[english]{babel}"));
}

#[test]
fn init_writes_ztxbase_sty_with_reference_engine() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_minimal_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("init")
        .assert()
        .success();

    let ztxbase = fs::read_to_string(root.join("template/ztxbase.sty")).expect("ztxbase read");
    assert!(
        ztxbase.contains("\\ProvidesPackage{ztxbase}"),
        "ztxbase.sty debe generarse; contenido: {ztxbase}"
    );
    assert!(
        ztxbase.contains("\\RequirePackage[hypertexnames=false]{hyperref}"),
        "ztxbase.sty debe incluir hyperref; contenido: {ztxbase}"
    );
    assert!(ztxbase.contains("\\newcommand{\\ztxhtmlhref}"));
    let style = fs::read_to_string(root.join("template/style.sty")).expect("style read");
    assert!(style.contains("\\usepackage[margin=2.5cm]{geometry}"));
}
