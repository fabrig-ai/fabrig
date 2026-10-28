//! `temper help` (protocol/documentation.md, section 4).
use std::fmt::Write;
use temper_docs::PAGES;

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Index,
    Page(Box<[u8]>),
    Search(Box<[u8]>),
}

/// Its words after `help`: none, a NAME, or `--search` and words
/// (joined with one space); anything else is None.
#[must_use]
pub fn request(words: &[Box<[u8]>]) -> Option<Request> {
    match words {
        [] => Some(Request::Index),
        [flag, rest @ ..] if flag.as_ref() == b"--search" && !rest.is_empty() => {
            Some(Request::Search(rest.join(&b' ').into_boxed_slice()))
        }
        [name] if !name.is_empty() && !name.starts_with(b"-") => Some(Request::Page(name.clone())),
        _ => None,
    }
}

/// What it prints on stdout, or its error line for stderr.
pub fn answer(request: &Request) -> Result<String, String> {
    match request {
        Request::Index => Ok(temper_docs::find(b"index").expect("compiled index page").text.into()),
        Request::Page(name) => match temper_docs::find(name) {
            Some(topic) => Ok(topic.text.into()),
            None => Err(format!("temper: no page {}; temper help lists them\n", String::from_utf8_lossy(name))),
        },
        Request::Search(words) => {
            let found = temper_docs::search(&PAGES, words, 20).map_err(|why| format!("temper: {}\n", why.says()))?;
            if found.hits.is_empty() {
                return Err("temper: nothing matches\n".into());
            }
            let mut text = String::new();
            for hit in found.hits.as_slice() {
                writeln!(text, "{}: {}: {}", hit.page, hit.heading, hit.line).expect("writing into a string");
            }
            if found.more {
                text.push_str("and more; narrow the words\n");
            }
            Ok(text)
        }
    }
}
