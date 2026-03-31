use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Inclusion {
    pub note_filename: String,
    pub tag: String,
}

pub fn parse_note(content: &str) -> Result<ParsedNote> {
    let mut parsed = ParsedNote::default();

    let label_re = Regex::new(r"\\label\{([^}]+)\}")?;
    let currentdoc_re = Regex::new(r"\\currentdoc\{([^}]+)\}")?;
    let cite_re = Regex::new(r"\\cite[a-zA-Z\*]*\s*(?:\[[^\]]*\]\s*)?\{([^}]+)\}")?;
    let excref_re = Regex::new(r"\\excref\{([^}]+)\}\{([^}]+)\}")?;
    let exhyperref_re = Regex::new(r"\\exhyperref\[([^\]]+)\]\{([^}]+)\}\{[^}]*\}")?;
    let exref_re = Regex::new(r"\\exref\[([^\]]+)\]\{([^}]+)\}")?;

    for caps in label_re.captures_iter(content) {
        parsed.labels.push(caps[1].trim().to_string());
    }
    for caps in currentdoc_re.captures_iter(content) {
        parsed.labels.push(caps[1].trim().to_string());
    }

    for caps in cite_re.captures_iter(content) {
        for citation_key in caps[1].split(',') {
            let key = citation_key.trim();
            if !key.is_empty() {
                parsed.citations.push(key.to_string());
            }
        }
    }

    for caps in excref_re.captures_iter(content) {
        parsed.references.push(Reference {
            target_note: caps[1].trim().to_string(),
            target_label: caps[2].trim().to_string(),
        });
    }

    for caps in exhyperref_re.captures_iter(content) {
        parsed.references.push(Reference {
            target_note: caps[2].trim().to_string(),
            target_label: caps[1].trim().to_string(),
        });
    }

    for caps in exref_re.captures_iter(content) {
        parsed.references.push(Reference {
            target_note: caps[2].trim().to_string(),
            target_label: caps[1].trim().to_string(),
        });
    }

    Ok(parsed)
}

pub fn parse_project_inclusions(content: &str) -> Result<Vec<Inclusion>> {
    let mut inclusions = Vec::new();
    let transclude_re = Regex::new(r"\\transclude(?:\[([^\]]+)\])?\{([^}]+)\}")?;

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

    Ok(inclusions)
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
        if ch != '\\' {
            prev_backslash = false;
        }
    }

    out
}
