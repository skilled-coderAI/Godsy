use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{KbError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Text,
    Markdown,
    Pdf,
    Docx,
    Xlsx,
}

impl DocumentKind {
    fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "txt" | "log" | "csv" | "json" | "yaml" | "yml" | "toml" => Some(Self::Text),
            "md" | "markdown" => Some(Self::Markdown),
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "xlsx" | "xlsm" | "xlsb" | "xls" => Some(Self::Xlsx),
            _ => None,
        }
    }
}

#[must_use]
pub fn supported_extensions() -> &'static [&'static str] {
    &["txt", "md", "markdown", "log", "csv", "json", "yaml", "yml", "toml", "pdf", "docx", "xlsx"]
}

/// Extract plain UTF-8 text from a file path. Returns `(kind, body)`.
pub fn extract_text(path: &Path) -> Result<(DocumentKind, String)> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| KbError::Unsupported(format!("no extension on {}", path.display())))?;
    let kind = DocumentKind::from_extension(ext)
        .ok_or_else(|| KbError::Unsupported(format!("extension .{ext}")))?;
    let body = match kind {
        DocumentKind::Text | DocumentKind::Markdown => std::fs::read_to_string(path)?,
        DocumentKind::Pdf => extract_pdf(path)?,
        DocumentKind::Docx => extract_docx(path)?,
        DocumentKind::Xlsx => extract_xlsx(path)?,
    };
    Ok((kind, body))
}

fn extract_pdf(path: &Path) -> Result<String> {
    pdf_extract::extract_text(path).map_err(|e| KbError::Extract(format!("pdf: {e}")))
}

fn extract_docx(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| KbError::Extract(format!("docx: {e}")))?;
    let mut doc = zip
        .by_name("word/document.xml")
        .map_err(|e| KbError::Extract(format!("docx missing document.xml: {e}")))?;
    let mut xml = String::new();
    doc.read_to_string(&mut xml)?;
    extract_docx_text(&xml)
}

fn extract_docx_text(xml: &str) -> Result<String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = String::new();
    let mut in_text = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:t" => {
                in_text = true;
            }
            Ok(Event::Text(t)) if in_text => {
                let s = t.unescape().map_err(|e| KbError::Extract(format!("docx xml: {e}")))?;
                out.push_str(&s);
            }
            Ok(Event::End(ref e)) => {
                let n = e.name();
                if n.as_ref() == b"w:t" {
                    in_text = false;
                } else if n.as_ref() == b"w:p" {
                    out.push('\n');
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(KbError::Extract(format!("docx xml: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

fn extract_xlsx(path: &Path) -> Result<String> {
    use calamine::{Data, Reader};
    let mut wb: calamine::Sheets<_> = calamine::open_workbook_auto(path)
        .map_err(|e| KbError::Extract(format!("xlsx open: {e}")))?;
    let sheets = wb.sheet_names();
    let mut out = String::new();
    for name in sheets {
        let range = wb
            .worksheet_range(&name)
            .map_err(|e| KbError::Extract(format!("xlsx sheet {name}: {e}")))?;
        out.push_str(&format!("# Sheet: {name}\n"));
        for row in range.rows() {
            let cells: Vec<String> = row
                .iter()
                .map(|c| match c {
                    Data::Empty => String::new(),
                    Data::String(s) => s.clone(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(d) => d.to_string(),
                    Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
                    Data::Error(e) => format!("#ERR:{e:?}"),
                })
                .collect();
            out.push_str(&cells.join("\t"));
            out.push('\n');
        }
        out.push('\n');
    }
    Ok(out)
}

/// Walk a directory recursively, returning every supported file path.
pub fn walk_supported(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk_inner(root, &mut out)?;
    Ok(out)
}

fn walk_inner(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_file() {
        if let Some(ext) = dir.extension().and_then(|e| e.to_str()) {
            if DocumentKind::from_extension(ext).is_some() {
                out.push(dir.to_path_buf());
            }
        }
        return Ok(());
    }
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_inner(&p, out)?;
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if DocumentKind::from_extension(ext).is_some() {
                out.push(p);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ext_mapping_is_case_insensitive() {
        assert_eq!(DocumentKind::from_extension("MD"), Some(DocumentKind::Markdown));
        assert_eq!(DocumentKind::from_extension("Pdf"), Some(DocumentKind::Pdf));
        assert_eq!(DocumentKind::from_extension("zip"), None);
    }

    #[test]
    fn extracts_simple_docx_xml() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="x">
  <w:body>
    <w:p><w:r><w:t>Hello</w:t></w:r></w:p>
    <w:p><w:r><w:t>World</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        let body = extract_docx_text(xml).unwrap();
        assert!(body.contains("Hello"));
        assert!(body.contains("World"));
    }

    #[test]
    fn extracts_text_file() {
        let dir = std::env::temp_dir().join(format!("godsy-kb-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("a.txt");
        std::fs::write(&p, "hello kb").unwrap();
        let (kind, body) = extract_text(&p).unwrap();
        assert_eq!(kind, DocumentKind::Text);
        assert_eq!(body.trim(), "hello kb");
        std::fs::remove_dir_all(&dir).ok();
    }
}
