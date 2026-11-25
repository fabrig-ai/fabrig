//! Render reads issued as effects (temper/docs/design/client/journey-writer.md, 6.2, steps 4–7).

use skein_browser::boundary::{Element, Node, Property, Rect, Refusal};
use skein_lib::Token;

use crate::host::{
    Answer, Ask, Host,
    browser::{Command, Reply},
};

pub(super) const PERSON: Token = Token::new(1);

pub(super) struct Browser<'a> {
    pub(super) host: &'a mut Host,
    pub(super) item: String,
    next_page: u64,
}

impl<'a> Browser<'a> {
    pub(super) fn new(host: &'a mut Host) -> Self {
        Self { host, item: String::new(), next_page: PERSON.raw() + 1 }
    }

    fn ask(&mut self, command: Command) -> Reply {
        let Answer::Browser(answer) = self.host.ask(Ask::Browser(command)) else {
            unreachable!("the host answers its browser requests")
        };
        for observed in answer.trouble {
            let text = String::from_utf8_lossy(&observed.text);
            let url = String::from_utf8_lossy(&observed.url);
            let page = &self.item;
            let kind = observed.kind;
            let location = if url.is_empty() { String::new() } else { format!(" ({url})") };
            self.host.err(format!("look-render: {page}: {kind:?}: {text}{location}\n").as_bytes());
        }
        answer.reply
    }

    pub(super) fn done(&mut self, command: Command) -> Result<(), String> {
        match self.ask(command) {
            Reply::Done => Ok(()),
            other => Err(error(other)),
        }
    }

    pub(super) fn open(&mut self, url: Box<[u8]>) -> Result<Token, String> {
        let page = Token::new(self.next_page);
        self.next_page += 1;
        self.done(Command::Open { person: PERSON, page, url, files: true })?;
        Ok(page)
    }

    pub(super) fn elements(&mut self, page: Token, selector: Option<&str>) -> Result<(Vec<Element>, u32), String> {
        let filtering = selector.is_some();
        let selector = selector.map(|selector| selector.as_bytes().into());
        match self.ask(Command::Elements { page, selector }) {
            Reply::Elements { elements, more } => Ok((elements, more)),
            Reply::Refused { why: Refusal::Protocol, detail }
                if filtering
                    && detail.starts_with(b"DOM.querySelectorAll: ")
                    && detail.as_ref() != b"DOM.querySelectorAll: malformed reply" =>
            {
                Ok((Vec::new(), 0))
            }
            other => Err(error(other)),
        }
    }

    pub(super) fn style(&mut self, page: Token, node: Node) -> Result<Vec<Property>, String> {
        match self.ask(Command::Style { page, node }) {
            Reply::Style(style) => Ok(style),
            other => Err(error(other)),
        }
    }

    pub(super) fn rect(&mut self, page: Token, node: Node) -> Result<Option<Rect>, String> {
        match self.ask(Command::Rect { page, node }) {
            Reply::Rect(rect) => Ok(rect),
            other => Err(error(other)),
        }
    }
}

fn error(reply: Reply) -> String {
    match reply {
        Reply::Failed(why) => why,
        Reply::Refused { why, detail } if !detail.is_empty() => {
            format!("Chromium refused a read ({why:?}): {}", String::from_utf8_lossy(&detail))
        }
        Reply::Refused { why, .. } => format!("Chromium refused a read ({why:?})"),
        reply => unreachable!("unexpected browser reply {reply:?}"),
    }
}
