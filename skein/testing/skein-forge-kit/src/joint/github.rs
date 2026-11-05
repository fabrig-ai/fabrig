//! GitHub HTTP peers sharing the joint's domain and cutting pipes (skein/docs/design/fake-forge.md, 10.6).
use super::{Joint, StartCounts, Peer};
use crate::{pipe::{Cut, Pipe, RequestCounts}, sockets::Sockets};
use skein_fake_forge_forgejo::{Route, Tally};
use skein_fake_forge_github::{
    fixture,
    inline::{self, Lent},
    server::{self, Event, Server},
    service::{Limits, Service},
};
use skein_forge::github::request::Method;
use skein_http::server as http;
use skein_lib::{Env, Queue, Token, stream::Down};
use std::collections::VecDeque;

// More heads than a tick can receive through the fixture's bounded read.
const HEADS: u32 = 64;

pub(super) struct Github {
    service: Service,
    app: skein_fake_forge_github::app::App,
    verified: skein_fake_forge_github::app::Verified,
    peers: Sockets<Peer<Server, Event>>,
    pub tally: Tally,
}
impl Github {
    pub fn new(seed: u64) -> Self {
        Self {
            service: crate::github::Fake::new(seed).into_service(),
            app: skein_fake_forge_github::fixture::app(),
            verified: skein_fake_forge_github::app::Verified::empty(),
            peers: Sockets::default(),
            tally: Tally::new(1),
        }
    }
    pub fn busy(&self) -> bool {
        self.peers.values().any(|peer| {
            peer.pipe.has_work() || peer.server.has_work() || !peer.above.is_empty() || !peer.below.is_empty()
        })
    }
    pub fn owners(&self) -> impl Iterator<Item = Token> + '_ {
        self.peers.keys().copied()
    }
    fn env(world: &Joint) -> Env<Limits> {
        world.env.with(fixture::limits())
    }
    pub fn tick(&mut self, world: &mut Joint) {
        let env = Self::env(world);
        let owners: Vec<_> = self.owners().collect();
        for owner in owners {
            let peer = self.peers.get_mut(&owner).expect("live peer");
            while let Some(down) = peer.below.pop() {
                peer.pipe.server_down(down);
            }
            if let Some(up) = peer.pipe.server_up() {
                server::up(&mut peer.server, &mut self.service, &env, up, &mut peer.above, &mut peer.below);
            }
            for _ in 0..HEADS {
                inline::pass(
                    &mut peer.server,
                    &mut self.service,
                    &mut Lent { domain: &mut world.domain, env: &world.fake_env, hooks: false },
                    &env,
                    &mut peer.above,
                    &mut peer.below,
                );
                let Some(call) = peer.server.take_head() else { break };
                if peer.heads.pop_front().expect("head follows first byte") {
                    world.starts.served += 1;
                }
                let user =
                    peer.server.credential().and_then(|header| header.strip_prefix(b"Bearer ")).and_then(|bearer| {
                        (self.service.installations().token(bearer, env.wall).is_ok()
                            || self.verified.check(bearer, &self.app, env.wall).is_ok())
                        .then_some(9)
                    });
                self.tally.count(if user.is_some() { route(&call) } else { None }, user);
            }
            let up = peer.pipe.client_up();
            if let Some(up) = up {
                world.up(skein_io::Event::Stream { owner, up });
            }
        }
    }
    pub fn io(&mut self, world: &mut Joint, request: skein_io::Request) {
        let owner = match &request {
            skein_io::Request::Connect { owner, .. } | skein_io::Request::Stream { stream: owner, .. } => *owner,
            skein_io::Request::Abort { entity } | skein_io::Request::Close { entity } => { self.end(world, *entity); return; }
            _ => Token::new(0),
        };
        let before = self.peers.get(&owner).map_or(0, |peer| peer.pipe.request_counts().written);
        let mut events = Queue::with_capacity(3);
        let _ = self.peers.request(request, |_| {
                let cut =
                    if world.rng.chance(world.cut_chance) { Cut::At(world.rng.between(0, 2048)) } else { Cut::Never };
                let env = Self::env(world);
                let mut peer = Peer {
                    pipe: Pipe::new(world.rng.next_u64(), cut),
                    server: Server::new(owner, &fixture::limits()).expect("server"),
                    above: Queue::with_capacity(64),
                    below: Queue::with_capacity(64),
                    heads: VecDeque::new(),
                    line_indices: Vec::new(),
                };
                server::start(&mut peer.server, &mut self.service, &env, &mut peer.above, &mut peer.below);
            Some(peer)
        }, &mut events);
        if let Some(peer) = self.peers.get_mut(&owner) {
                    let written = peer.pipe.request_counts().written - before;
                    let startup = world.forges.iter().any(|forge| {
                        matches!(
                            world.component.start(*forge),
                            Some(
                                skein_forge_connection::start::Start::Listing { .. }
                                    | skein_forge_connection::start::Start::Minting { .. }
                                    | skein_forge_connection::start::Start::Identity { .. }
                                    | skein_forge_connection::start::Start::Waiting { .. }
                            )
                        )
                    });
                    peer.observe(written, startup, &mut world.starts, &mut world.lines);
        }
        while let Some(event) = events.pop() { world.up(event); }
    }
    pub fn end(&mut self, world: &mut Joint, owner: Token) {
        let mut events = Queue::with_capacity(1);
        if let Some(mut peer) = self.peers.close(owner, &mut events) {
            peer.freeze(&mut world.starts, &mut world.closed_counts);
            while let Some(event) = events.pop() { world.up(event); }
        }
    }
    pub fn start_counts(&self, counts: &mut StartCounts) {
        for peer in self.peers.values() {
            if peer.heads.front() == Some(&true) {
                counts.cut_in_head += peer.pipe.request_counts().cut_in_head;
            }
        }
    }
    pub fn request_counts(&self, counts: &mut RequestCounts) {
        for peer in self.peers.values() {
            let requests = peer.pipe.request_counts();
            counts.written += requests.written;
            counts.cut_in_head += requests.cut_in_head;
        }
    }
}

