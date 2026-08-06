use super::*;

const HTML_MATH_SVG_SCALE: f64 = 0.75;

const HTML_CSS_OVERRIDES: &str = "/* zetteltex-html-overrides */\n\
@font-face {\n\
    font-family: 'ZTX-LM-Roman';\n\
    src: url('fonts/lmroman10-regular.otf') format('opentype');\n\
    font-weight: 400;\n\
    font-style: normal;\n\
    font-display: swap;\n\
}\n\
@font-face {\n\
    font-family: 'ZTX-LM-Roman';\n\
    src: url('fonts/lmroman10-italic.otf') format('opentype');\n\
    font-weight: 400;\n\
    font-style: italic;\n\
    font-display: swap;\n\
}\n\
@font-face {\n\
    font-family: 'ZTX-LM-Roman';\n\
    src: url('fonts/lmroman10-bold.otf') format('opentype');\n\
    font-weight: 700;\n\
    font-style: normal;\n\
    font-display: swap;\n\
}\n\
@font-face {\n\
    font-family: 'ZTX-LM-Roman';\n\
    src: url('fonts/lmroman10-bolditalic.otf') format('opentype');\n\
    font-weight: 700;\n\
    font-style: italic;\n\
    font-display: swap;\n\
}\n\
body {\n\
    max-width: 980px;\n\
    margin: 2.5rem auto;\n\
    padding: 0 1.5rem;\n\
    line-height: 1.5;\n\
    font-family: 'ZTX-LM-Roman', 'Latin Modern Roman', 'Computer Modern', 'CMU Serif', serif;\n\
}\n\
dl.enumerate-enumitem {\n\
    margin: 0.35rem 0 0.8rem;\n\
    display: grid;\n\
    grid-template-columns: max-content 1fr;\n\
    column-gap: 0.6rem;\n\
    row-gap: 0.25rem;\n\
}\n\
dl.enumerate-enumitem > dt {\n\
    margin: 0;\n\
    padding: 0;\n\
}\n\
dl.enumerate-enumitem > dd {\n\
    margin: 0;\n\
    padding: 0;\n\
}\n\
ol, ul {\n\
    margin: 0.75rem 0 1rem;\n\
    padding-left: 2.2rem;\n\
    list-style-position: outside;\n\
}\n\
ol li, ul li {\n\
    margin: 0.35rem 0;\n\
}\n\
ol li > p, ul li > p {\n\
    margin: 0;\n\
    display: inline;\n\
}\n\
ol li > p + p, ul li > p + p {\n\
    display: block;\n\
    margin-top: 0.6rem;\n\
}\n\
ol li > br, ul li > br {\n\
    display: none;\n\
}\n\
p {\n\
    margin: 0.6rem 0;\n\
}\n";

pub fn postprocess_html_output(paths: &WorkspacePaths) -> Result<()> {
    copy_html_fonts(paths)?;
    copy_html_resources(paths)?;
    let output_dir = html_output_dir(paths);
    append_html_css_overrides(&output_dir)?;
    inline_html_css_overrides(&output_dir)?;
    rewrite_html_asset_paths(&output_dir)?;
    scale_html_math_svgs(&output_dir)
}

fn scale_html_math_svgs(output_dir: &Path) -> Result<()> {
    if (HTML_MATH_SVG_SCALE - 1.0).abs() < f64::EPSILON {
        return Ok(());
    }

    let mut svg_paths = BTreeSet::new();
    let regexes = html_math_svg_regexes();
    let img_re = &regexes[0];
    let alt_re = &regexes[1];
    let src_re = &regexes[2];
    let mut stack = vec![output_dir.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("html") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            for mat in img_re.find_iter(&content) {
                let tag = mat.as_str();
                let Some(alt_caps) = alt_re.captures(tag) else {
                    continue;
                };
                let alt = alt_caps
                    .name("alt_dq")
                    .or_else(|| alt_caps.name("alt_sq"))
                    .map(|m| m.as_str())
                    .unwrap_or("");
                if !alt.contains('$') {
                    continue;
                }
                let Some(src_caps) = src_re.captures(tag) else {
                    continue;
                };
                let src = src_caps
                    .name("src_dq")
                    .or_else(|| src_caps.name("src_sq"))
                    .map(|m| m.as_str())
                    .unwrap_or("");
                if src.is_empty() || src.starts_with("http") || src.starts_with("data:") {
                    continue;
                }
                if !src.ends_with(".svg") {
                    continue;
                }

                let src_path = Path::new(src);
                let resolved = if src_path.is_absolute() {
                    src_path.to_path_buf()
                } else {
                    path.parent().unwrap_or(output_dir).join(src_path)
                };
                if resolved.exists() {
                    svg_paths.insert(resolved);
                }
            }
        }
    }

    for svg_path in svg_paths {
        scale_svg_file(&svg_path)?;
    }

    Ok(())
}

