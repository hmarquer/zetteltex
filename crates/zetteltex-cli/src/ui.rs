use super::*;
use crate::i18n::tr;

pub(crate) fn run_fuzzy_tui(
    paths: &WorkspacePaths,
    index: &FuzzyIndex,
) -> Result<Option<FuzzyUiAction>> {
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
        let results = fuzzy_results_for_ui(
            index,
            &query,
            index.settings.max_results,
            index.settings.history_results,
            &history,
        );
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
            code, modifiers, ..
        }) = event::read()?
        else {
            continue;
        };

        match (code, modifiers) {
            (KeyCode::Esc, _) => return Ok(None),
            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => return Ok(None),
            (KeyCode::Backspace, m) if m.contains(KeyModifiers::CONTROL) => {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyExhyperref {
                        item: (*item).clone(),
                    }));
                }
            }
            (KeyCode::Left, m) if m.contains(KeyModifiers::CONTROL) => {
                let chars: Vec<char> = query.chars().collect();
                if cursor_pos > 0 {
                    let mut p = cursor_pos - 1;
                    while p > 0 && chars[p].is_whitespace() {
                        p -= 1;
                    }
                    while p > 0 && !chars[p - 1].is_whitespace() {
                        p -= 1;
                    }
                    cursor_pos = p;
                }
            }
            (KeyCode::Right, m) if m.contains(KeyModifiers::CONTROL) => {
                let chars: Vec<char> = query.chars().collect();
                if cursor_pos < chars.len() {
                    let mut p = cursor_pos;
                    while p < chars.len() && chars[p].is_whitespace() {
                        p += 1;
                    }
                    while p < chars.len() && !chars[p].is_whitespace() {
                        p += 1;
                    }
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
            (KeyCode::Enter, _) => {}
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'h') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyExhyperref {
                        item: (*item).clone(),
                    }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'r') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyExcref {
                        item: (*item).clone(),
                    }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'e') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::OpenEditor {
                        item: (*item).clone(),
                    }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'p') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::OpenPdf {
                        item: (*item).clone(),
                    }));
                }
            }
            (KeyCode::Char(ch), m)
                if m.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'t') =>
            {
                if let Some((item, _)) = results.get(selected) {
                    return Ok(Some(FuzzyUiAction::CopyTransclude {
                        item: (*item).clone(),
                    }));
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
                status_line = Some(
                    tr(
                        "Ctrl+Alt+N requiere barra de busqueda vacia",
                        "Ctrl+Alt+N requires an empty search bar",
                    )
                    .to_string(),
                );
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
    history_results: usize,
    history: &[String],
) -> Vec<(&'a FuzzyItem, f64)> {
    if !query.trim().is_empty() {
        return fuzzy_search(index, query, max_results);
    }

    // Paridad con fuzzy.py: historial en orden de recencia y luego populares.
    let target = history_results;
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

#[allow(clippy::too_many_arguments)]
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

    render_search_bar(
        f,
        query,
        left_chunks[0],
        index.settings.accent_color,
        cursor_pos,
    );
    render_results_list(
        f,
        results,
        selected,
        left_chunks[1],
        index.settings.accent_color,
    );
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

fn render_search_bar(
    f: &mut Frame,
    query: &str,
    area: Rect,
    accent_color: Color,
    cursor_pos: usize,
) {
    let paragraph = Paragraph::new(query.to_string())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", tr("Buscar", "Search")))
                .border_style(Style::default().fg(accent_color)),
        );
    f.render_widget(paragraph, area);

    // Set terminal cursor position
    let inner_area = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 1,
    });
    let max_x = inner_area.right().saturating_sub(1);
    let target_x = inner_area.x + cursor_pos as u16;
    f.set_cursor_position(ratatui::layout::Position::new(
        target_x.min(max_x),
        inner_area.y,
    ));
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
        .map(|(item, _)| {
            ListItem::new(item.display.clone()).style(Style::default().fg(Color::White))
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " {} ({}) ",
                    tr("Resultados", "Results"),
                    results.len()
                ))
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
    let help = tr(
        "Ctrl+H: exhyperref | Ctrl+R: excref | Ctrl+T: transclude | Ctrl+E: editor | Ctrl+P: PDF | Ctrl+N: nota nueva | Ctrl+Alt+N: portapapeles | Esc: salir",
        "Ctrl+H: exhyperref | Ctrl+R: excref | Ctrl+T: transclude | Ctrl+E: editor | Ctrl+P: PDF | Ctrl+N: new note | Ctrl+Alt+N: clipboard | Esc: quit"
    );
    let (text, style) = if let Some(msg) = status_line {
        (
            msg,
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
        )
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
        .map(|(item, _)| {
            preview_lines_for_item(index, item, area.height.saturating_sub(2) as usize)
        })
        .unwrap_or_else(|| vec![tr("No hay resultados", "No results").to_string()]);

    let lines = preview
        .iter()
        .map(|line| highlight_latex_line(line, &search_term, index.settings.accent_color))
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", tr("Vista Previa", "Preview")))
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
