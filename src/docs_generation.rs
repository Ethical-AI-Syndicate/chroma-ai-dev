use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_html_docs(output_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    fs::create_dir_all(output_dir)?;

    let sources = [
        "TOOLS.md",
        "PROMPTS.md",
        "EVALS.md",
        "AGENTS.md",
        "MCP_SERVERS.md",
        "CLAUDE.md",
    ];

    let mut generated = Vec::new();
    for source in sources {
        let markdown = fs::read_to_string(source)?;
        let output_file = output_dir.join(source.replace(".md", ".html"));
        let html = render_html_document(source, &markdown);
        fs::write(&output_file, html)?;
        generated.push(output_file);
    }

    Ok(generated)
}

fn render_html_document(title: &str, markdown: &str) -> String {
    let escaped = escape_html(markdown);
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>{title}</title>\n  <style>body{{font-family:ui-sans-serif,system-ui,sans-serif;max-width:960px;margin:2rem auto;padding:0 1rem;}}pre{{white-space:pre-wrap;background:#f6f8fa;padding:1rem;border-radius:6px;}}</style>\n</head>\n<body>\n  <h1>{title}</h1>\n  <pre>{escaped}</pre>\n</body>\n</html>\n"
    )
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