fn method(method: skein_http::Method) -> Method {
    match method {
        skein_http::Method::Get => Method::Get,
        skein_http::Method::Post => Method::Post,
        skein_http::Method::Put => Method::Put,
        skein_http::Method::Patch => Method::Patch,
        skein_http::Method::Delete => Method::Delete,
        skein_http::Method::Head | skein_http::Method::Options => panic!("API method"),
    }
}

// Tally's shared route names cover both dialects' repository operations.
// App, delivery-log and GraphQL entrances have no Forgejo route name.
fn route(call: &http::Call) -> Option<Route> {
    use skein_forge::github::request::Request as R;
    let documents = crate::fixtures::fake_limits().documents;
    let limits = skein_forge::github::limits::Limits {
        json: documents.json,
        writer: documents.writer,
        string_bytes: documents.string_bytes,
        body_bytes: documents.body_bytes,
        inner: documents.inner,
        target_bytes: documents.target_bytes,
    };
    Some(match skein_forge::github::server::route(method(call.method), &call.target, &limits)? {
        R::Reachable { .. } => Route::UserRepos,
        R::Viewer => Route::User,
        R::Merge { .. } => Route::Merge,
        R::BotUser { .. } | R::User { .. } => Route::UserOf,
        R::Repository { .. } => Route::RepositoryById,
        R::Locate { .. } => Route::Repository,
        R::Issues { .. } => Route::Issues,
        R::Issue { .. } => Route::Issue,
        R::Comments { .. } => Route::Comments,
        R::Pull { .. } => Route::Pull,
        R::PullsFor { .. } => Route::PullFor,
        R::Reviews { .. } => Route::Reviews,
        R::ReviewComments { .. } => Route::ReviewComments,
        R::CombinedStatus { .. } => Route::CombinedStatus,
        R::CheckRuns { .. } => Route::Runs,
        R::Job { .. } => Route::Job,
        R::JobLog { .. } => Route::JobLog,
        R::Branch { .. } => Route::Branch,
        R::Branches { .. } => Route::Branches,
        R::PullFiles { .. } => Route::PullFiles,
        R::Compare { .. } => Route::Compare,
        R::Commits { .. } => Route::Commits,
        R::Raw { .. } => Route::Raw,
        R::RulesForBranch { .. } | R::ClassicProtection { .. } => Route::Protection,
        R::Ruleset { .. } => Route::Protections,
        R::Collaborators { .. } => Route::Collaborators,
        R::Permission { .. } => Route::Permission,
        R::Reopen { .. } | R::Edit { .. } | R::Close { .. } => Route::EditIssue,
        R::Status { .. } => Route::PostStatus,
        R::CreateBranch { .. } => Route::CreateBranch,
        R::Reviewers { add: false, .. } => Route::RemoveReviewers,
        R::Reviewers { .. } => Route::RequestReviewers,
        R::Update { .. } => Route::Update,
        R::Review { .. } => Route::PostReview,
        R::Post { .. } => Route::PostComment,
        R::CreateIssue { .. } => Route::CreateIssue,
        R::OpenPull { .. } => Route::OpenPull,
        R::DeleteBranch { .. } => Route::DeleteBranch,
        R::LogPage { .. }
        | R::LogDelivery { .. }
        | R::Installations { .. }
        | R::Installation { .. }
        | R::Mint { .. }
        | R::Enqueue { .. }
        | R::PullGraphql { .. }
        | R::App
        | R::HookConfig
        | R::SetHook { .. }
        | R::Uninstall { .. }
        | R::Conversion { .. }
        | R::Followed { .. } => return None,
    })
}

