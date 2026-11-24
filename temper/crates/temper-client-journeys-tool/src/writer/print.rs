//! The writer's paths and report (temper/docs/design/client/journey-writer.md, 2 and 4).
use super::{Stop, decimal, fail};

pub(super) fn paths(status: &[u8]) -> Result<Vec<u8>, Stop> {
    let mut paths = Vec::new();
    for entry in status.split(|&b| b == 0).filter(|entry| !entry.is_empty()) {
        let path = entry.get(3..).unwrap_or_default();
        if !written(path) {
            return Err(fail(&[b"outside the written files: ", &quoted(path)]));
        }
        paths.extend_from_slice(path);
        paths.push(0);
    }
    Ok(paths)
}

fn written(path: &[u8]) -> bool {
    let Some(relative) = path.strip_prefix(b"temper/docs/design/client/") else { return false };
    let (tui, relative) = if let Some(web) = relative.strip_prefix(b"web/journeys/") {
        if matches!(web, b"skein-ui.css" | b"journey.js") {
            return true;
        }
        (false, web)
    } else if let Some(tui) = relative.strip_prefix(b"tui/journeys/") {
        if matches!(tui, b"theme.css" | b"tui-journey.js") {
            return true;
        }
        (true, tui)
    } else {
        return false;
    };
    let Some(slash) = relative.iter().position(|&b| b == b'/') else { return false };
    if slash == 0 {
        return false;
    }
    let file = &relative[slash + 1..];
    file == b"recording"
        || file
            .strip_suffix(b".html")
            .or_else(|| if tui { file.strip_suffix(b".txt") } else { None })
            .is_some_and(|digits| digits.len() >= 2 && decimal(digits))
}

// The path diagnostic uses the script's Python bytes repr, so even newlines and non-UTF-8 bytes stay on one line.
fn quoted(path: &[u8]) -> Vec<u8> {
    let quote = if path.contains(&b'\'') && !path.contains(&b'"') { b'"' } else { b'\'' };
    let mut shown = vec![b'b', quote];
    for &byte in path {
        match byte {
            b'\t' => shown.extend_from_slice(b"\\t"),
            b'\n' => shown.extend_from_slice(b"\\n"),
            b'\r' => shown.extend_from_slice(b"\\r"),
            byte if byte == quote || byte == b'\\' => shown.extend_from_slice(&[b'\\', byte]),
            b' '..=b'~' => shown.push(byte),
            _ => shown.extend_from_slice(format!("\\x{byte:02x}").as_bytes()),
        }
    }
    shown.push(quote);
    shown
}

pub(super) fn body(stdout: &[u8], tip: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    for line in stdout.split(|&b| b == b'\n').filter(|line| line.starts_with(b"stopped ")) {
        let end = line.windows(b"; scratch kept at ".len()).position(|bytes| bytes == b"; scratch kept at ");
        body.extend_from_slice(&line[..end.unwrap_or(line.len())]);
        body.push(b'\n');
    }
    if !body.is_empty() {
        body.push(b'\n');
    }
    body.extend_from_slice(b"Printed: ");
    body.extend_from_slice(tip);
    body
}
