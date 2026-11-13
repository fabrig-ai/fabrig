//! Temper's pages and forge share one durable jig client.
use crate::{Event, Limits, Request, forge, route};
use skein_lib::{Env, Queue, Rng, Time};
#[derive(Debug)]
pub struct Domain {
    zone_minutes: i32,
    pub(crate) copy: crate::copy::Copy,
    pub(crate) pages: crate::pages::Pages,
    pub(crate) forge: forge::Side,
    pub(crate) client: jig_client::Client,
    pub(crate) unlinked: bool,
    pub(crate) tab: Option<jig_client::Key>,
    pub(crate) tab_revision: u64,
    pub(crate) authorization: Option<crate::authorize::Authorization>,
    pub(crate) authorizing: crate::authorize::Flight,
    pub(crate) authorization_revision: u64,
    pub(crate) approval: Option<crate::approve::Page>,
    pub(crate) approving: crate::approve::Flight,
    pub(crate) approval_revision: u64,
    pub(crate) asked: Option<crate::asked::State>,
    pub(crate) signed_out: bool,
    pub(crate) terminal: crate::pages::accounts::terminal_login::Terminal,
}
#[must_use]
pub fn new(limits: &Limits, seed: u64) -> Domain {
    build(limits, seed, false)
}
#[must_use]
pub fn unlinked(limits: &Limits, seed: u64) -> Domain {
    build(limits, seed, true)
}
fn build(limits: &Limits, seed: u64, unlinked: bool) -> Domain {
    assert!(crate::valid(limits), "root capacities are checked");
    let mut rng = Rng::new(seed);
    let client = if unlinked {
        jig_client::Client::unlinked(&limits.client, rng.next_u64())
    } else {
        jig_client::Client::new(&limits.client, rng.next_u64())
    }
    .expect("checked client limits");
    Domain {
        zone_minutes: 0,
        copy: crate::copy::Copy::new(),
        pages: crate::pages::Pages::seeded(rng.next_u64()),
        forge: forge::Side::new(&limits.forge, rng.next_u64()),
        client,
        unlinked,
        tab: None,
        tab_revision: 0,
        authorization: None,
        authorizing: crate::authorize::Flight::Idle,
        authorization_revision: 0,
        approval: None,
        approving: crate::approve::Flight::Idle,
        approval_revision: 0,
        asked: None,
        signed_out: false,
        terminal: crate::pages::accounts::terminal_login::Terminal::new(),
    }
}
impl Domain {
    /// The offered terminal's current login (client/tui/box.md, Lending its terminal).
    #[must_use]
    pub fn terminal_login(&self) -> Option<&jig_client::contract::TerminalLogin> {
        self.terminal.login(&self.client)
    }
    /// The approval page (client/pages/README.md, The approval page).
    #[must_use]
    pub const fn approval(&self) -> Option<&crate::approve::Page> {
        self.approval.as_ref()
    }
    /// A native client's ask (client/README.md, 4.1).
    #[must_use]
    pub const fn asked(&self) -> Option<crate::asked::State> {
        self.asked
    }
    #[must_use]
    pub const fn copy_label(&self) -> crate::copy::Label {
        self.copy.label()
    }
    /// Minutes east of universal time: the shell's input (temper/docs/design/client/view.md, 6).
    #[must_use]
    pub const fn zone_minutes(&self) -> i32 {
        self.zone_minutes
    }
    /// Set the shell's zone before its first event (temper/docs/design/client/web/README.md, 2.1).
    pub fn set_zone_minutes(&mut self, zone_minutes: i32) {
        self.zone_minutes = zone_minutes;
    }

