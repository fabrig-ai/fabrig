//! The page and its starter (ui-web.md, 4.3).
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;

use crate::{Digest, Inputs, Refusal, io};

pub(crate) fn starter(name: &str) -> Vec<u8> {
    format!("import init from \"./{name}.js\";\ninit();\n").into_bytes()
}

fn sheets(inputs: &Inputs) -> impl Iterator<Item = &str> {
    std::iter::once("skein-ui.css").chain(
        inputs
            .stylesheets
            .iter()
            .map(|path| path.file_name().and_then(|name| name.to_str()).expect("bundle checked stylesheet name")),
    )
}

pub(crate) fn loader(inputs: &Inputs, files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let row = |name: &str| {
        let plain = files.get(name).expect("bundle holds loader file").len();
        let gzipped = files.get(&format!("{name}.gz")).map_or(plain, Vec::len);
        let name = name.replace('\\', "\\\\").replace('"', "\\\"");
        format!("[\"{name}\", {plain}, {gzipped}]")
    };
    let head = format!(
        "const GLUE = {};\nconst WASM = {};\nconst SHEETS = [{}];\n",
        row(&format!("{}.js", inputs.name)),
        row(&format!("{}_bg.wasm", inputs.name)),
        sheets(inputs).map(row).collect::<Vec<_>>().join(", ")
    );
    [head.as_bytes(), include_str!("../loader.js").as_bytes()].concat()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

pub(crate) fn render(inputs: &Inputs, digest: Digest) -> Result<Vec<u8>, Refusal> {
    let directory = format!("{}{digest}/", inputs.prefix);
    if let Some(path) = &inputs.page {
        let bytes = io(path, fs::read(path))?;
        return replace_marker(&bytes, b"{bundle}", directory.as_bytes())
            .ok_or_else(|| Refusal::NoMarker { page: path.clone() });
    }
    let mut page = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\">\n<title>{}</title>\n",
        escape(inputs.title.as_deref().unwrap_or(&inputs.name))
    );
    let names: Vec<&str> = match &inputs.placeholder {
        Some(placeholder) => vec![
            placeholder.stylesheet.file_name().and_then(|name| name.to_str()).expect("bundle checked stylesheet name"),
        ],
        None => sheets(inputs).collect(),
    };
    for name in names {
        writeln!(page, "<link rel=\"stylesheet\" href=\"{}{}\">", escape(&directory), escape(name))
            .expect("write page");
    }
    writeln!(page, "<script type=\"module\" src=\"{}boot.js\"></script>", escape(&directory)).expect("write page");
    page.push_str("</head>\n<body>\n<div id=\"app\"></div>\n");
    let noscript = format!("<noscript>{}</noscript>", escape(&inputs.noscript));
    let mut page = page.into_bytes();
    if let Some(placeholder) = &inputs.placeholder {
        let markup = io(&placeholder.markup, fs::read(&placeholder.markup))?;
        let markup = replace_marker(&markup, b"{noscript}", noscript.as_bytes())
            .ok_or_else(|| Refusal::NoNoscriptMarker { markup: placeholder.markup.clone() })?;
        page.extend_from_slice(&markup);
    } else {
        page.extend_from_slice(noscript.as_bytes());
    }
    page.extend_from_slice(b"\n</body>\n</html>\n");
    Ok(page)
}

fn replace_marker(bytes: &[u8], marker: &[u8], replacement: &[u8]) -> Option<Vec<u8>> {
    let mut remaining = bytes;
    let mut page = Vec::new();
    let mut found = false;
    while let Some(index) = remaining.windows(marker.len()).position(|part| part == marker) {
        page.extend_from_slice(&remaining[..index]);
        page.extend_from_slice(replacement);
        remaining = &remaining[index + marker.len()..];
        found = true;
    }
    page.extend_from_slice(remaining);
    found.then_some(page)
}
