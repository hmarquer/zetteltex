//! Language Server Protocol implementation for ZettelTeX.
//!
//! Launches on stdio and answers `textDocument/completion` so the user gets
//! contextual completion while typing in the editor:
//!
//! * inside `\excref[<cursor>]{NOTA}` (the `[...]` slot) it completes the
//!   labels of `NOTA`;
//! * inside `\excref[LABEL]{<cursor>}` (the `{...}` slot) it completes note
//!   names.
//!
//! The same applies to `\exref` and `\exhyperref`, which share the
//! `[label]{note}` shape.

use std::collections::HashMap;

use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionOptionsCompletionItem,
    CompletionParams, CompletionResponse, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions,
};
use zetteltex_core::WorkspacePaths;
use zetteltex_parser::parse_note;

use crate::i18n::tr;

/// Commands whose argument shape is `[label]{note}`.
const LINK_SHAPED: &[&str] = &["\\excref", "\\exref", "\\exhyperref"];

/// What the cursor is asking for on the current line.
#[derive(Debug)]
enum CompletionContext {
    /// Inside the `{...}` slot: complete note names. `arg_start` is the byte
    /// offset of the `{` and `arg_end` the byte offset one past the region to
    /// replace (may include a trailing `}`), so accepting a note can insert
    /// the full `\excref[<label>]{note}` shape and re-open completion.
    Notes {
        prefix: String,
        arg_start: usize,
        arg_end: usize,
    },
    /// Inside the `[...]` slot: complete labels of `note`. `ls`/`le` are the
    /// byte offsets of the label slot content (cursor is within), so a "no
    /// label" item can remove the whole `[...]` when selected.
    Labels {
        note: String,
        prefix: String,
        ls: usize,
        le: usize,
    },
    /// Not inside a recognized command slot.
    None,
}

/// Runs the LSP server on stdio until the client exits.
pub(crate) fn run_lsp(paths: &WorkspacePaths) -> Result<()> {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["{".to_string(), "[".to_string(), ",".to_string()]),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
            }),
        }),
        ..ServerCapabilities::default()
    })
    .expect("server capabilities serializable");

    let client_params = connection
        .initialize(server_capabilities)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let init_params: lsp_types::InitializeParams =
        serde_json::from_value(client_params).unwrap_or_default();

    // If the extension did not pass --workspace-root, fall back to the first
    // workspace folder announced by the client, when available.
    let workspace_root = init_params
        .workspace_folders
        .and_then(|folders| folders.into_iter().next())
        .and_then(|folder| file_uri_to_path(&folder.uri.to_string()))
        .filter(|p| p.is_dir());

    let mut server = LspServer {
        paths: paths.clone(),
        documents: HashMap::new(),
        fallback_root: workspace_root,
    };
    eprintln!(
        "[zetteltex-lsp] ready, slipbox={}",
        paths.notes_slipbox.display()
    );

    let result = server_main_loop(&connection, &mut server);

    // Drop the connection so the writer thread sees its sender closed and can
    // terminate; then join the io threads. Without this, `io_threads.join()`
    // below would block forever on the still-open writer channel.
    drop(connection);
    io_threads.join()?;
    result
}

