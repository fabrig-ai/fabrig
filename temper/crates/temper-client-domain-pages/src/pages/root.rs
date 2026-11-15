//! The pages' access to the client root (temper/docs/design/client/domain-crates.md, 3.2).
use super::{Event, Pages, Request, accounts};
use crate::{Limits, forge};
use skein_lib::{Env, Queue, Wall};

/// What a page reads of the client root (temper/docs/design/client/domain-crates.md, 3.2).
pub trait Root {
    /// The pages' state.
    #[must_use]
    fn pages(&self) -> &Pages;
    /// The shared client.
    #[must_use]
    fn client(&self) -> &jig_client::Client;
    /// The forge's side.
    #[must_use]
    fn forge(&self) -> &forge::Side;
    /// The kept forges.
    #[must_use]
    fn forges(&self) -> &forge::forges::State;
    /// Minutes east of universal time.
    #[must_use]
    fn zone_minutes(&self) -> i32;
    /// Whether the project's Away row is on at the wall.
    #[must_use]
    fn away_on(&self, project: u32, wall: Wall) -> bool;
    /// Whether Away is shown for the project at the wall.
    #[must_use]
    fn away_shown(&self, project: u32, wall: Wall) -> bool;
}
/// What a page's act asks of the root's routes, at the point it asks
/// (temper/docs/design/client/domain-crates.md, 3.2).
pub trait Routing: Root {
    /// Borrow the pages for a change.
    fn pages_mut(&mut self) -> &mut Pages;
    /// Borrow the state a page's act changes together.
    fn parts(&mut self) -> Parts<'_>;
    /// The root routes one page request now, as it routes the requests of `pages::step`.
    fn request(&mut self, env: &Env<Limits>, request: Request);
    /// The root routes a page event now, as `pages::Event`s from the shell.
    fn event(&mut self, env: &Env<Limits>, event: Event);
    /// The root routes the forge's side's requests now.
    fn forge_requests(&mut self, env: &Env<Limits>, requests: &mut Queue<forge::Request>);
}
/// The root's state a page's act changes, borrowed together.
#[derive(Debug)]
pub struct Parts<'a> {
    /// The pages' state.
    pub pages: &'a mut Pages,
    /// The shared client.
    pub client: &'a jig_client::Client,
    /// The forge's side.
    pub forge: &'a mut forge::Side,
    /// The offered terminal's login watch.
    pub terminal: &'a mut accounts::terminal_login::Terminal,
}
