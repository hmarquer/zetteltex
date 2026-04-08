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

#[test]
fn help_works() {
    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex debe existir");
    cmd.arg("--help")
        .assert()
        .success()
    .stdout(contains("CLI Rust para gestionar ZettelTeX"));
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
        .stderr(contains("Error de workspace"));
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
        .arg("html")
        .assert()
        .code(1)
        .stderr(contains("Unsupported format"));
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
    fs::write(root.join("notes/slipbox/b.tex"), "\\excref{a}{defn:a}\n").expect("write b");
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
        "\\excref{missing-note}{defn:ghost}\n",
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
        .stdout(contains("Project teoria_de_grafos created"));

    let project_path = root.join("projects/teoria_de_grafos/teoria_de_grafos.tex");
    let project_content = fs::read_to_string(project_path).expect("project tex");
    assert!(project_content.contains("\\title{Teoria De Grafos}"));

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
    assert!(note_content.contains("\\title{Mi Nota}"));

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
        "\\excref{old}{defn:a}\\n\\hyperref[old-defn:a]{ver}\\n",
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
        .arg("rename_file")
        .arg("old")
        .arg("new")
        .assert()
        .success()
        .stdout(contains("Successfully renamed old to new"));

    assert!(!root.join("notes/slipbox/old.tex").exists());
    assert!(root.join("notes/slipbox/new.tex").exists());

    let ref_content = fs::read_to_string(root.join("notes/slipbox/ref.tex")).expect("ref read");
    assert!(ref_content.contains("\\excref{new}{defn:a}"));
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
fn rename_label_updates_references() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/target.tex"), "\\label{l1}\\n").expect("target");
    fs::write(
        root.join("notes/slipbox/consumer.tex"),
        "\\excref{target}{l1}\\n\\ref{target-l1}\\n\\hyperref[target-l1]{X}\\n",
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
        .arg("rename_label")
        .arg("target")
        .arg("l1")
        .arg("l2")
        .assert()
        .success()
        .stdout(contains("Successfully renamed label l1 to l2 in target"));

    let target_content =
        fs::read_to_string(root.join("notes/slipbox/target.tex")).expect("target read");
    assert!(target_content.contains("\\label{l2}"));
    assert!(!target_content.contains("\\label{l1}"));

    let consumer_content =
        fs::read_to_string(root.join("notes/slipbox/consumer.tex")).expect("consumer read");
    assert!(consumer_content.contains("\\excref{target}{l2}"));
    assert!(consumer_content.contains("\\ref{target-l2}"));
    assert!(consumer_content.contains("\\hyperref[target-l2]"));
}

#[test]
fn remove_note_removes_file_documents_and_db() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/killme.tex"), "\\label{x}\\n").expect("killme");
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
        .assert()
        .success()
        .stdout(contains("Removed note killme"));

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
        "\\label{defn:b}\\n\\excref{a}{defn:a}\n",
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
        .stdout(contains("Successfully renamed newer to renamed"));

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
        "\\excref{note-a}{defn:a}\nTODO: revisar ejemplo\n",
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
        .arg("export_project_markdown")
        .arg("materias")
        .assert()
        .success();

    let note_md =
        fs::read_to_string(root.join("jabberwocky/latex/zettelkasten/note-b.md")).expect("note md");
    assert!(note_md.contains("[[note-b.pdf]]"));
    assert!(note_md.contains("## Referencias"));
    assert!(note_md.contains("[note-a](./note-a.md)"));
    assert!(note_md.contains("#TODO revisar ejemplo"));

    let project_md = fs::read_to_string(root.join("jabberwocky/latex/asignaturas/materias.md"))
        .expect("project md");
    assert!(project_md.contains("[[materias.pdf]]"));
    assert!(project_md.contains("## Notas incluidas"));
    assert!(project_md.contains("[note-a](./note-a.md)"));
    assert!(project_md.contains("[note-b](./note-b.md)"));
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
        .stderr(contains("already exists in the database"));
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
        .stderr(contains("already exists in the database"));
}

#[test]
fn rename_file_fails_for_missing_note() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("rename_file")
        .arg("missing")
        .arg("newname")
        .assert()
        .failure()
        .stderr(contains("not found in database"));
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
        .stderr(contains("Project file not found"));
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
        .stderr(contains("Input file not found"));
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
        .stderr(contains("not found in database"));
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
    fs::write(root.join("projects/rp/rp.tex"), "\\chapter{X}\n").expect("rp");

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
        .arg("pdf")
        .arg("true")
        .assert()
        .success();

    let mut render_project = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    render_project
        .env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("render_project")
        .arg("rp")
        .assert()
        .success();

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
    assert!(logs.contains("pdflatex"));
    assert!(logs.contains("--jobname=nr"));
    assert!(logs.contains("--jobname=rp"));
    assert!(logs.contains("biber nr"));
    assert_eq!(logs.matches("--jobname=nr").count(), 2);
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
        .arg("render_all_projects")
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
    assert!(logs.contains("--jobname=a"));
    assert!(logs.contains("--jobname=b"));
    assert!(logs.contains("--jobname=pbatch"));
    assert!(logs.contains("biber a"));
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
    assert!(logs.contains("biber stale"));
    assert!(logs.contains("biber p-stale"));
}

#[test]
fn force_synchronize_notes_updates_note_db_state() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    fs::write(root.join("notes/slipbox/a.tex"), "\\label{defn:a}\n").expect("note a");
    fs::write(
        root.join("notes/slipbox/b.tex"),
        "\\excref{a}{defn:a}\n\\cite{key:b}\n",
    )
    .expect("note b");

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("force_synchronize_notes")
        .assert()
        .success()
        .stdout(contains("Force synchronize notas:"));

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
        .arg("force_synchronize_notes")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.arg("--workspace-root")
        .arg(root)
        .arg("force_synchronize_projects")
        .assert()
        .success()
        .stdout(contains("Force synchronize proyectos:"));

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
        .stdout(contains("Force synchronize completo:"));

    let conn = Connection::open(root.join("slipbox.db")).expect("open db");
    let projects_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM project", [], |row| row.get(0))
        .expect("project count");
    assert_eq!(projects_count, 1);
}

#[test]
fn render_all_pdf_alias_invokes_pdf_render_pipeline() {
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
        .arg("render_all_pdf")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read render_all_pdf log");
    assert!(logs.contains("--jobname=a"));
    assert!(logs.contains("--jobname=b"));
    assert!(logs.contains("biber a"));
}

#[test]
fn biber_project_invokes_biber_for_project_name() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&fake_bin).expect("fake bin");
    let log = root.join("biber-project.log");
    install_fake_tool(&fake_bin, "biber", &log);
    let path_env = prepend_path(&fake_bin);

    let mut cmd = Command::cargo_bin("zetteltex").expect("bin zetteltex");
    cmd.env("PATH", &path_env)
        .arg("--workspace-root")
        .arg(root)
        .arg("biber_project")
        .arg("proyecto-demo")
        .assert()
        .success();

    let logs = fs::read_to_string(&log).expect("read biber project log");
    assert!(logs.contains("biber proyecto-demo"));
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
        .stderr(contains("pdflatex not found in PATH"));
}

#[test]
fn biber_fails_when_biber_missing() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    setup_workspace(root);

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
        .stderr(contains("biber not found in PATH"));
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
        .stdout(contains("Removed 1 duplicate citation"));

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
        .stderr(contains("No such file:"));
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
    assert!(logs.contains("fuzzy --inline"));
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
