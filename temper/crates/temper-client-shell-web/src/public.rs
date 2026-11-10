//! Anonymous browser host, without a client or store (temper/docs/design/client/web/public.md, 2).
use crate::{host::Duties, limits};
use jig_client_protocol_http::{self as http, PublicHeard};
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{Duration, List, Queue, Time, Wall};
use skein_ui::{Act, NodeId, Patch};
use skein_ui_web::{
    fetch::Cache,
    page::{Op, Page},
};
use std::collections::BTreeMap;
use temper_client_domain::{
    Place,
    address::{Public, parse, print_public},
    public::{Doc, Heard as Answer, PublicEvent, PublicLimits, PublicRead, Reader},
};
use temper_client_view::{
    public::{self as view, PublicView},
    sign_in::Ways,
    words::{Shell, Words},
};

#[cfg(target_arch = "wasm32")]
pub(crate) mod web;

pub enum Report {
    Start,
    Due,
    Act { node: NodeId, act: Act },
    Place { path: Box<[u8]> },
    Heard(Heard),
    Visible { visible: bool },
}
/// Only public GETs cross this host's boundary.
pub struct PublicHost {
    reader: Reader,
    view: PublicView,
    pub bounds: skein_ui::Limits,
    now: Time,
    wall: Wall,
    out: Queue<PublicRead>,
    patches: Queue<Patch>,
    mapping: http::Mapping,
    tickets: BTreeMap<u32, Doc>,
    next: u32,
    show: Option<(Box<[u8]>, bool)>,
}
impl PublicHost {
    #[must_use]
    pub fn new(now: Time, wall: Wall, ways: Ways) -> Self {
        let bounds = bounds();
        let words = Words::english(temper_client_protocol_wire::FORGE, limits::CONVERSATION_TEXT, Shell::Browser)
            .expect("public words fit");
        Self {
            reader: Reader::new(&PublicLimits {
                jig: jig_client_protocol_wire::public_v1::CEILINGS,
                forge: temper_client_protocol_wire::public_v1::CEILINGS,
            }),
            view: PublicView::new(&bounds, words, ways),
            bounds,
            now,
            wall,
            out: Queue::uncapped(),
            patches: Queue::uncapped(),
            mapping: http::Mapping::new(b"/jig").expect("public mapping"),
            tickets: BTreeMap::new(),
            next: 0,
            show: None,
        }
    }
    pub fn start(&mut self, path: &[u8]) {
        self.opened(path, true);
    }
    pub fn clocks(&mut self, now: Time, wall: Wall) {
        self.now = now;
        self.wall = wall;
    }
    fn opened(&mut self, path: &[u8], replace: bool) {
        let page = match parse(path) {
            Place::Public(page) => page,
            Place::Approve { .. }
            | Place::Address(_)
            | Place::Authorize { .. }
            | Place::AuthorizeMalformed
            | Place::Missing => Public::Projects,
        };
        self.tickets.clear();
        while self.out.pop().is_some() {}
        let printed = print_public(&page);
        if printed.as_ref() != path || !replace {
            self.show = Some((printed, replace));
        }
        self.event(PublicEvent::Open { page });
    }
    fn event(&mut self, event: PublicEvent) {
        self.reader.step(self.now, event, &mut self.out);
    }
    pub fn report(&mut self, report: Report) -> Duties {
        match report {
            Report::Start => {}
            Report::Due => self.event(PublicEvent::Due),
            Report::Visible { visible } => self.event(PublicEvent::Visible { visible }),
            Report::Place { path } => self.opened(&path, true),
            Report::Act { node, act } => {
                if let Some(PublicEvent::Open { page }) = view::act(&mut self.view, node, act) {
                    self.opened(&print_public(&page), false);
                }
            }
            Report::Heard(heard) => self.heard(heard),
        }
        self.event(PublicEvent::Due);
        let mut asked = Vec::new();
        while let Some(read) = self.out.pop() {
            self.next = self.next.checked_add(1).expect("public exchange tickets fit");
            self.tickets.insert(self.next, read.doc);
            let ids = List::from_boxed(read.ids.into_iter().flatten().collect());
            asked.push(Asked::Exchange {
                id: self.next,
                call: http::public(
                    &self.mapping,
                    &http::PublicPath { owner: read.owner, version: read.version, kind: read.kind, ids },
                ),
                bound: limits::BODY_BYTES,
                cache: Cache::NoStore,
                keep: http::PUBLIC_KEEP,
            });
        }
        Duties { asked, show: self.show.take(), ..Duties::default() }
    }
    fn heard(&mut self, heard: Heard) {
        let (exchange, answer) = match heard {
            Heard::Answered { exchange, status, body, headers } => {
                let answer = match http::public_answered(status, body, &headers) {
                    PublicHeard::Document { body, fresh } => {
                        Answer::Document { body, fresh: Duration::from_secs(u64::from(fresh)) }
                    }
                    PublicHeard::Unknown { fresh } => Answer::Unknown { fresh: Duration::from_secs(u64::from(fresh)) },
                    PublicHeard::Busy { retry_after } => {
                        Answer::Busy { retry: Duration::from_secs(u64::from(retry_after.unwrap_or(1))) }
                    }
                    PublicHeard::Lost => Answer::Failed,
                };
                (exchange, answer)
            }
            Heard::Failed { exchange } => (exchange, Answer::Failed),
            Heard::StreamOpened
            | Heard::StreamRefused { .. }
            | Heard::Piece { .. }
            | Heard::StreamEnded
            | Heard::Stored { .. }
            | Heard::NotStored { .. } => return,
        };
        if let Some(doc) = self.tickets.remove(&exchange) {
            self.event(PublicEvent::Heard { doc, heard: answer });
            if matches!(self.reader.shown(), temper_client_domain::public::Shown::Unknown) {
                self.tickets.clear();
            }
        }
    }
    pub fn draw(&mut self, page: &mut Page<'_>, ops: &mut Vec<Op>) {
        view::build_public(&mut self.view, &self.reader, self.wall, &mut self.patches);
        let mut patches = Vec::new();
        while let Some(patch) = self.patches.pop() {
            patches.push(patch);
        }
        page.apply(&patches, ops);
        for node in page.links().to_vec() {
            if let Some(path) = view::href(&mut self.view, node) {
                page.set_href(node, &path, ops);
            }
        }
    }
    #[must_use]
    pub fn tree(&self) -> &skein_ui::Tree {
        self.view.tree()
    }
    #[must_use]
    pub fn deadline(&self) -> Option<Time> {
        let view = self.view.next_change().map(|at| {
            self.now.saturating_add(Duration::from_nanos(at.as_nanos().saturating_sub(self.wall.as_nanos())))
        });
        skein_lib::earliest(self.reader.deadline(), view)
    }
}
fn bounds() -> skein_ui::Limits {
    let jig = jig_client_protocol_wire::public_v1::CEILINGS;
    // Every live and ended goal can carry its own maximum tree. A line draws
    // at most sixteen nodes and 1024 bytes; queue and landing rows fit that
    // allowance too. Projects and forge rows add at most 16384 rows.
    let lines = jig
        .public_board_live
        .checked_add(jig.public_board_ended)
        .and_then(|goals| goals.checked_mul(jig.public_goal_tree.checked_add(1)?))
        .and_then(|lines| lines.checked_add(16_384))
        .expect("public schema rows fit");
    let nodes = lines.checked_mul(16).expect("public nodes fit");
    skein_ui::Limits {
        nodes,
        depth: 32,
        text_per_node: 4096,
        text_bytes: lines.checked_mul(1024).expect("public text fits"),
        drawing_bytes: 0,
        styles_per_node: 3,
        patches: nodes.checked_mul(5).expect("public patches fit"),
        bindings: lines,
        slots: 0,
        markdown_depth: 0,
        scene_marks: 0,
        scene_bytes: 0,
    }
}
#[cfg(test)]
#[path = "host_tests_public.rs"]
mod tests;
