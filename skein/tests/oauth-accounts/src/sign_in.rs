//! Actual accounts sign-in with an independent issuer and scripted browser.
//! The shared world owns schedules, replay and process heaps (oauth/accounts.md, 8).
use crate::world::Client;
pub use crate::world::Process;
use skein_fake_oauth as fake;
use skein_fake_peers::{Transport, oauth as peer};
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Env, List, Time, Wall, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts as accounts;
use skein_world::domain::Ledger;
use skein_world::{Host, Memory, Outcome, Referee, World};
use std::net::Ipv4Addr;
mod browser;
#[cfg(test)]
mod cuts;
mod expected;
mod issuer;
mod owner;
mod paste;
pub(crate) mod protocol;
pub use browser::Browser;
use issuer::issuer;
pub use issuer::key_set;
pub use protocol::protocol;
const ROOM: u32 = 64;
fn io_limits() -> io::Limits {
    super::world::limits().io
}

/// The outside owner's actions during one sign-in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Story {
    /// Cancel a visited fixed-port sign-in and start its replacement in the same step.
    Restart,
    /// The listener's per-sign-in return setting.
    Return(Return),
    /// Two sign-ins fetch fresh issuer documents before their visits.
    OpenIdAgain,
    /// An issued client id completes a new registration.
    IssuedClientId,
    /// The returned subject differs from the owner's expectation.
    AnotherSubject,
    /// Sign-in keeps declined scopes, then its grant is refused.
    RequiredScopes,
    /// The person pastes the public client's redirect.
    Paste,
    /// An accepted browser connection sends its redirect after the paste wins.
    PasteThenListener,
    /// The browser receives its callback page before the person pastes.
    ListenerThenPaste,
    /// A second identical paste is refused.
    PasteTwice,
    /// Malformed and mismatched pastes leave the sign-in waiting.
    PasteWrong,
    /// Exhausted paste state refusals leave the listener available.
    StateRefusals,
    /// A listener request with another state leaves the sign-in waiting.
    ListenerWrongState,
    /// A paste after the deadline is refused.
    PasteExpired,
    /// A paste after cancellation is refused.
    PasteCancelled,
    /// A paste completes a public sign-in through the owner's close.
    PasteClose,
    /// Public sign-in followed by a rotating refresh.
    Public,
    /// `OpenID` profile sign-in and a refresh whose answer optionally carries claims.
    Profile(bool),
    /// The preferred loopback port is held by another listener.
    VaryingTaken,
    /// Two sign-ins wait together on their own ephemeral ports.
    TwoPorts,
    /// The owner's web server hands in a confidential redirect.
    Confidential,
    /// A wrong path is refused before the real callback.
    WrongPath,
    /// Another authority is refused before the real callback.
    WrongHost,
    /// An oversized request head is refused before the real callback.
    LongHead,
    /// Close keeps waiting for the person's redirect, then drains.
    CloseWaiting,
    /// A confidential redirect completes the admitted sign-in through a close.
    CloseConfidential,
    /// Owner cancel ends its sign-in and listener.
    Cancel,
    /// Abort after close cancels the sign-in.
    Abort,
    /// A keeper failure is the sign-in's failure terminal.
    NotKept,
    /// The shipped person deadline ends a sign-in that receives no redirect.
    Timeout,
}
/// The owner's return settings at the sign-in boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Return {
    /// Return admitted while the sign-in loads.
    Back,
    /// Return before any sign-in is dropped.
    Before,
    /// Return after the terminal is dropped before a second sign-in.
    After,
    /// A later setting replaces the first.
    Replace,
    /// Oversized setting is dropped.
    Long,
    /// A space makes a setting invalid.
    Space,
    /// A missing scheme makes a setting invalid.
    Scheme,
    /// An empty setting is dropped.
    Empty,
    /// A control byte makes a setting invalid.
    Control,
    /// A non-ASCII byte makes a setting invalid.
    NonAscii,
    /// Exactly the byte bound is admitted.
    Bound,
    /// A refused path gets no return.
    WrongPath,
}
pub(super) fn again(story: Story) -> bool {
    matches!(story, Story::OpenIdAgain | Story::Return(Return::After))
}
pub(super) fn return_location(story: Story) -> Option<Box<[u8]>> {
    match story {
        Story::Return(Return::Back | Return::WrongPath) => Some(bytes::copy_of(b"http://127.0.0.1:32000/back")),
        Story::Return(Return::Replace) => Some(bytes::copy_of(b"https://application.example/replaced")),
        Story::Return(Return::Bound) => {
            let mut location = b"https://application.example/".to_vec();
            location.resize(accounts::RETURN_BYTES, b'x');
            Some(location.into_boxed_slice())
        }
        Story::Paste
        | Story::PasteThenListener
        | Story::ListenerThenPaste
        | Story::PasteTwice
        | Story::PasteWrong
        | Story::StateRefusals
        | Story::ListenerWrongState
        | Story::PasteExpired
        | Story::PasteCancelled
        | Story::PasteClose
        | Story::Return(
            Return::Before
            | Return::After
            | Return::Long
            | Return::Space
            | Return::Scheme
            | Return::Empty
            | Return::Control
            | Return::NonAscii,
        )
        | Story::OpenIdAgain
        | Story::IssuedClientId
        | Story::AnotherSubject
        | Story::RequiredScopes
        | Story::Public
        | Story::Profile(_)
        | Story::Restart
        | Story::VaryingTaken
        | Story::TwoPorts
        | Story::Confidential
        | Story::WrongPath
        | Story::WrongHost
        | Story::LongHead
        | Story::CloseWaiting
        | Story::CloseConfidential
        | Story::Cancel
        | Story::Abort
        | Story::NotKept
        | Story::Timeout => None,
    }
}
fn confidential(story: Story) -> bool {
    matches!(story, Story::Confidential | Story::CloseConfidential)
}
/// The sign-in story's registration, shared by component heap fixtures.
#[must_use]
pub fn registration(story: Story) -> oauth::Registration {
    let mut registration = super::world::registration(Transport::Plaintext);
    if paste::story(story) || story == Story::ListenerWrongState {
        registration.redirect_uri = bytes::copy_of(b"http://127.0.0.1:31234/callback");
    }
    if confidential(story) {
        registration.client_secret = Some(skein_lib::Secret::new(bytes::copy_of(b"fake-secret")));
        registration.redirect_uri = bytes::copy_of(b"https://service.example/callback");
    }
    if matches!(story, Story::VaryingTaken | Story::TwoPorts) {
        registration.redirect_uri = bytes::copy_of(if story == Story::TwoPorts {
            b"http://127.0.0.1:0/callback"
        } else {
            b"http://127.0.0.1:31234/callback"
        });
        registration.terms = Some(oauth::Terms { varying_port: true, ..crate::world::terms() });
    }
    if story == Story::RequiredScopes {
        registration.scope = bytes::copy_of(REQUIRED_SCOPES);
        registration.terms =
            Some(oauth::Terms { required_scopes: Some(bytes::copy_of(REQUIRED_SCOPES)), ..crate::world::terms() });
    }
    if matches!(
        story,
        Story::Profile(_)
            | Story::OpenIdAgain
            | Story::Return(Return::After)
            | Story::IssuedClientId
            | Story::AnotherSubject
    ) {
        registration.scope = bytes::copy_of(b"openid");
        registration.terms = Some(oauth::Terms {
            issued_client_id: story == Story::IssuedClientId,
            openid: Some(oauth::OpenId {
                profile: if matches!(story, Story::Profile(_)) {
                    oauth::Profile { email: true, name: true }
                } else {
                    oauth::Profile::default()
                },
                issuer: bytes::copy_of(ISSUER),
                key_set: None,
            }),
            ..crate::world::terms()
        });
    }
    registration
}
/// Content-free outside facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fact {
    /// Authorization URL was ready to show.
    Visit,
    /// The owner's redirect was taken.
    Taken,
    /// A new record reached the durable keeper.
    Keep(u64),
    /// The kept sign-in completed.
    SignedIn(u64),
    /// Profile claims accompanied the sign-in terminal.
    Profile,
    /// Profile claims accompanied a durably kept refresh.
    Refreshed(u64),
    /// Required scopes declined by the issuer.
    Missing(Box<[u8]>),
    /// The owner heard a usable grant.
    Granted(u64),
    /// An admitted request failed.
    Failed(accounts::Failure),
    /// The owner heard a redirect refusal.
    Refused(accounts::Refusal),
    /// The component settled all children.
    Closed,
}
/// The sign-in terminal referee uses the shared request ledger.
pub struct Contracts {
    sign_in: Ledger<u32, ()>,
    keeps: Ledger<u64, ()>,
    kept: Vec<u64>,
    closed: bool,
}
impl Contracts {
    /// Starts with one admitted sign-in.
    #[must_use]
    pub fn new() -> Self {
        let mut sign_in = Ledger::new("sign-in and keeper terminals");
        sign_in.open(0, ());
        Self { sign_in, keeps: Ledger::new("sign-in and keeper terminals"), kept: Vec::new(), closed: false }
    }
    /// Records a candidate that has no keeper answer yet.
    pub fn keep(&mut self, generation: u64) {
        self.keeps.open(generation, ());
    }
    /// Records the owner's durable answer.
    pub fn kept(&mut self, generation: u64, yes: bool) {
        self.keeps.end(generation);
        if yes {
            self.kept.push(generation);
        }
    }
    /// Success must follow keeping and end the admitted sign-in once.
    pub fn signed_in(&mut self, generation: u64) {
        assert!(self.kept.contains(&generation), "signed in before kept");
        self.sign_in.end(0);
    }
    /// A failed sign-in has the same one-terminal rule.
    pub fn failed(&mut self) {
        self.sign_in.end(0);
    }
    /// Lending requires durable keeping.
    pub fn granted(&mut self, generation: u64) {
        assert!(self.kept.contains(&generation), "grant before kept");
    }
    /// Closed follows all request and keeper terminals once.
    pub fn closed(&mut self) {
        assert!(!self.closed);
        self.sign_in.assert_settled();
        self.keeps.assert_settled();
        self.closed = true;
    }
}
impl Default for Contracts {
    fn default() -> Self {
        Self::new()
    }
}
/// Sign-in decisions and observations on the shared owner harness.
pub(crate) struct Owner {
    opened_private: Option<Time>,
    stored_private: Option<Time>,
    story: Story,
    facts: Vec<Fact>,
    records: Vec<oauth::SavedToken>,
    contracts: Vec<Contracts>,
    visits: Vec<Box<[u8]>>,
    large: bool,
    redirected: [u8; 1024],
    redirected_len: usize,
    again_at: Option<Time>,
    callback: paste::Callback,
    pending_keep: Option<(u32, u64)>,
    expired_paste: Option<Box<[u8]>>,
}