    /// The row at the wall (temper/docs/design/client/pages/README.md, Away).
    #[must_use]
    pub fn away_on(&self, project: u32, wall: skein_lib::Wall) -> bool {
        match self.forge.away(project) {
            Some(row) => row.on_at(wall),
            None => false,
        }
    }
    /// Classes with an item Away may decide, or an on row, show Away (temper/docs/design/client/pages/README.md).
    #[must_use]
    pub fn away_shown(&self, project: u32, wall: skein_lib::Wall) -> bool {
        self.away_on(project, wall)
            || match self.forge.classes(project) {
                Some(classes) => classes.away_may_decide(),
                None => false,
            }
    }
    /// The open project's live or ended goals (temper/docs/design/client/view.md, 3).
    #[must_use]
    pub fn has_goals(&self, project: u32) -> bool {
        self.pages.has_goals(&self.client, project)
    }
    #[must_use]
    pub const fn authorization(&self) -> Option<&crate::authorize::Authorization> {
        self.authorization.as_ref()
    }
    #[must_use]
    pub const fn signed_out(&self) -> bool {
        self.signed_out
    }
    #[must_use]
    pub const fn pages(&self) -> &crate::pages::Pages {
        &self.pages
    }
    #[must_use]
    pub const fn client(&self) -> &jig_client::Client {
        &self.client
    }
    #[must_use]
    pub const fn forges(&self) -> &forge::forges::State {
        self.forge.forges()
    }
    #[must_use]
    pub const fn forge(&self) -> &forge::Side {
        &self.forge
    }
    #[must_use]
    pub const fn forge_number(&self) -> u16 {
        temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge)
    }
}
pub fn step(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    route::event(root, env, event, out);
    route::welcome_progress(root, env, out);
    route::welcome_model_progress(root, out);
    route::playground_progress(root, env, out);
    route::ways_progress(root, env, out);
    crate::authorize::progress(root, env, out);
    crate::approve::progress(root, env, out);
    crate::asked::progress(root);
    welcome(root, out);
}
pub fn fire(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    root.copy.fire(env.now);
    let mut client = Queue::uncapped();
    if root.unlinked {
        root.client.fire_unlinked(env.now, env.wall, &mut client);
        route::route_unlinked(root, env, &mut client, out);
    } else {
        let mut client = Queue::uncapped();
        root.client.fire(env.now, env.wall, &mut client);
        route::route_client(root, env, &mut client, out);
    }
    route::fire_pages(root, env, out);
    route::welcome_progress(root, env, out);
    route::welcome_model_progress(root, out);
    route::playground_progress(root, env, out);
    route::ways_progress(root, env, out);
    crate::authorize::progress(root, env, out);
    crate::approve::progress(root, env, out);
    crate::asked::progress(root);
    welcome(root, out);
}
#[must_use]
pub fn next_deadline(root: &Domain) -> Option<Time> {
    [
        root.copy.next_deadline(),
        crate::approve::next_deadline(root),
        crate::pages::next_deadline(&root.pages),
        forge::next_deadline(&root.forge),
        root.client.next_deadline(),
    ]
    .into_iter()
    .flatten()
    .min()
}
/// Reclaim only after every child's tells have been routed.
pub fn take(root: &mut Domain, _env: &Env<Limits>, _out: &mut Queue<Request>) {
    root.client.reclaim();
    forge::reclaim(&mut root.forge);
}
#[must_use]
pub fn shown(root: &Domain) -> u64 {
    let count = crate::pages::shown(&root.pages);
    count
        .checked_add(forge::shown(&root.forge))
        .expect("forge shown count fits")
        .checked_add(root.copy.shown)
        .expect("copy shown count fits")
        .checked_add(root.authorization_revision)
        .expect("authorization revision fits")
        .checked_add(root.approval_revision)
        .expect("approval revision fits")
        .checked_add(root.tab_revision)
        .expect("tab revision fits")
}

fn welcome(root: &mut Domain, out: &mut Queue<Request>) {
    if root.authorization.is_some() || root.approval.is_some() {
        return;
    }
    if let crate::Address::Welcome { .. } = root.pages.address()
        && let Some(deployment) = root.forge.deployment()
        && !deployment.welcome
    {
        out.push(Request::Show {
            address: crate::Address::Inbox { filter: crate::address::Filter::ALL },
            replace: true,
        });
        return;
    }
    let crate::Address::Welcome { step: crate::Step::Welcome | crate::Step::WelcomeAsked } = root.pages.address()
    else {
        return;
    };
    let Some(page) = root.pages.welcome() else { return };
    if page.decided {
        return;
    }
    if let Some(state) = crate::pages::welcome::decision(root) {
        root.pages.decide_welcome();
        let address = match state {
            crate::pages::welcome::FirstRun::Empty => return,
            crate::pages::welcome::FirstRun::Unfinished { step } => {
                if *root.pages.address() == (crate::Address::Welcome { step: crate::Step::WelcomeAsked }) {
                    return;
                }
                crate::Address::Welcome { step }
            }
            crate::pages::welcome::FirstRun::Whole => crate::Address::Chats { filter: crate::ChatsFilter::All },
        };
        out.push(Request::Show { address, replace: true });
    }
}