fn html_math_svg_regexes() -> &'static [Regex; 3] {
    static RE: OnceLock<[Regex; 3]> = OnceLock::new();
    RE.get_or_init(|| {
        [
            Regex::new(r"(?is)<img\b[^>]*>").expect("regex img valida"),
            Regex::new(r#"(?is)\balt\s*=\s*(?:\"(?P<alt_dq>[^\"]*)\"|'(?P<alt_sq>[^']*)')"#)
                .expect("regex alt valida"),
            Regex::new(r#"(?is)\bsrc\s*=\s*(?:\"(?P<src_dq>[^\"]*)\"|'(?P<src_sq>[^']*)')"#)
                .expect("regex src valida"),
        ]
    })
}

fn scale_svg_file(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    if content.contains("data-ztx-scale=") {
        return Ok(());
    }

    let regexes = svg_root_regexes();
    let svg_re = &regexes[0];
    let width_re = &regexes[1];
    let height_re = &regexes[2];
    let Some(mat) = svg_re.find(&content) else {
        return Ok(());
    };
    let tag = mat.as_str();

    let scaled_tag = scale_svg_tag(tag, width_re, height_re, HTML_MATH_SVG_SCALE);
    let Some(updated_tag) = scaled_tag else {
        return Ok(());
    };

    let mut updated = String::with_capacity(content.len() + 32);
    updated.push_str(&content[..mat.start()]);
    updated.push_str(&updated_tag);
    updated.push_str(&content[mat.end()..]);
    fs::write(path, updated)?;
    Ok(())
}

fn svg_root_regexes() -> &'static [Regex; 3] {
    static RE: OnceLock<[Regex; 3]> = OnceLock::new();
    RE.get_or_init(|| {
        [
            Regex::new(r"(?is)<svg\b[^>]*>").expect("regex svg valida"),
            Regex::new(
                r#"(?is)\bwidth\s*=\s*(?:\"(?P<value_dq>[0-9.]+)(?P<unit_dq>[a-z%]*)\"|'(?P<value_sq>[0-9.]+)(?P<unit_sq>[a-z%]*)')"#,
            )
            .expect("regex width valida"),
            Regex::new(
                r#"(?is)\bheight\s*=\s*(?:\"(?P<value_dq>[0-9.]+)(?P<unit_dq>[a-z%]*)\"|'(?P<value_sq>[0-9.]+)(?P<unit_sq>[a-z%]*)')"#,
            )
            .expect("regex height valida"),
        ]
    })
}

fn scale_svg_tag(tag: &str, width_re: &Regex, height_re: &Regex, scale: f64) -> Option<String> {
    let mut updated = width_re
        .replace(tag, |caps: &regex::Captures| {
            let value = caps
                .name("value_dq")
                .or_else(|| caps.name("value_sq"))
                .map(|m| m.as_str())
                .unwrap_or("0");
            let unit = caps
                .name("unit_dq")
                .or_else(|| caps.name("unit_sq"))
                .map(|m| m.as_str())
                .unwrap_or("");
            let quote = caps.name("value_dq").map(|_| "\"").unwrap_or("'");
            let Ok(parsed) = value.parse::<f64>() else {
                return caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
            };
            let scaled = parsed * scale;
            let scaled_value = format_svg_number(scaled);
            format!("width={}{}{}{}", quote, scaled_value, unit, quote)
        })
        .to_string();

    updated = height_re
        .replace(&updated, |caps: &regex::Captures| {
            let value = caps
                .name("value_dq")
                .or_else(|| caps.name("value_sq"))
                .map(|m| m.as_str())
                .unwrap_or("0");
            let unit = caps
                .name("unit_dq")
                .or_else(|| caps.name("unit_sq"))
                .map(|m| m.as_str())
                .unwrap_or("");
            let quote = caps.name("value_dq").map(|_| "\"").unwrap_or("'");
            let Ok(parsed) = value.parse::<f64>() else {
                return caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
            };
            let scaled = parsed * scale;
            let scaled_value = format_svg_number(scaled);
            format!("height={}{}{}{}", quote, scaled_value, unit, quote)
        })
        .to_string();

    if updated == tag {
        return None;
    }

    if !updated.contains("data-ztx-scale=") {
        let scale_label = format_svg_number(scale);
        if let Some(idx) = updated.find("<svg") {
            let insert_at = idx + "<svg".len();
            let mut out = String::with_capacity(updated.len() + 32);
            out.push_str(&updated[..insert_at]);
            out.push_str(" data-ztx-scale=\"");
            out.push_str(&scale_label);
            out.push('"');
            out.push_str(&updated[insert_at..]);
            updated = out;
        }
    }

    Some(updated)
}