fn client(story: Story, large: bool) -> Client {
    client_with_limits(story, large, &crate::world::limits(), ROOM)
}
fn client_with_limits(story: Story, large: bool, limits: &accounts::Limits, room: u32) -> Client {
    let mut tuned = *limits;
    if matches!(story, Story::Return(_)) {
        tuned.server.response = tuned.server.response.max(4096);
    }
    if matches!(story, Story::TwoPorts | Story::Restart) {
        tuned.listeners = 2;
        tuned.io.sockets = 6;
    }
    let mut client = Client::with_limits(crate::world::Story::Rotate, Transport::Plaintext, &tuned, room);
    client.sign_in = Some(Owner {
        opened_private: None,
        stored_private: None,
        story,
        facts: Vec::with_capacity(16),
        records: Vec::with_capacity(2),
        contracts: (0..if story == Story::TwoPorts { 2 } else { 1 }).map(|_| Contracts::new()).collect(),
        visits: Vec::with_capacity(2),
        large,
        redirected: [0; 1024],
        redirected_len: 0,
        again_at: None,
        callback: paste::Callback::Waiting,
        pending_keep: None,
        expired_paste: None,
    });
    client
}

fn owner(client: &Client) -> &Owner {
    client.sign_in.as_ref().expect("sign-in story")
}
fn owner_mut(client: &mut Client) -> &mut Owner {
    client.sign_in.as_mut().expect("sign-in story")
}
pub(crate) use owner::{observe, restart, start};

