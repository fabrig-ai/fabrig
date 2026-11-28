//! The render check's browser effects (temper/docs/design/client/journey-writer-requests.md, 1.2).

use std::path::PathBuf;

use skein_browser::boundary::{Element, Node, Property, Rect, Refusal, Trouble};
use skein_lib::Token;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Start { profile: PathBuf },
    Person { person: Token },
    Open { person: Token, page: Token, url: Box<[u8]>, files: bool },
    Viewport { page: Token, width: u32, height: u32 },
    Elements { page: Token, selector: Option<Box<[u8]>> },
    Style { page: Token, node: Node },
    Rect { page: Token, node: Node },
    Close { page: Token },
    Stop,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Answer {
    pub(crate) reply: Reply,
    pub(crate) trouble: Vec<Observation>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Reply {
    Done,
    Elements { elements: Vec<Element>, more: u32 },
    Style(Vec<Property>),
    Rect(Option<Rect>),
    Refused { why: Refusal, detail: Box<[u8]> },
    Failed(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Observation {
    pub(crate) page: Token,
    pub(crate) kind: Trouble,
    pub(crate) text: Box<[u8]>,
    pub(crate) url: Box<[u8]>,
}
