//! A frame's text and linked page (temper/docs/design/client/tui/journeys.md, 6.2 to 6.4).
use crate::print::Printed;
use skein_ui_screen::{Frame, Paint, Position, Rgb};
use skein_ui_screen_terminal::Key;
use std::{fmt::Write, fs, io, path::Path};
use temper_client_shell_tui::journey::Mark;
use temper_client_tui::{
    Scheme,
    theme::{palette, theme},
};

pub const SCRIPT: &str = include_str!("../../tui-journey.js");

#[must_use]
pub fn pages(name: &str, scheme: Scheme, marks: &[Mark]) -> Vec<Printed> {
    let mut files = Vec::new();
    for (index, mark) in marks.iter().enumerate() {
        let number = index + 1;
        let mut html = format!(
            "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{} {number:02} · temper</title><link rel=\"stylesheet\" href=\"../theme.css\"><link rel=\"stylesheet\" href=\"../overlay.css\">",
            escape(name)
        );
        if index > 0 {
            write!(html, "<link rel=\"prev\" href=\"{index:02}.html\">").expect("write string");
        }
        if number < marks.len() {
            write!(html, "<link rel=\"next\" href=\"{:02}.html\">", number + 1).expect("write string");
        }
        write!(
            html,
            "<script src=\"../tui-journey.js\" defer></script></head><body><pre class=\"tui tui-{}\"",
            scheme_name(scheme)
        )
        .expect("write string");
        if let Some(key) = &mark.key {
            assert!(number < marks.len(), "a key's mark has a next page");
            write!(
                html,
                " data-journey-key=\"{}\" data-journey-next=\"{:02}.html\"",
                escape(&key_name(key)),
                number + 1
            )
            .expect("write string");
        }
        // The newline immediately after pre is stripped by HTML, keeping row zero.
        html.push_str(">\n");
        frame(&mut html, &mark.frame, scheme);
        html.push_str("</pre></body></html>\n");
        files.push(Printed {
            name: format!("{number:02}.txt"),
            document: String::from_utf8(mark.frame.text().into()).expect("frame text is UTF-8"),
        });
        files.push(Printed { name: format!("{number:02}.html"), document: html });
    }
    files
}

fn scheme_name(scheme: Scheme) -> &'static str {
    match scheme {
        Scheme::Dark => "dark",
        Scheme::Light => "light",
    }
}

fn frame(html: &mut String, frame: &Frame, scheme: Scheme) {
    let base = theme(scheme, None).base;
    for row in 0..frame.size().rows {
        let cells = frame.row(row).expect("row in frame");
        let mut start = 0;
        while start < cells.len() {
            let paint = cells[start].paint;
            let end = start + cells[start..].iter().take_while(|cell| cell.paint == paint).count();
            if paint != base {
                html.push_str(&paint_span(paint, scheme));
            }
            for (offset, cell) in cells[start..end].iter().enumerate() {
                if cell.glyph == '\0' {
                    continue;
                }
                let cursor = frame.cursor()
                    == Some(Position { row, column: u16::try_from(start + offset).expect("column fits u16") });
                if cursor {
                    html.push_str("<span class=\"cursor\">");
                }
                html.push_str(&escape(&cell.glyph.to_string()));
                if cursor {
                    html.push_str("</span>");
                }
            }
            if paint != base {
                html.push_str("</span>");
            }
            start = end;
        }
        html.push('\n');
    }
}

fn paint_span(paint: Paint, scheme: Scheme) -> String {
    let mut classes = Vec::new();
    let mut styles = Vec::new();
    for (colour, prefix, property) in [(paint.fore, "f", "color"), (paint.back, "b", "background")] {
        if let Some(colour) = colour {
            if let Some((name, _)) = palette(scheme).into_iter().find(|(_, rgb)| *rgb == colour) {
                classes.push(format!("{prefix}-{name}"));
            } else {
                styles.push(format!("{property}:{}", hex(colour)));
                if paint.reverse {
                    styles.push(format!("--tui-{prefix}:{}", hex(colour)));
                }
            }
        }
    }
    for (set, name) in [
        (paint.bold, "bold"),
        (paint.dim, "dim"),
        (paint.italic, "italic"),
        (paint.underline, "underline"),
        (paint.reverse, "reverse"),
    ] {
        if set {
            classes.push(name.to_owned());
        }
    }
    let mut span = String::from("<span");
    if !classes.is_empty() {
        write!(span, " class=\"{}\"", classes.join(" ")).expect("write string");
    }
    if !styles.is_empty() {
        write!(span, " style=\"{}\"", styles.join(";")).expect("write string");
    }
    span.push('>');
    span
}

fn hex(rgb: Rgb) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb.red, rgb.green, rgb.blue)
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn key_name(key: &Key) -> String {
    match key {
        Key::Enter => "enter".into(),
        Key::Tab => "tab".into(),
        Key::BackTab => "backtab".into(),
        Key::Backspace => "backspace".into(),
        Key::Delete => "delete".into(),
        Key::Up => "up".into(),
        Key::Down => "down".into(),
        Key::Left => "left".into(),
        Key::Right => "right".into(),
        Key::Home => "home".into(),
        Key::End => "end".into(),
        Key::PageUp => "pageup".into(),
        Key::PageDown => "pagedown".into(),
        Key::Escape => "escape".into(),
        Key::Char(ch) => ch.to_string(),
        Key::Control(byte) => format!("control {}", char::from(*byte)),
        Key::Alt(ch) => format!("alt {ch}"),
        Key::Paste(bytes) => format!("paste {}", String::from_utf8_lossy(bytes)),
    }
}

#[must_use]
pub fn stylesheet() -> String {
    let mut css = String::new();
    for scheme in [Scheme::Dark, Scheme::Light] {
        writeln!(css, ".tui-{} {{", scheme_name(scheme)).expect("write string");
        for (name, colour) in palette(scheme) {
            writeln!(css, "  --{name}: {};", hex(colour)).expect("write string");
        }
        css.push_str("}\n");
    }
    css.push_str("pre.tui { color: var(--ink); background: var(--bg); font-family: monospace; line-height: 1; width: max-content; }\n");
    for (name, _) in palette(Scheme::Dark) {
        writeln!(css, ".f-{name} {{ --tui-f: var(--{name}); color: var(--tui-f); }}\n.b-{name} {{ --tui-b: var(--{name}); background: var(--tui-b); }}").expect("write string");
    }
    css.push_str(".bold { font-weight: bold; }\n.dim { opacity: .65; }\n.italic { font-style: italic; }\n.underline { text-decoration: underline; }\n.reverse { color: var(--tui-b, var(--bg)) !important; background: var(--tui-f, var(--ink)) !important; }\n.cursor { outline: 1px solid currentColor; }\n");
    css
}

pub fn beside(root: &Path) -> io::Result<()> {
    fs::write(root.join("theme.css"), stylesheet())?;
    fs::write(root.join("tui-journey.js"), SCRIPT)?;
    match fs::OpenOptions::new().write(true).create_new(true).open(root.join("overlay.css")) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests;