pub(crate) fn accepted(client: &mut Client, event: &io::Event) {
    if matches!(event, io::Event::Accepted { .. }) {
        owner_mut(client).callback = paste::Callback::Accepted;
    }
}
pub(crate) fn redirected(client: &mut Client, env: &Env<accounts::Limits>) {
    if owner(client).redirected_len > 0 {
        let state = owner_mut(client);
        let uri = bytes::copy_of(&state.redirected[..state.redirected_len]);
        state.redirected_len = 0;
        paste::paste(client, env, uri);
    }
}
pub(crate) fn work_pending(owner: &Owner) -> bool {
    owner.redirected_len > 0 || (owner.pending_keep.is_some() && owner.callback == paste::Callback::Finished)
}
pub(crate) fn next_deadline(owner: &Owner) -> Option<Time> {
    owner.again_at
}
pub(crate) fn file_event(client: &mut Client, now: Time, event: &io::file::Event) {
    if matches!(event, io::file::Event::Opened { .. }) {
        owner_mut(client).opened_private = Some(now);
    }
    if matches!(event, io::file::Event::Stored { .. }) {
        owner_mut(client).stored_private = Some(now);
    }
}
pub(crate) fn keeping(owner: &mut Owner, generation: u64) {
    owner.contracts[0].keep(generation);
    owner.facts.push(Fact::Keep(generation));
}
pub(crate) fn kept(owner: &mut Owner, record: oauth::SavedToken, yes: bool) {
    owner.contracts[0].kept(record.generation, yes);
    owner.records.push(record);
}

