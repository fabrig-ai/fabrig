use super::*;
use crate::seam::Access;
use crate::{
    Account, Change, Component, Fact, Request,
    installations::{Metadata, Row},
    tests::fixtures,
};
use alloc::boxed::Box;
use skein_lib::{List, Time, Wall};

impl Component {
    fn github_progress(&mut self, progress: Progress, env: &Env<Limits>, up: &mut Queue<Event>) {
        let super::super::ForgeEntry { base, dialect } = self.forges.get_mut(0).expect("forge");
        match dialect {
            super::super::Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            super::super::Dialect::GitHub(gh) => self.common.github_progress(base, gh, progress, env, up),
        }
    }
    fn github_local(&mut self, env: &Env<Limits>, up: &mut Queue<Event>) -> bool {
        let super::super::ForgeEntry { base, dialect } = self.forges.get_mut(0).expect("forge");
        match dialect {
            super::super::Dialect::Forgejo { .. } => unreachable!("fixture dialect"),
            super::super::Dialect::GitHub(gh) => self.common.github_local(base, gh, env, up),
        }
    }
}
// Twenty rows include lifecycle then rename on id 1.
const ROWS: u32 = 20;
fn pending() -> (Component, Env<Limits>) {
    let limits = Limits { installations: ROWS, ..fixtures::limits() };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
    let gh = component.test_github_mut();
    gh.baseline = false;
    drop(
        gh.rows
            .insert(
                1,
                Row {
                    state: Tokens::Gone,
                    revision: 0,
                    token_revision: 0,
                    refused_revision: None,
                    full: Clock::empty(),
                    contents: Clock::empty(),
                    account: Account { id: 22, login: Box::from(b"acme".as_slice()) },
                    suspended: false,
                    accepted: false,
                    api: None,
                    permissions: document::Levels::NONE,
                    api_expires: Wall::EPOCH,
                    git_expires: Wall::EPOCH,
                },
            )
            .expect("old row"),
    );
    gh.begin_read(limits.start_backoff);
    match &mut gh.read {
        Read::Reading(reading) => {
            let _seen = reading.seen.insert(1).expect("seen");
            for id in 1..=ROWS {
                drop(
                    reading
                        .staged
                        .insert(
                            u64::from(id),
                            Metadata {
                                account: Account {
                                    id: u64::from(id).saturating_add(21),
                                    login: Box::from(b"renamed".as_slice()),
                                },
                                suspended: true,
                                seen: true,
                            },
                        )
                        .expect("staged rows"),
                );
            }
        }
        Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("reading"),
    }
    gh.commit();
    (component, env)
}
fn installation(id: u64, login: &[u8]) -> document::Installation {
    document::Installation {
        id,
        account: 22,
        login: Box::from(login),
        kind: document::Account::Organization,
        created: Wall::EPOCH,
        suspended: false,
    }
}
fn queues() -> (Queue<Event>, Queue<skein_io::Request>) {
    (Queue::with_capacity(128), Queue::with_capacity(128))
}
#[test]
fn installation_output_preserves_lifecycle_then_rename() {
    let (mut component, env) = pending();
    let (mut up, mut io) = queues();
    component.test_local(&env, &mut up);
    match up.pop().expect("first phase") {
        Event::Installation { installation, change, .. } => {
            assert_eq!((installation, change), (1, Change::Suspended));
        }
        Event::Fact { .. }
        | Event::GitToken { .. }
        | Event::Answered(_)
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed => unreachable!("lifecycle"),
    }
    let mut changes = List::with_capacity(ROWS.saturating_add(1));
    for _ in 0_u32..4 {
        component.fire(&env, &mut up, &mut io);
        for _ in 0..up.len() {
            match up.pop().expect("queued") {
                Event::Installation { installation, change, .. } => {
                    changes.push((installation, change)).expect("bounded changes");
                }
                Event::Connected { .. } => assert_eq!(changes.len(), ROWS),
                Event::Fact { .. }
                | Event::GitToken { .. }
                | Event::Answered(_)
                | Event::Hint(_)
                | Event::Listening { .. }
                | Event::ListenerLost
                | Event::Stopped { .. }
                | Event::Closed => unreachable!("change output"),
            }
        }
    }
    assert_eq!(changes.len(), ROWS);
    assert_eq!(changes.get(0), Some(&(1, Change::Renamed)));
    for (index, (id, change)) in changes.as_slice().iter().enumerate().skip(1) {
        assert_eq!((*id, *change), (u64::try_from(index).expect("index").saturating_add(1), Change::Created));
    }
    assert!(io.is_empty());
    assert_eq!(component.next_deadline(), Some(env.now.saturating_add(env.limits.log_poll)));
}
#[test]
fn held_unkept_precedes_the_next_page() {
    let limits = Limits { installations: 1, ..fixtures::limits() };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
    let gh = component.test_github_mut();
    gh.begin_read(limits.start_backoff);
    let mut rows = List::with_capacity(2);
    for id in [1, 2] {
        rows.push(installation(id, b"name")).expect("page");
    }
    match &mut gh.read {
        Read::Reading(reading) => {
            reading
                .page(rows.into_boxed(), &gh.rows, &gh.accepted, fixtures::shared().name_bytes)
                .expect("bounded page");
            reading.page = 2;
        }
        Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("reading"),
    }
    let (mut up, mut io) = queues();
    component.test_local(&env, &mut up);
    match up.pop().expect("held fact") {
        Event::Fact { fact, .. } => assert_eq!(fact, Fact::Unkept { installation: 2 }),
        Event::Installation { .. }
        | Event::GitToken { .. }
        | Event::Answered(_)
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed => unreachable!("fact"),
    }
    assert!(io.is_empty());
    component.fire(&env, &mut up, &mut io);
    assert_eq!(io.len(), 1);
    assert!(up.is_empty());
}
#[test]
fn close_abandons_pending_installation_output() {
    let (mut component, env) = pending();
    let (mut up, mut io) = queues();
    component.test_local(&env, &mut up);
    drop(up.pop());
    component.down(&env, Request::Close, &mut up, &mut io);
    match up.pop().expect("closed") {
        Event::Closed => {}
        Event::Installation { .. }
        | Event::Fact { .. }
        | Event::GitToken { .. }
        | Event::Answered(_)
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. } => unreachable!("Closed last"),
    }
    component.fire(&env, &mut up, &mut io);
    assert!(up.is_empty());
    assert!(io.is_empty());
}
#[test]
fn connect_replacement_abandons_pending_installation_output() {
    let (mut component, env) = pending();
    let (mut up, mut io) = queues();
    component.test_local(&env, &mut up);
    drop(up.pop());
    component.down(
        &env,
        Request::Connect {
            forge: fixtures::FORGE,
            access: fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("public key"), 7, 9),
        },
        &mut up,
        &mut io,
    );
    assert!(up.is_empty());
    assert_eq!(io.len(), 1);
    let gh = component.test_github();
    match gh.read {
        Read::Reading(_) => {}
        Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("replacement read"),
    }
}
#[test]
fn installation_page_number_overflow_fails_the_attempt() {
    let limits = fixtures::limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
    let gh = component.test_github_mut();
    gh.begin_read(limits.start_backoff);
    match &mut gh.read {
        Read::Reading(reading) => reading.page = u32::MAX,
        Read::Idle | Read::Changes(_) | Read::Waiting { .. } => unreachable!("reading"),
    }
    let (mut up, mut io) = queues();
    component.github_progress(Progress::Paged(Ok(InstallationPage { rows: Box::from([]), next: true })), &env, &mut up);
    assert_eq!(
        component.start(fixtures::FORGE),
        Some(Start::Waiting {
            until: env.now.saturating_add(limits.start_backoff),
            next: limits.start_backoff.saturating_mul(2)
        })
    );
    assert!(up.is_empty());
    component.fire(&env, &mut up, &mut io);
    assert!(io.is_empty());
}

