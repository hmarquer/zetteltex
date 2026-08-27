use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reference {
    pub target_note: String,
    pub target_label: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ParsedNote {
    pub labels: Vec<String>,
    pub citations: Vec<String>,
    pub references: Vec<Reference>,
    pub plain_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Inclusion {
    pub note_filename: String,
    pub tag: String,
}

static LABEL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\label\{([^}]+)\}").expect("regex label valida"));
static CURRENTDOC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\currentdoc\{([^}]+)\}").expect("regex currentdoc valida"));
static CITE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\cite[a-zA-Z\*]*\s*(?:\[[^\]]*\]\s*)?\{([^}]+)\}").expect("regex cite valida")
});
static REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\ref\{([^}]+)\}").expect("regex ref valida"));
static EXCREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\excref\[([^\]]+)\]\{([^}]+)\}").expect("regex excref valida"));
static EXHYPERREF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\exhyperref\[([^\]]+)\]\{([^}]+)\}\{[^}]*\}").expect("regex exhyperref valida")
});
static EXREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\exref\[([^\]]+)\]\{([^}]+)\}").expect("regex exref valida"));
static TRANSCLUDE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\transclude(?:\[([^\]]+)\])?\{([^}]+)\}").expect("regex transclude valida")
});

pub fn parse_note(content: &str) -> ParsedNote {
    let mut parsed = ParsedNote::default();

    // Strip LaTeX comments so commented-out commands are ignored
    let clean: String = content
        .lines()
        .map(strip_latex_comments)
        .collect::<Vec<_>>()
        .join("\n");

    let label_re = &*LABEL_RE;
    let currentdoc_re = &*CURRENTDOC_RE;
    let cite_re = &*CITE_RE;
    let ref_re = &*REF_RE;
    let excref_re = &*EXCREF_RE;
    let exhyperref_re = &*EXHYPERREF_RE;
    let exref_re = &*EXREF_RE;

    for caps in label_re.captures_iter(&clean) {
        parsed.labels.push(caps[1].trim().to_string());
    }
    for caps in currentdoc_re.captures_iter(&clean) {
        parsed.labels.push(caps[1].trim().to_string());
    }

    for caps in cite_re.captures_iter(&clean) {
        for citation_key in caps[1].split(',') {
            let key = citation_key.trim();
            if !key.is_empty() {
                parsed.citations.push(key.to_string());
            }
        }
    }

    for caps in ref_re.captures_iter(&clean) {
        parsed.plain_refs.push(caps[1].trim().to_string());
    }

    for caps in excref_re.captures_iter(&clean) {
        parsed.references.push(Reference {
            target_note: caps[2].trim().to_string(),
            target_label: caps[1].trim().to_string(),
        });
    }

    for caps in exhyperref_re.captures_iter(&clean) {
        parsed.references.push(Reference {
            target_note: caps[2].trim().to_string(),
            target_label: caps[1].trim().to_string(),
        });
    }

    for caps in exref_re.captures_iter(&clean) {
        parsed.references.push(Reference {
            target_note: caps[2].trim().to_string(),
            target_label: caps[1].trim().to_string(),
        });
    }

    parsed
}

pub fn parse_project_inclusions(content: &str) -> Vec<Inclusion> {
    let mut inclusions = Vec::new();
    let transclude_re = &*TRANSCLUDE_RE;

    for raw_line in content.lines() {
        let line = strip_latex_comments(raw_line);
        if line.trim().is_empty() {
            continue;
        }

        for caps in transclude_re.captures_iter(&line) {
            let tag = caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let note_filename = caps[2].trim().to_string();
            inclusions.push(Inclusion { note_filename, tag });
        }
    }

    inclusions
}

fn strip_latex_comments(line: &str) -> String {
    let mut out = String::new();
    let mut prev_backslash = false;

    for ch in line.chars() {
        if ch == '%' && !prev_backslash {
            break;
        }
        out.push(ch);
        prev_backslash = ch == '\\' && !prev_backslash;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{parse_note, parse_project_inclusions};

    #[test]
    fn parse_note_keeps_commands_after_escaped_percent() {
        let parsed = parse_note("Texto \\% \\label{ok}");

        assert_eq!(parsed.labels, vec!["ok"]);
    }

    #[test]
    fn parse_note_ignores_commands_after_real_comment() {
        let parsed = parse_note("Texto \\\\% \\label{bad}\n\\label{good}");

        assert_eq!(parsed.labels, vec!["good"]);
    }

    #[test]
    fn parse_project_inclusions_ignores_commented_transcludes() {
        let inclusions = parse_project_inclusions(
            "\\transclude{visible}\nTexto \\% \\transclude{kept}\nTexto \\\\% \\transclude{hidden}",
        );

        assert_eq!(
            inclusions
                .iter()
                .map(|item| item.note_filename.as_str())
                .collect::<Vec<_>>(),
            vec!["visible", "kept"]
        );
    }
}