#[expect(clippy::struct_excessive_bools, reason = "independent outside delivery, shutdown and success observations")]
struct Judge {
    delivered: usize,
    redirected: bool,
    shutdown: bool,
    wake: Option<Time>,
    passed: bool,
    story: Story,
    private_failure: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, processes: &mut [Process]) {
        let address = processes.iter().find_map(|process| match process {
            Process::Issuer(peer) => peer.address(),
            Process::Client(_) | Process::Browser(_) => None,
        });
        let holder_ready =
            processes.iter().any(|process| matches!(process, Process::Browser(browser) if browser.holder.is_some()));
        let browser_done = processes.iter().any(|process| matches!(process, Process::Browser(browser) if browser.done));
        let (paste_ready, callback_permitted) = processes
            .iter()
            .find_map(|process| match process {
                Process::Client(client) => Some((
                    paste::delivery_ready(self.story, owner(client).callback, browser_done, &owner(client).facts),
                    paste::callback_permitted(self.story, &owner(client).facts),
                )),
                Process::Browser(_) | Process::Issuer(_) => None,
            })
            .expect("owner");
        let mut visit = [0; 1024];
        let mut visit_len = 0;
        let mut redirect = [0; 1024];
        let mut redirect_len = 0;
        for process in processes.iter() {
            match process {
                Process::Client(client) => {
                    if let Some(url) = owner(client).visits.get(self.delivered) {
                        visit_len = url.len();
                        visit[..visit_len].copy_from_slice(url);
                    }
                }
                Process::Browser(browser) => {
                    if let Some(uri) = &browser.redirect {
                        redirect_len = uri.len();
                        redirect[..redirect_len].copy_from_slice(uri);
                    }
                }
                Process::Issuer(_) => {}
            }
        }
        let settled = processes.iter().filter(|process| !matches!(process, Process::Issuer(_))).all(Host::is_empty);
        for process in processes {
            match process {
                Process::Client(client) => {
                    if self.story != Story::VaryingTaken || holder_ready {
                        client.address = address;
                    }
                    if browser_done {
                        owner_mut(client).callback = paste::Callback::Finished;
                    }
                    if (confidential(self.story) || paste::story(self.story))
                        && paste_ready
                        && !self.redirected
                        && redirect_len > 0
                    {
                        owner_mut(client).redirected[..redirect_len].copy_from_slice(&redirect[..redirect_len]);
                        owner_mut(client).redirected_len = redirect_len;
                        self.redirected = true;
                    }
                }
                Process::Browser(browser) => {
                    browser.callback_permitted = callback_permitted;
                    if !matches!(self.story, Story::Cancel | Story::Abort | Story::Timeout) && visit_len > 0 {
                        browser.inbox[..visit_len].copy_from_slice(&visit[..visit_len]);
                        browser.inbox_len = visit_len;
                        self.delivered += 1;
                    }
                }
                Process::Issuer(peer) => {
                    if settled && !self.shutdown {
                        peer.shutdown();
                        self.shutdown = true;
                    }
                }
            }
        }
    }
    fn watch(&mut self, now: Time, processes: &[Process]) -> Result<(), String> {
        let client = processes.iter().find_map(Process::client).expect("owner");
        let browser = processes.iter().find_map(Process::browser).expect("browser");
        let issuer = processes.iter().find_map(Process::issuer).expect("issuer");
        self.wake = ((client.component.is_none()
            && issuer.address().is_some()
            && (self.story != Story::VaryingTaken || browser.holder.is_some()))
            || (owner(client).visits.len() > self.delivered
                && !matches!(self.story, Story::Cancel | Story::Abort | Story::Timeout))
            || ((confidential(self.story) || paste::story(self.story))
                && paste::delivery_ready(self.story, owner(client).callback, browser.done, &owner(client).facts)
                && !self.redirected
                && browser.redirect.is_some())
            || browser.callback_permitted != paste::callback_permitted(self.story, &owner(client).facts)
            || (browser.done && owner(client).pending_keep.is_some())
            || (!self.shutdown && client.is_empty() && browser.is_empty()))
        .then_some(now);
        if client.closed {
            self.client_closed(client, issuer)?;
        }
        let posts = issuer
            .observations()
            .iter()
            .filter(|observation| matches!(observation, peer::Observation::Post { .. }))
            .count();
        if posts
            > if matches!(
                self.story,
                Story::Public | Story::Profile(_) | Story::TwoPorts | Story::OpenIdAgain | Story::Return(Return::After)
            ) {
                2
            } else {
                1
            }
        {
            return Err(String::from(
                "assertion failed: posts <= if matches!( self.story, Story::Public | Story::Profile(_) | Story::TwoPorts | Story::OpenIdAgain | Story::Return(Return::After) ) { 2 } else { 1 }",
            ));
        }
        if !self.private_failure
            && client.closed
            && browser.is_empty()
            && !matches!(self.story, Story::Cancel | Story::Abort | Story::Timeout)
        {
            if !browser.done {
                return Err(String::from("assertion failed: browser.done"));
            }
            if posts
                != if matches!(
                    self.story,
                    Story::Public
                        | Story::Profile(_)
                        | Story::TwoPorts
                        | Story::OpenIdAgain
                        | Story::Return(Return::After)
                ) {
                    2
                } else {
                    usize::from(!matches!(self.story, Story::PasteExpired | Story::PasteCancelled))
                }
            {
                return Err(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    posts,
                    if matches!(
                        self.story,
                        Story::Public
                            | Story::Profile(_)
                            | Story::TwoPorts
                            | Story::OpenIdAgain
                            | Story::Return(Return::After)
                    ) {
                        2
                    } else {
                        usize::from(!matches!(self.story, Story::PasteExpired | Story::PasteCancelled))
                    }
                ));
            }
        }
        self.passed = self.shutdown && processes.iter().all(Host::is_empty);
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake.or(Some(Time::from_nanos(180_000_000_000)))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now >= Time::from_nanos(180_000_000_000) && !self.passed).then(|| "accounts sign-in did not settle".to_owned())
    }
    fn passed(&self) -> bool {
        self.passed
    }
}
/// Runs sign-in with the actual component and independent socket processes.
#[must_use]
pub fn run(seed: u64, story: Story, faulted: bool, large: bool, memory: Memory) -> Outcome<Process> {
    let mut config = skein_sim::Config::calm();
    if faulted {
        config.buffer = 127;
        config.faults.short_recv = 300;
        config.faults.short_send = 300;
    }
    let mut world = World::new(
        seed,
        config,
        Judge {
            delivered: usize::from(story == Story::Restart),
            redirected: false,
            shutdown: false,
            wake: None,
            passed: false,
            story,
            private_failure: false,
        },
        memory,
    );
    world.spawn(|| issuer(story, large));
    world.spawn(|| Process::Client(Box::new(client(story, large))));
    world.spawn(|| Process::Browser(Box::new(Browser::new(story))));
    world.run()
}
/// Reads owner facts from a simulated outcome.
#[must_use]
pub fn facts(processes: &[Process]) -> &[Fact] {
    processes
        .iter()
        .find_map(|process| match process {
            Process::Client(client) => Some(owner(client).facts.as_slice()),
            Process::Issuer(_) | Process::Browser(_) => None,
        })
        .expect("client")
}
/// Compares the actual wire trace, owner terminals and issuer observations.
pub fn assert_replay(first: &Outcome<Process>, second: &Outcome<Process>) {
    assert_eq!(first.trace, second.trace);
    for (first, second) in first.procs.iter().zip(&second.procs) {
        match (first, second) {
            (Process::Client(first), Process::Client(second)) => {
                assert_eq!(owner(first).facts, owner(second).facts);
                assert_eq!(owner(first).records, owner(second).records);
                assert_eq!(owner(first).visits, owner(second).visits);
            }
            (Process::Issuer(first), Process::Issuer(second)) => assert!(first.observations() == second.observations()),
            (Process::Browser(first), Process::Browser(second)) => {
                assert_eq!(first.done, second.done);
                assert_eq!(first.redirect, second.redirect);
            }
            _ => panic!("process order"),
        }
    }
}