fn format_svg_number(value: f64) -> String {
    let mut out = format!("{:.6}", value);
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    out
}

fn copy_html_resources(paths: &WorkspacePaths) -> Result<()> {
    let output_dir = html_output_dir(paths);
    let resources_src = paths.root.join("resources");
    let figures_src = paths.root.join("notes").join("figures");
    let resources_dst = output_dir.join("resources");
    let figures_dst = output_dir.join("notes").join("figures");

    copy_dir_recursive(&resources_src, &resources_dst)?;
    copy_dir_recursive(&figures_src, &figures_dst)?;
    Ok(())
}

fn copy_html_fonts(paths: &WorkspacePaths) -> Result<()> {
    if !command_exists("kpsewhich") {
        return Ok(());
    }

    let output_dir = html_output_dir(paths);
    let fonts_dst = output_dir.join("fonts");
    let fonts = [
        "lmroman10-regular.otf",
        "lmroman10-italic.otf",
        "lmroman10-bold.otf",
        "lmroman10-bolditalic.otf",
    ];

    for font in fonts {
        if let Some(src_path) = kpsewhich_path(font) {
            fs::create_dir_all(&fonts_dst)?;
            let dst_path = fonts_dst.join(font);
            let _ = fs::copy(src_path, dst_path);
        }
    }

    Ok(())
}

fn kpsewhich_path(name: &str) -> Option<PathBuf> {
    let output = Command::new("kpsewhich").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout);
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn rewrite_html_asset_paths(output_dir: &Path) -> Result<()> {
    let mut stack = vec![output_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if ext != "html" && ext != "css" {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let updated = content
                .replace("../../resources/", "resources/")
                .replace("../../notes/figures/", "notes/figures/");
            if updated != content {
                fs::write(&path, updated)?;
            }
        }
    }
    Ok(())
}

fn append_html_css_overrides(output_dir: &Path) -> Result<()> {
    let mut stack = vec![output_dir.to_path_buf()];
    let overrides = format!("\n{}", HTML_CSS_OVERRIDES);

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("css") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            if content.contains("zetteltex-html-overrides") {
                continue;
            }
            let mut updated = String::with_capacity(content.len() + overrides.len());
            updated.push_str(&content);
            updated.push_str(&overrides);
            fs::write(&path, updated)?;
        }
    }

    Ok(())
}

fn inline_html_css_overrides(output_dir: &Path) -> Result<()> {
    let mut stack = vec![output_dir.to_path_buf()];
        let style_block = format!("<style>\n{}</style>\n", HTML_CSS_OVERRIDES);

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("html") {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            if content.contains("zetteltex-html-overrides") {
                continue;
            }

            let updated = if let Some(idx) = content.find("</head>") {
                let mut out = String::with_capacity(content.len() + style_block.len());
                out.push_str(&content[..idx]);
                out.push_str(&style_block);
                out.push_str(&content[idx..]);
                out
            } else if let Some(idx) = content.find("<head>") {
                let insert_at = idx + "<head>".len();
                let mut out = String::with_capacity(content.len() + style_block.len());
                out.push_str(&content[..insert_at]);
                out.push('\n');
                out.push_str(&style_block);
                out.push_str(&content[insert_at..]);
                out
            } else {
                let mut out = String::with_capacity(content.len() + style_block.len());
                out.push_str(&style_block);
                out.push_str(&content);
                out
            };

            fs::write(&path, updated)?;
        }
    }

    Ok(())
}