#[test]
fn failed_later_page_preserves_committed_table() {
    let limits = Limits { installations: 1, ..fixtures::limits() };
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
    component.forges.get_mut(0).expect("forge").base.start = Start::Ready { cap: 100 };
    let gh = component.test_github_mut();
    gh.baseline = false;
    gh.connected = true;
    gh.identity_checked = true;
    gh.accept(&[1]);
    let token = b"artificial-api";
    let expiry = Wall::from_nanos(3_600_000_000_000);
    drop(
        gh.rows
            .insert(
                1,
                Row {
                    state: Tokens::Current,
                    revision: 0,
                    token_revision: 0,
                    refused_revision: None,
                    full: Clock { expiry: Time::from_nanos(expiry.as_nanos()), ..Clock::empty() },
                    contents: Clock::empty(),
                    account: Account { id: 22, login: Box::from(b"acme".as_slice()) },
                    suspended: false,
                    accepted: true,
                    api: Some(skein_lib::Secret::new(Box::from(token.as_slice()))),
                    permissions: document::Levels::NONE,
                    api_expires: expiry,
                    git_expires: expiry,
                },
            )
            .expect("committed row"),
    );
    gh.begin_read(limits.start_backoff);
    let (mut up, mut io) = queues();
    let mut staged = List::with_capacity(2);
    for (id, login) in [(1, b"renamed".as_slice()), (2, b"stray".as_slice())] {
        staged.push(installation(id, login)).expect("page rows");
    }
    component.github_progress(
        Progress::Paged(Ok(InstallationPage { rows: staged.into_boxed(), next: true })),
        &env,
        &mut up,
    );
    component.test_local(&env, &mut up);
    component.github_progress(Progress::Paged(Err(api::Error::InvalidAnswer)), &env, &mut up);
    let gh = component.test_github();
    let row = gh.rows.get(&1).expect("committed row");
    assert_eq!((row.account.login.as_ref(), row.api_expires, row.git_expires), (b"acme".as_slice(), expiry, expiry));
    assert!(row.api.as_ref().expect("API token").expose() == token);
    assert_eq!(gh.rows.len(), 1);
    assert_eq!(up.len(), 1);
    match up.pop().expect("capacity diagnostic survives failure") {
        Event::Fact { fact, .. } => assert_eq!(fact, Fact::Unkept { installation: 2 }),
        Event::Installation { .. }
        | Event::GitToken { .. }
        | Event::Answered(_)
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed => unreachable!("one fact"),
    }
    env.now = env.now.saturating_add(limits.start_backoff);
    let _progressed = component.github_local(&env, &mut up);
    let page = installation(1, b"acme");
    component.github_progress(
        Progress::Paged(Ok(InstallationPage { rows: Box::from([page]), next: false })),
        &env,
        &mut up,
    );
    component.drive(&env, &mut up, &mut io);
    assert!(up.is_empty());
    assert!(io.is_empty());
    assert!(component.test_github().rows.get(&1).expect("row").api.as_ref().expect("preserved").expose() == token);
}

#[path = "../../tests/github_events.rs"]
mod events;
