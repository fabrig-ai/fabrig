//! Record and replay the shipped host's seam (client/journeys.md, 2–4).
use crate::host::{self, Host};
use skein_fake_person::role_name;
use skein_lib::{Time, Wall};
use skein_ui::{NodeId, Role, Tree};
use skein_ui_web::{mapping::Primary, page::Page};
mod line;
mod record;
mod replay;
pub use line::{Line, Unreadable, lines};
pub use record::Recorder;
pub use replay::{Mark, Refusal, Replay, Stale, prove};
#[cfg(test)]
mod tests;

/// The start's facts, as the shell reads them (client/journeys.md, 3.2).
pub struct Facts {
    pub seed: u64,
    pub now: Time,
    pub wall: Wall,
    pub primary: Primary,
    #[expect(clippy::type_complexity, reason = "the start keeps the meta content and its page origin together")]
    pub ways: Option<(Box<[u8]>, Box<[u8]>)>,
    pub forges: Vec<Box<[u8]>>,
    pub path: Box<[u8]>,
    pub secret: Option<Box<[u8]>>,
    pub stored: Vec<skein_ui_web::store::Record>,
}
impl Facts {
    /// Start exactly as the browser does (client/journeys.md, 4.1).
    #[must_use]
    pub fn start(&self) -> (Host, Page<'static>) {
        let ways = self.ways.as_ref().map_or(temper_client_view::sign_in::Ways::NONE, |(content, origin)| {
            temper_client_view::sign_in::Ways::parse(&String::from_utf8_lossy(content), origin)
        });
        let mut host = Host::new(self.seed, self.now, self.wall, ways);
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

/// An act's role, accessible name and order (client/journeys.md, 3.3).
#[derive(Clone, Debug)]
pub struct Control {
    pub role: Role,
    pub name: Box<[u8]>,
    pub order: u32,
}
impl PartialEq for Control {
    fn eq(&self, other: &Self) -> bool {
        role_name(self.role) == role_name(other.role) && self.name == other.name && self.order == other.order
    }
}
impl Eq for Control {}

impl Control {
    #[must_use]
    pub fn of(tree: &Tree, node: NodeId) -> Option<Self> {
        let index = tree.nodes().iter().position(|candidate| candidate.id == node)?;
        let role = tree.nodes()[index].role;
        let name = name(tree, index);
        let order = (0..=index)
            .filter(|&at| role_name(tree.nodes()[at].role) == role_name(role) && name == self::name(tree, at))
            .count();
        Some(Self { role, name, order: u32::try_from(order).expect("tree count fits u32") })
    }
    #[must_use]
    pub fn find(&self, tree: &Tree) -> Option<NodeId> {
        let index = usize::try_from(self.order.checked_sub(1)?).ok()?;
        tree.nodes()
            .iter()
            .enumerate()
            .filter(|(at, node)| role_name(node.role) == role_name(self.role) && name(tree, *at) == self.name)
            .nth(index)
            .map(|(_, node)| node.id)
    }
}
fn name(tree: &Tree, index: usize) -> Box<[u8]> {
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
