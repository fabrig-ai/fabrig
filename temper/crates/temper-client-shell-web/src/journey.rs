//! Record and replay the shipped host's seam (client/web/journeys.md, 2–4).
use crate::host::{self, Host};
use skein_lib::{Time, Wall};
use skein_ui::{Act, NodeId, Tree};
use skein_ui_web::{mapping::Primary, page::Page};
mod fields;
mod line;
mod originals;
mod record;
mod redact;
mod replay;
pub mod scan;
mod stream;
pub use fields::fields_mut;
pub use line::{Field, Line, Read as Fields, Unreadable, fields as read_fields, lines};
pub use originals::{SECRET_EVENT, kept_head, originals, typed};
pub use record::Recorder;
pub use redact::{Redacted, Unredacted, redact, redact_lines, saved, saved_lines};
pub use replay::prove_drawn;
pub use replay::{Mark, Refusal, Replay, Stale, prove, prove_lines};
pub use scan::{Found, Place, Shape, What, original, shape};
pub use stream::{Event, Plain, Streams, Undecoded};
pub use temper_client_redaction::{Framing, Kind, Original, Originals, placeholder, redact_field};
#[cfg(test)]
mod tests;

/// The start's facts, as the shell reads them (client/web/journeys.md, 3.2;
/// temper/docs/design/client/journey-clock.md, 4.2).
pub struct Facts {
    pub fixed: bool,
    pub seed: u64,
    pub now: Time,
    pub wall: Wall,
    pub primary: Primary,
    pub zone_minutes: i32,
    #[expect(clippy::type_complexity, reason = "the start keeps the meta content and its page origin together")]
    pub ways: Option<(Box<[u8]>, Box<[u8]>)>,
    pub forges: Vec<Box<[u8]>>,
    pub path: Box<[u8]>,
    pub secret: Option<Box<[u8]>>,
    pub stored: Vec<skein_ui_web::store::Record>,
}
impl Facts {
    /// Start exactly as the browser does (client/web/journeys.md, 4.1).
    #[must_use]
    pub fn start(&self) -> (Host, Page<'static>) {
        let ways = self.ways.as_ref().map_or(temper_client_view::sign_in::Ways::NONE, |(content, origin)| {
            temper_client_view::sign_in::Ways::parse(&String::from_utf8_lossy(content), origin)
        });
        let mut host = Host::new(crate::limits::client(), self.seed, self.now, self.wall, self.zone_minutes, ways);
        let mut forges = temper_client_view::forge_sign_in::Forges::new();
        for content in &self.forges {
            forges.read(&String::from_utf8_lossy(content));
        }
        host.forge_sign_in(forges);
        host.start(&self.path, self.stored.clone(), self.secret.clone());
        let page = host::page(&host.bounds.tree, self.primary);
        (host, page)
    }
}

/// The event the view makes of `act` on `node`, as a recording keeps it: the
/// domain `Event`'s Debug print (client/web/journeys.md, 3.3). None when it makes none.
#[must_use]
pub fn event(host: &Host, node: NodeId, act: &Act) -> Option<Box<[u8]>> {
    host.event_of(node, line::clone_act(act)).map(|event| format!("{event:?}").into_bytes().into())
}
/// Every node of the host's tree, in tree order, of which the view makes `event` from `act`.
#[must_use]
pub fn giving(host: &Host, act: &Act, event: &[u8]) -> Vec<NodeId> {
    host.tree()
        .nodes()
        .iter()
        .filter_map(|node| (self::event(host, node.id, act).as_deref() == Some(event)).then_some(node.id))
        .collect()
}
/// A node's accessible name, for a reader: its name, else its subtree's text.
fn name(tree: &Tree, node: NodeId) -> Box<[u8]> {
    let index =
        tree.nodes().iter().position(|candidate| candidate.id == node).expect("an event's node is in the host's tree");
    // Same fallback as skein-fake-person's tree.rs, exactly_named.
    let nodes = tree.nodes();
    let node = &nodes[index];
    if let Some(name) = &node.name {
        return name.clone();
    }
    let end = index + usize::try_from(node.size).expect("subtree fits usize");
    nodes[index..end]
        .iter()
        .flat_map(|node| node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()))
        .collect()
}

/// How a mark's controls lead on (client/web/journeys.md, 5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lead {
    Press,
    Submit,
    Toggle,
}
