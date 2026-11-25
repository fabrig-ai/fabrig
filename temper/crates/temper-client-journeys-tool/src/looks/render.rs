//! The lines from look-render (temper/docs/design/client/journey-writer.md, 6.2, step 8).

use std::fmt::Write as _;

use super::{RESULTS, escaped};

pub(crate) struct Render {
    pub(super) result: String,
    pages: Vec<Page>,
}

struct Page {
    path: String,
    report: Report,
}

enum Report {
    Identical,
    Uncompared(String),
    Differences { rows: Vec<[String; 5]>, more: Option<u32> },
}

impl Render {
    pub(crate) fn read(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "look-render output is not UTF-8")?;
        let mut pages = Vec::new();
        let mut lines = text.split_terminator('\n').peekable();
        while let Some(line) = lines.next() {
            let line =
                line.split('\t').map(unescape).collect::<Option<Vec<_>>>().ok_or("invalid look-render escape")?;
            if let [kind, result] = line.as_slice()
                && kind == "result"
            {
                if RESULTS.contains(&result.as_str()) && lines.peek().is_none() {
                    return Ok(Self { result: result.clone(), pages });
                }
                return Err("invalid final look-render result".into());
            }
            Self::read_page(&mut pages, &line)?;
        }
        Err("missing final look-render result".into())
    }

    fn read_page(pages: &mut Vec<Page>, line: &[String]) -> Result<(), String> {
        let [kind, path, fields @ ..] = line else { return Err("invalid look-render line".into()) };
        if page_path(path).is_none() {
            return Err("invalid look-render page path".into());
        }
        let report = match (kind.as_str(), fields) {
            ("identical", []) => Report::Identical,
            ("uncompared", [why]) => Report::Uncompared(why.clone()),
            ("differs", [width, element, property, requested, landed]) => {
                let row = [width.clone(), element.clone(), property.clone(), requested.clone(), landed.clone()];
                if let Some(page) = pages.last_mut()
                    && page.path == *path
                {
                    if let Report::Differences { rows, more: None } = &mut page.report {
                        rows.push(row);
                        return Ok(());
                    }
                    return Err("unexpected look-render difference".into());
                }
                Report::Differences { rows: vec![row], more: None }
            }
            ("more", [count]) => {
                if let Some(page) = pages.last_mut()
                    && page.path == *path
                    && let Report::Differences { more, .. } = &mut page.report
                    && more.is_none()
                    && let Ok(count) = count.parse()
                {
                    *more = Some(count);
                    return Ok(());
                }
                return Err("unexpected look-render more line".into());
            }
            _ => return Err("invalid look-render page line".into()),
        };
        pages.push(Page { path: path.clone(), report });
        Ok(())
    }

    pub(super) fn html(&self, html: &mut String, tui: bool) {
        for page in &self.pages {
            let (page_is_tui, path) = page_path(&page.path).expect("read validated the path");
            let other_root = match (tui, page_is_tui) {
                (true, false) => "../../web/journeys/",
                (false, true) => "../../tui/journeys/",
                _ => "",
            };
            write!(html, "<h3><a href=\"{other_root}{}\">{}</a></h3>", url_path(path), escaped(&page.path))
                .expect("write to String");
            match &page.report {
                Report::Identical => html.push_str("<p>identical</p>"),
                Report::Uncompared(why) => {
                    write!(html, "<p>not compared: {}</p>", escaped(why)).expect("write to String");
                }
                Report::Differences { rows, more } => table(html, rows, *more),
            }
        }
    }
}

fn table(html: &mut String, rows: &[[String; 5]], more: Option<u32>) {
    html.push_str("<table><thead><tr><th>Width</th><th>Element</th><th>Property</th><th>Requested</th><th>Landed</th></tr></thead><tbody>");
    for row in rows {
        html.push_str("<tr>");
        for value in row {
            write!(html, "<td>{}</td>", escaped(value)).expect("write to String");
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    if let Some(more) = more {
        write!(html, "<p>{more} more</p>").expect("write to String");
    }
}

fn unescape(value: &str) -> Option<String> {
    let mut text = String::new();
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        text.push(match character {
            '\\' => match characters.next()? {
                '\\' => '\\',
                't' => '\t',
                'r' => '\r',
                'n' => '\n',
                _ => return None,
            },
            '\r' => return None,
            other => other,
        });
    }
    Some(text)
}

fn page_path(path: &str) -> Option<(bool, &str)> {
    let path = path
        .strip_prefix("web/journeys/")
        .map(|path| (false, path))
        .or_else(|| path.strip_prefix("tui/journeys/").map(|path| (true, path)))?;
    let (name, file) = path.1.split_once('/')?;
    let number = file.strip_suffix(".html")?;
    (!matches!(name, "" | "." | "..") && number.len() >= 2 && number.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some(path)
}

fn url_path(path: &str) -> String {
    let mut url = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'/') {
            url.push(char::from(byte));
        } else {
            write!(url, "%{byte:02X}").expect("write to String");
        }
    }
    url
}