fn server_main_loop(connection: &Connection, server: &mut LspServer) -> Result<()> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if req.method.as_str() == "shutdown" {
                    // Responds to the shutdown request and blocks until the
                    // `exit` notification arrives (with a 30s timeout).
                    connection
                        .handle_shutdown(&req)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    return Ok(());
                }
                if handle_request(connection, server, req)? {
                    return Ok(());
                }
            }
            Message::Notification(not) => {
                if not.method.as_str() == "exit" {
                    return Ok(());
                }
                if handle_notification(server, not)? {
                    return Ok(());
                }
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

struct LspServer {
    paths: WorkspacePaths,
    /// uri -> full text
    documents: HashMap<String, String>,
    /// root announced by the client during `initialize` (used as fallback).
    #[allow(dead_code)]
    fallback_root: Option<std::path::PathBuf>,
}

/// Returns `Ok(true)` if the server should exit.
fn handle_request(connection: &Connection, server: &mut LspServer, req: Request) -> Result<bool> {
    match req.method.as_str() {
        "textDocument/completion" => {
            let (id, params) = req
                .extract::<CompletionParams>("textDocument/completion")
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            eprintln!(
                "[zetteltex-lsp] completion request at {}:{}:{}",
                *params.text_document_position.text_document.uri,
                params.text_document_position.position.line,
                params.text_document_position.position.character
            );
            let items = server.completion(&params);
            eprintln!("[zetteltex-lsp] returning {} items", items.len());
            let result = CompletionResponse::Array(items);
            let resp = Response::new_ok(id, result);
            connection.sender.send(resp.into())?;
            Ok(false)
        }
        _ => {
            let resp = Response::new_err(
                req.id.clone(),
                lsp_server::ErrorCode::MethodNotFound as i32,
                format!("unhandled method {}", req.method),
            );
            connection.sender.send(resp.into())?;
            Ok(false)
        }
    }
}

/// Returns `Ok(true)` if the server should exit.
fn handle_notification(server: &mut LspServer, not: Notification) -> Result<bool> {
    match not.method.as_str() {
        "textDocument/didOpen" => {
            let params: lsp_types::DidOpenTextDocumentParams =
                serde_json::from_value(not.params).map_err(|e| anyhow::anyhow!("{e}"))?;
            eprintln!(
                "[zetteltex-lsp] didOpen {} ({} chars, lang={:?})",
                *params.text_document.uri,
                params.text_document.text.len(),
                params.text_document.language_id
            );
            server.documents.insert(
                params.text_document.uri.to_string(),
                params.text_document.text,
            );
            Ok(false)
        }
        "textDocument/didChange" => {
            let params: lsp_types::DidChangeTextDocumentParams =
                serde_json::from_value(not.params).map_err(|e| anyhow::anyhow!("{e}"))?;
            let uri = params.text_document.uri.to_string();
            if let Some(change) = params.content_changes.last() {
                server.documents.insert(uri, change.text.clone());
            }
            Ok(false)
        }
        "textDocument/didClose" => {
            let params: lsp_types::DidCloseTextDocumentParams =
                serde_json::from_value(not.params).map_err(|e| anyhow::anyhow!("{e}"))?;
            server
                .documents
                .remove(&params.text_document.uri.to_string());
            Ok(false)
        }
        "initialized" => Ok(false),
        _ => Ok(false),
    }
}

impl LspServer {
    fn completion(&self, params: &CompletionParams) -> Vec<CompletionItem> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let Some(text) = self.documents.get(&uri) else {
            return Vec::new();
        };
        let Some(line_text) = line_at(text, params.text_document_position.position.line) else {
            return Vec::new();
        };
        let char_col = params.text_document_position.position.character as usize;
        let line_no = params.text_document_position.position.line;

        match completion_context(line_text, char_col) {
            CompletionContext::Notes {
                prefix,
                arg_start: _arg_start,
                arg_end,
            } => {
                let notes = list_notes(self.paths_for(uri.as_str()));

                // `arg_start` = index of the `{`; `arg_end` is one past the note
                // content, extended by one when a closing `}` was already typed
                // (see `context_for_command`). So a trailing `}` exists exactly
                // when the region up to `arg_end` ends with `}`.
                let has_close_brace = line_text[..arg_end].ends_with('}');

                let items: Vec<CompletionItem> =
                    filter_items(notes.iter().map(|s| s.as_str()), &prefix)
                        .into_iter()
                        .map(|name| {
                            // VS Code rejects completion items whose `textEdit` range
                            // starts before the current cursor, so we cannot replace the
                            // leading `{`. Instead we insert the note name (plus a closing
                            // `}` when the user did not type one) at the cursor / current
                            // word, which VS Code accepts.
                            let insert = if has_close_brace {
                                name.clone()
                            } else {
                                format!("{name}}}")
                            };
                            CompletionItem {
                                label: name.clone(),
                                kind: Some(CompletionItemKind::FILE),
                                detail: Some(tr!("nota", "note").to_string()),
                                insert_text: Some(insert),
                                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                                ..CompletionItem::default()
                            }
                        })
                        .collect();
                items
            }
            CompletionContext::Labels {
                note,
                prefix,
                ls: _ls,
                le,
            } => {
                let labels = list_labels(self.paths_for(uri.as_str()), &note);

                // Replace whatever the user has typed in the `[...]` slot (from
                // the current cursor position through the end of the slot) with
                // the chosen label. Starting the edit AT the cursor means the
                // range never precedes the cursor, so VS Code accepts it.
                let edit_range = |till: usize| {
                    lsp_types::Range::new(
                        lsp_types::Position::new(line_no, char_col as u32),
                        lsp_types::Position::new(line_no, till as u32),
                    )
                };

                // Offer a "just the note" option first: selecting it simply
                // clears the typed label prefix (leaving an empty `[...]`).
                let no_label_item = CompletionItem {
                    label: tr!("(sin etiqueta)", "(no label)").to_string(),
                    kind: Some(CompletionItemKind::TEXT),
                    detail: Some(
                        tr!("solo la nota, sin etiqueta", "just the note, no label").to_string(),
                    ),
                    text_edit: Some(lsp_types::CompletionTextEdit::Edit(
                        lsp_types::TextEdit::new(edit_range(le), String::new()),
                    )),
                    ..CompletionItem::default()
                };

                let mut items = vec![no_label_item];
                items.extend(
                    filter_items(labels.iter().map(|s| s.as_str()), &prefix)
                        .into_iter()
                        .map(|label| CompletionItem {
                            label: label.clone(),
                            kind: Some(CompletionItemKind::FIELD),
                            detail: Some(tr!("label en '{note}'", "label in '{note}'").to_string()),
                            text_edit: Some(lsp_types::CompletionTextEdit::Edit(
                                lsp_types::TextEdit::new(edit_range(le), label.clone()),
                            )),
                            ..CompletionItem::default()
                        }),
                );
                items
            }
            CompletionContext::None => Vec::new(),
        }
    }

    /// The completion context is resolved from the current/fallback workspace.
    fn paths_for(&self, _uri: &str) -> &WorkspacePaths {
        &self.paths
    }
}

