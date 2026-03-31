use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};

#[derive(Debug, Clone)]
pub struct NoteRecord {
    pub id: i64,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct ProjectRecord {
    pub id: i64,
    pub name: String,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub name: String,
    pub filename: String,
    pub created: Option<String>,
    pub last_edit_date: Option<String>,
    pub last_build_date_pdf: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LabelRecord {
    pub id: i64,
    pub note_id: i64,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct LinkRecord {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
}

#[derive(Debug, Clone)]
pub struct CitationRecord {
    pub id: i64,
    pub note_id: i64,
    pub citation_key: String,
}

#[derive(Debug, Clone)]
pub struct InclusionRecord {
    pub id: i64,
    pub project_id: i64,
    pub note_id: i64,
    pub source_file: String,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct ProjectInclusionView {
    pub note_filename: String,
    pub source_file: String,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct NoteProjectView {
    pub project_name: String,
    pub source_file: String,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct NotePopularityRecord {
    pub filename: String,
    pub in_refs: i64,
    pub out_refs: i64,
}

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(database_path: &Path) -> Result<Self> {
        let conn = Connection::open(database_path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA synchronous = NORMAL;
            "#,
        )?;

        // Evitamos WAL para no generar archivos auxiliares .db-wal/.db-shm.
        // Cambiar journal_mode puede requerir lock exclusivo. Si hay contencion,
        // no abortamos la apertura: seguimos con el modo actual y dejamos que las
        // operaciones normales usen busy_timeout/reintentos a nivel de CLI.
        if let Err(err) = conn.execute_batch("PRAGMA journal_mode = DELETE;") {
            if !is_sqlite_lock_like(&err) {
                return Err(err.into());
            }
        }

        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS note (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL UNIQUE,
                last_build_date_pdf TEXT,
                last_edit_date TEXT,
                created TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL UNIQUE,
                last_build_date_pdf TEXT,
                last_edit_date TEXT,
                created TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS label (
                id INTEGER PRIMARY KEY,
                note_id INTEGER NOT NULL,
                label TEXT NOT NULL,
                FOREIGN KEY(note_id) REFERENCES note(id) ON DELETE CASCADE,
                UNIQUE(note_id, label)
            );

            CREATE TABLE IF NOT EXISTS link (
                id INTEGER PRIMARY KEY,
                source_id INTEGER NOT NULL,
                target_id INTEGER NOT NULL,
                FOREIGN KEY(source_id) REFERENCES note(id) ON DELETE CASCADE,
                FOREIGN KEY(target_id) REFERENCES label(id) ON DELETE CASCADE,
                UNIQUE(source_id, target_id)
            );

            CREATE TABLE IF NOT EXISTS citation (
                id INTEGER PRIMARY KEY,
                note_id INTEGER NOT NULL,
                citationkey TEXT NOT NULL,
                FOREIGN KEY(note_id) REFERENCES note(id) ON DELETE CASCADE,
                UNIQUE(note_id, citationkey)
            );

            CREATE TABLE IF NOT EXISTS inclusion (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL,
                note_id INTEGER NOT NULL,
                source_file TEXT NOT NULL,
                tag TEXT NOT NULL DEFAULT '',
                FOREIGN KEY(project_id) REFERENCES project(id) ON DELETE CASCADE,
                FOREIGN KEY(note_id) REFERENCES note(id) ON DELETE CASCADE,
                UNIQUE(project_id, note_id, source_file, tag)
            );

            CREATE TABLE IF NOT EXISTS tag (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS notetag (
                id INTEGER PRIMARY KEY,
                note_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                FOREIGN KEY(note_id) REFERENCES note(id) ON DELETE CASCADE,
                FOREIGN KEY(tag_id) REFERENCES tag(id) ON DELETE CASCADE,
                UNIQUE(note_id, tag_id)
            );
            "#,
        )?;
        Ok(())
    }

    pub fn upsert_note(&self, filename: &str, last_edit_date: DateTime<Utc>) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        let last_edit = last_edit_date.to_rfc3339();

        self.conn.execute(
            r#"
            INSERT INTO note (filename, last_edit_date, created)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(filename)
            DO UPDATE SET last_edit_date = excluded.last_edit_date
            "#,
            params![filename, last_edit, now],
        )?;

        self.note_id_by_filename(filename)?
            .ok_or_else(|| anyhow::anyhow!("no se pudo recuperar id para nota '{filename}'"))
    }

    pub fn upsert_project(
        &self,
        name: &str,
        filename: &str,
        last_edit_date: DateTime<Utc>,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        let last_edit = last_edit_date.to_rfc3339();

        self.conn.execute(
            r#"
            INSERT INTO project (name, filename, last_edit_date, created)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(name)
            DO UPDATE SET
                filename = excluded.filename,
                last_edit_date = excluded.last_edit_date
            "#,
            params![name, filename, last_edit, now],
        )?;

        self.project_id_by_name(name)?
            .ok_or_else(|| anyhow::anyhow!("no se pudo recuperar id para proyecto '{name}'"))
    }

    pub fn project_id_by_name(&self, name: &str) -> Result<Option<i64>> {
        let id = self
            .conn
            .query_row(
                "SELECT id FROM project WHERE name = ?1",
                params![name],
                |row| row.get(0),
            )
            .optional()?;
        Ok(id)
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, filename FROM project ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ProjectRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                filename: row.get(2)?,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn project_metadata_by_name(&self, name: &str) -> Result<Option<ProjectMetadata>> {
        let metadata = self
            .conn
            .query_row(
                r#"
                SELECT name, filename, created, last_edit_date, last_build_date_pdf
                FROM project
                WHERE name = ?1
                "#,
                params![name],
                |row| {
                    Ok(ProjectMetadata {
                        name: row.get(0)?,
                        filename: row.get(1)?,
                        created: row.get(2)?,
                        last_edit_date: row.get(3)?,
                        last_build_date_pdf: row.get(4)?,
                    })
                },
            )
            .optional()?;
        Ok(metadata)
    }

    pub fn list_notes(&self) -> Result<Vec<NoteRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, filename FROM note ORDER BY filename ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(NoteRecord {
                id: row.get(0)?,
                filename: row.get(1)?,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn note_popularity_stats(&self) -> Result<Vec<NotePopularityRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                n.filename,
                COALESCE((
                    SELECT COUNT(*)
                    FROM label lb
                    INNER JOIN link lk ON lk.target_id = lb.id
                    WHERE lb.note_id = n.id
                ), 0) AS in_refs,
                COALESCE((
                    SELECT COUNT(*)
                    FROM link lk
                    WHERE lk.source_id = n.id
                ), 0) AS out_refs
            FROM note n
            ORDER BY n.filename ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(NotePopularityRecord {
                filename: row.get(0)?,
                in_refs: row.get(1)?,
                out_refs: row.get(2)?,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn list_project_inclusions_by_name(
        &self,
        project_name: &str,
    ) -> Result<Vec<ProjectInclusionView>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT n.filename, i.source_file, COALESCE(i.tag, '')
            FROM inclusion i
            INNER JOIN project p ON p.id = i.project_id
            INNER JOIN note n ON n.id = i.note_id
            WHERE p.name = ?1
            ORDER BY i.source_file ASC, n.filename ASC
            "#,
        )?;
        let rows = stmt.query_map(params![project_name], |row| {
            Ok(ProjectInclusionView {
                note_filename: row.get(0)?,
                source_file: row.get(1)?,
                tag: row.get(2)?,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn list_note_projects(&self, note_filename: &str) -> Result<Vec<NoteProjectView>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT p.name, i.source_file, COALESCE(i.tag, '')
            FROM inclusion i
            INNER JOIN project p ON p.id = i.project_id
            INNER JOIN note n ON n.id = i.note_id
            WHERE n.filename = ?1
            ORDER BY p.name ASC, i.source_file ASC
            "#,
        )?;
        let rows = stmt.query_map(params![note_filename], |row| {
            Ok(NoteProjectView {
                project_name: row.get(0)?,
                source_file: row.get(1)?,
                tag: row.get(2)?,
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn note_id_by_filename(&self, filename: &str) -> Result<Option<i64>> {
        let id = self
            .conn
            .query_row(
                "SELECT id FROM note WHERE filename = ?1",
                params![filename],
                |row| row.get(0),
            )
            .optional()?;
        Ok(id)
    }

    pub fn labels_for_note(&self, note_filename: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT l.label
            FROM label l
            INNER JOIN note n ON n.id = l.note_id
            WHERE n.filename = ?1
            ORDER BY l.label ASC
            "#,
        )?;
        let rows = stmt.query_map(params![note_filename], |row| row.get::<_, String>(0))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn note_exists(&self, filename: &str) -> Result<bool> {
        Ok(self.note_id_by_filename(filename)?.is_some())
    }

    pub fn notes_needing_render(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT filename
            FROM note
            WHERE last_build_date_pdf IS NULL
               OR last_edit_date IS NULL
               OR last_edit_date > last_build_date_pdf
            ORDER BY filename ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn projects_needing_render(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT name
            FROM project
            WHERE last_build_date_pdf IS NULL
               OR last_edit_date IS NULL
               OR last_edit_date > last_build_date_pdf
            ORDER BY name ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn note_has_citations(&self, filename: &str) -> Result<bool> {
        let exists = self.conn.query_row(
            r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM citation c
                    INNER JOIN note n ON n.id = c.note_id
                    WHERE n.filename = ?1
                )
                "#,
            params![filename],
            |row| row.get::<_, i64>(0),
        )? == 1;
        Ok(exists)
    }

    pub fn set_note_last_build_date_pdf(
        &self,
        filename: &str,
        build_date: DateTime<Utc>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE note SET last_build_date_pdf = ?1 WHERE filename = ?2",
            params![build_date.to_rfc3339(), filename],
        )?;
        Ok(())
    }

    pub fn set_project_last_build_date_pdf(
        &self,
        name: &str,
        build_date: DateTime<Utc>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE project SET last_build_date_pdf = ?1 WHERE name = ?2",
            params![build_date.to_rfc3339(), name],
        )?;
        Ok(())
    }

    pub fn list_unreferenced_notes(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT n.filename
            FROM note n
            LEFT JOIN label l ON l.note_id = n.id
            LEFT JOIN link k ON k.target_id = l.id
            GROUP BY n.id, n.filename
            HAVING COUNT(k.id) = 0
            ORDER BY n.filename ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn rename_note_filename(&self, old_filename: &str, new_filename: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE note SET filename = ?1 WHERE filename = ?2",
            params![new_filename, old_filename],
        )?;
        Ok(())
    }

    pub fn delete_note_by_filename(&self, filename: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM note WHERE filename = ?1", params![filename])?;
        Ok(())
    }

    pub fn replace_labels(&self, note_id: i64, labels: &[String]) -> Result<()> {
        self.conn
            .execute("DELETE FROM label WHERE note_id = ?1", params![note_id])?;

        let mut seen = HashSet::new();
        for label in labels {
            if !seen.insert(label.clone()) {
                continue;
            }
            self.conn.execute(
                "INSERT OR IGNORE INTO label (note_id, label) VALUES (?1, ?2)",
                params![note_id, label],
            )?;
        }
        Ok(())
    }

    pub fn replace_citations(&self, note_id: i64, citation_keys: &[String]) -> Result<()> {
        self.conn
            .execute("DELETE FROM citation WHERE note_id = ?1", params![note_id])?;

        let mut seen = HashSet::new();
        for key in citation_keys {
            if !seen.insert(key.clone()) {
                continue;
            }
            self.conn.execute(
                "INSERT OR IGNORE INTO citation (note_id, citationkey) VALUES (?1, ?2)",
                params![note_id, key],
            )?;
        }
        Ok(())
    }

    pub fn remove_duplicate_citations(&self) -> Result<usize> {
        self.conn.execute(
            r#"
            DELETE FROM citation
            WHERE id NOT IN (
                SELECT MIN(id)
                FROM citation
                GROUP BY note_id, citationkey
            )
            "#,
            [],
        )?;
        Ok(self.conn.changes() as usize)
    }

    pub fn clear_links(&self) -> Result<()> {
        self.conn.execute("DELETE FROM link", [])?;
        Ok(())
    }

    pub fn replace_project_inclusions(
        &self,
        project_id: i64,
        inclusions: &[(i64, String, String)],
    ) -> Result<()> {
        self.conn.execute(
            "DELETE FROM inclusion WHERE project_id = ?1",
            params![project_id],
        )?;

        let mut seen = HashSet::new();
        for (note_id, source_file, tag) in inclusions {
            let key = (*note_id, source_file.clone(), tag.clone());
            if !seen.insert(key) {
                continue;
            }
            self.conn.execute(
                r#"
                INSERT OR IGNORE INTO inclusion (project_id, note_id, source_file, tag)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                params![project_id, note_id, source_file, tag],
            )?;
        }

        Ok(())
    }

    pub fn target_label_id(&self, note_filename: &str, label: &str) -> Result<Option<i64>> {
        let target = self
            .conn
            .query_row(
                r#"
                SELECT l.id
                FROM label l
                INNER JOIN note n ON n.id = l.note_id
                WHERE n.filename = ?1 AND l.label = ?2
                "#,
                params![note_filename, label],
                |row| row.get(0),
            )
            .optional()?;
        Ok(target)
    }

    pub fn label_exists(&self, note_filename: &str, label: &str) -> Result<bool> {
        Ok(self.target_label_id(note_filename, label)?.is_some())
    }

    pub fn insert_link(&self, source_id: i64, target_label_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO link (source_id, target_id) VALUES (?1, ?2)",
            params![source_id, target_label_id],
        )?;
        Ok(())
    }
}

fn is_sqlite_lock_like(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(e, _) if matches!(e.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
    )
}

pub fn init_database(database_path: &Path) -> Result<Database> {
    Database::open(database_path)
}
