//! Record and replay the shipped host's seam (client/web/journeys.md, 2–4).
use crate::host::{self, Host};
use skein_lib::{Time, Wall};
use skein_ui::{Act, NodeId, Role, Tree};
use skein_ui_web::{mapping::Primary, page::Page};
mod fields;
mod line;
mod originals;
mod record;
mod redact;
mod replay;
pub mod scan;
mod setup;
mod stream;
mod walk;
pub use fields::fields_mut;
pub use line::{Line, Unreadable, lines};
pub use originals::{SECRET_EVENT, kept_head, originals, typed};
pub use record::Recorder;
pub use redact::{Redacted, Unredacted, redact};
pub use replay::prove_drawn;
pub use replay::{Mark, Refusal, Replay, Stale, prove};
pub use scan::{Found, Place, Shape, What, original, shape};
pub use setup::{MarkWait, Setup, SetupStep, setup};
pub use stream::{Event, Plain, Streams, Undecoded};
pub use temper_client_redaction::{Framing, Kind, Original, Originals, placeholder, redact_field};
pub use walk::{Step, Unwalkable, differs, still, walk};
#[cfg(test)]
mod tests;

/// The start's facts, as the shell reads them (client/web/journeys.md, 3.2).
pub struct Facts {
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
        let mut host = Host::new(self.seed, self.now, self.wall, self.zone_minutes, ways);
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

/// A control's role, accessible name and order among exact matches, from zero
/// (client/web/journeys.md, 8.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Control {
    pub role: Role,
    pub name: Box<[u8]>,
    pub order: u32,
}
impl Control {
    #[must_use]
    pub fn of(tree: &Tree, node: NodeId) -> Option<Self> {
        let index = tree.nodes().iter().position(|candidate| candidate.id == node)?;
        let role = tree.nodes()[index].role;
        let name = self::name(tree, node);
        let order = tree.nodes()[..index]
            .iter()
            .filter(|candidate| candidate.role == role && self::name(tree, candidate.id) == name)
            .count();
        Some(Self { role, name, order: u32::try_from(order).expect("tree count fits u32") })
    }
}
/// Replay a save to its end, then find the first control giving the act's event
/// outside a menu, which no journey opens (client/web/journeys.md, 8.1).
/// # Errors
/// Returns the save's unreadable or stale refusal.
pub fn control(save: &[u8], act: &Act, event: &[u8]) -> Result<Option<Control>, Refusal> {
    let mut replay = Replay::new(save)?;
    while replay.next().map_err(Refusal::Stale)?.is_some() {}
    let host = replay.host();
    let tree = host.tree();
    Ok(giving(host, act, event)
        .into_iter()
        .find(|node| {
            let index =
                tree.nodes().iter().position(|candidate| candidate.id == *node).expect("event's node is in tree");
            !tree.nodes()[..index].iter().enumerate().any(|(at, ancestor)| {
                ancestor.role == Role::Menu && index - at < usize::try_from(ancestor.size).expect("subtree fits usize")
            })
        })
        .and_then(|node| Control::of(tree, node)))
}