/// Returns the text of the given (0-indexed) line, or `None`.
fn line_at(text: &str, line: u32) -> Option<&str> {
    text.lines().nth(line as usize)
}

/// Classify what the cursor (at UTF-16 `col` within `line`) should complete.
fn completion_context(line: &str, col: usize) -> CompletionContext {
    // The cursor column (LSP UTF-16) maps to a byte offset in the line. This is
    // exact for the ASCII-heavy LaTeX text we care about.
    let byte_col = utf16_to_byte(line, col);

    // Find the command in `LINK_SHAPED` that encloses the cursor and determine
    // which slot it is in.
    for cmd in LINK_SHAPED {
        if let Some(ctx) = context_for_command(line, cmd, byte_col) {
            return ctx;
        }
    }
    CompletionContext::None
}

/// Checks whether `cmd` appears on `line` such that the cursor (`byte_col`)
/// lies within one of its argument slots, and returns the matching context.
fn context_for_command(line: &str, cmd: &str, byte_col: usize) -> Option<CompletionContext> {
    let mut search_from = 0;
    let cmd_bytes = cmd.len();
    while let Some(rel) = line[search_from..].find(cmd) {
        let cmd_start = search_from + rel;
        let after_cmd = cmd_start + cmd_bytes;
        let line_len = line.len();

        // Scanning state after `\excref`:
        let mut pos = after_cmd;

        // Slot 1: optional `[label]`. When the closing `]` is missing we treat
        // the end of line as its close, so completion works while typing.
        let label_range = if line[pos..].starts_with('[') {
            pos += 1; // skip '['
            let start = pos;
            let close = line[pos..].find(']').map(|r| pos + r);
            let end = close.unwrap_or(line_len);
            if close.is_some() {
                pos = end + 1;
            }
            Some((start, end))
        } else {
            None
        };

        // Slot 2: `{note}`. Missing `}` → end of line.
        let note_range = if line[pos..].starts_with('{') {
            pos += 1; // skip '{'
            let start = pos;
            let close = line[pos..].find('}').map(|r| pos + r);
            let end = close.unwrap_or(line_len);
            Some((start, end))
        } else {
            None
        };

        if let Some((ls, le)) = label_range {
            if byte_col >= ls && byte_col <= le {
                // Extract the note name already typed in `{...}` (if any).
                let note = note_range
                    .map(|(ns, ne)| line[ns..ne].trim().to_string())
                    .unwrap_or_default();
                let prefix = line[ls..byte_col.min(le)].trim().to_string();
                return Some(CompletionContext::Labels {
                    note,
                    prefix,
                    ls,
                    le,
                });
            }
        }

        if let Some((ns, ne)) = note_range {
            if byte_col >= ns && byte_col <= ne {
                let before_cursor = &line[ns..byte_col.min(ne)];
                let prefix = before_cursor.trim().to_string();
                // Region to replace when the user accepts a note: from the `{`
                // (argument open) through the note content, plus a trailing
                // `}` if one was already typed.
                let arg_start = ns - 1; // the `{`
                let arg_end = if line[ne..].starts_with('}') {
                    ne + 1
                } else {
                    ne
                };
                return Some(CompletionContext::Notes {
                    prefix,
                    arg_start,
                    arg_end,
                });
            }
        }

        search_from = cmd_start + 1;
    }
    None
}