/// Signs in with the private keeper on the actual minimal machine and independent issuer/browser.
#[must_use]
pub fn run_private(seed: u64, private: crate::private::Story) -> Outcome<Process, crate::private::FileSystem> {
    private_story(seed, private, Story::Confidential)
}

/// A public sign-in's return is set before its private load finishes.
#[must_use]
pub fn run_private_return(seed: u64) -> Outcome<Process, crate::private::FileSystem> {
    private_story(seed, crate::private::Story::Missing, Story::Return(Return::Back))
}

fn private_story(
    seed: u64,
    private: crate::private::Story,
    story: Story,
) -> Outcome<Process, crate::private::FileSystem> {
    let mut files = crate::private::FileSystem::new(private);
    let root = files.startup();
    let private_failure = matches!(
        private,
        crate::private::Story::RootMode
            | crate::private::Story::FileMode
            | crate::private::Story::FileOwner
            | crate::private::Story::FileLinks
            | crate::private::Story::FileLink
            | crate::private::Story::LoadFailed
            | crate::private::Story::LoadStalled
            | crate::private::Story::TooLarge
    );
    let mut config = skein_sim::Config::calm();
    config.wall = Wall::EPOCH;
    let mut world = World::new(
        seed,
        config,
        Judge { delivered: 0, redirected: false, shutdown: false, wake: None, passed: false, story, private_failure },
        Memory::Checked,
    )
    .with_machine(files);
    world.spawn(|| issuer(story, false));
    world.spawn_root(root, |fd| {
        let mut client = client(story, false);
        client.file_driver = Some(skein_world::files::Driver::new(
            fd,
            4,
            client.limits.client.document.record_bytes,
            client.limits.file_stall,
            1234,
        ));
        Process::Client(Box::new(client))
    });
    world.spawn(|| Process::Browser(Box::new(Browser::new(story))));
    let mut stalled = false;
    let mut outcome = world.run_with_faults(|_, _, submissions| {
        let stall = private == crate::private::Story::LoadStalled
            && !stalled
            && submissions.iter().any(|submit| matches!(submit.kind, kernel::Op::Read { .. }));
        stalled |= stall;
        Some(skein_sim::Faults { hung: if stall { 1000 } else { 0 }, ..skein_sim::Faults::NONE })
    });
    outcome.machine.faulted |= stalled;
    outcome
}

const REQUIRED_SCOPES: &[u8] = b"read write";

pub(super) const ISSUER: &[u8] = b"http://127.0.0.1:31000";
pub(super) const KEYS: &[u8] = b"http://127.0.0.1:31000/keys";
pub(super) const SUBJECT: &[u8] = b"account-subject";
pub(super) const EXPECTED_SUBJECT: &[u8] = b"expected-subject";
pub(super) const ISSUED_CLIENT: &[u8] = b"issued-client";