/// Convert an LSP UTF-16 column position into a byte offset within `line`.
fn utf16_to_byte(line: &str, col: usize) -> usize {
    let mut byte_count = 0;
    for (i, c) in line.char_indices() {
        if i >= col {
            break;
        }
        byte_count = i + c.len_utf8();
    }
    byte_count.min(line.len())
}

/// List note stems available in the workspace slipbox.
fn list_notes(paths: &WorkspacePaths) -> Vec<String> {
    let mut notes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&paths.notes_slipbox) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("tex") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    notes.push(stem.to_string());
                }
            }
        }
    }
    notes.sort();
    notes
}

/// List labels defined in the note `name` (via `\label` / `\currentdoc`).
fn list_labels(paths: &WorkspacePaths, name: &str) -> Vec<String> {
    if name.is_empty() {
        return Vec::new();
    }
    let tex_path = paths.notes_slipbox.join(format!("{name}.tex"));
    let Ok(content) = std::fs::read_to_string(&tex_path) else {
        return Vec::new();
    };
    let parsed = parse_note(&content);
    let mut labels = parsed.labels;
    labels.sort();
    labels.dedup();
    labels
}

/// Filter `candidates`, keeping those starting with `prefix` (case-insensitive).
fn filter_items<'a>(candidates: impl Iterator<Item = &'a str>, prefix: &str) -> Vec<String> {
    let prefix_lower = prefix.to_lowercase();
    candidates
        .filter(|c| c.to_lowercase().starts_with(&prefix_lower))
        .map(|c| c.to_string())
        .collect()
}

/// Convert a `file:///…` URI string to a filesystem path, decoding `%XX`
/// escapes. Returns `None` for non-`file` schemes.
fn file_uri_to_path(uri: &str) -> Option<std::path::PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    // `file:///path` → `/path` (leading slash belongs to the absolute path);
    // `file://server/path` → treat as local by dropping the host part.
    let path_part: String = if rest.starts_with('/') {
        rest.to_string()
    } else {
        match rest.split_once('/') {
            Some((_, p)) => format!("/{p}"),
            None => rest.to_string(),
        }
    };
    Some(std::path::PathBuf::from(percent_decode(&path_part)))
}

/// Decode percent-encoded (UTF-8) octets in a URI path.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(line: &str, col: usize) -> CompletionContext {
        completion_context(line, col)
    }

    #[test]
    fn context_notes_slot_in_excref() {
        // Cursor inside `{...}` of `\excref[defn:a]{<cursor>..}` (unclosed `}`)
        match ctx(r"\excref[defn:a]{cu", 18) {
            CompletionContext::Notes {
                prefix,
                arg_start,
                arg_end,
            } => {
                assert_eq!(prefix, "cu");
                // `\excref[defn:a]` is 15 bytes; `{` sits at 15, `cu` is 15..17,
                // and with no `}` the region extends to end of line (18).
                assert_eq!(arg_start, 15);
                assert_eq!(arg_end, 18);
            }
            other => panic!("expected Notes, got {other:?}"),
        }
    }

    #[test]
    fn context_labels_slot_in_excref() {
        // Cursor inside `[...]` of `\excref[<cursor>..]{cuerpo}`
        match ctx(r"\excref[de]{cuerpo}", 10) {
            CompletionContext::Labels { note, prefix, .. } => {
                assert_eq!(note, "cuerpo");
                assert_eq!(prefix, "de");
            }
            other => panic!("expected Labels, got {other:?}"),
        }
    }

    #[test]
    fn context_none_outside_command() {
        assert!(matches!(
            ctx("una linea normal de texto aqui", 10),
            CompletionContext::None
        ));
    }

    #[test]
    fn context_notes_prefix() {
        match ctx(r"\excref[LABEL]{mat", 18) {
            CompletionContext::Notes { prefix, .. } => assert_eq!(prefix, "mat"),
            other => panic!("expected Notes, got {other:?}"),
        }
    }
}
