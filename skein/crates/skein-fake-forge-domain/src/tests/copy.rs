//! Seeded copy calls and deletion while alarms still hold repository ids.
use super::*;
use crate::api::Fill;

const COPY: &[u8] = b"org/copy";

fn copying(fill: Fill) -> Op {
    write(Write::Copy { owner: copy_of(b"org"), name: copy_of(b"copy"), fill })
}

fn begin(h: &mut Harness, repository: &[u8], user: u64, op: Op) -> Token {
    h.calls = h.calls.checked_add(1).expect("few calls");
    let token = Token::new(h.calls);
    step(
        &mut h.domain,
        &h.env,
        Event::Call { reply_to: ReplyTo::new(token), user, repository: copy_of(repository), op },
        &mut h.out,
    );
    token
}

fn finish(h: &mut Harness, token: Token) -> Result<Answer, Error> {
    for _ in 0_u32..256 {
        if let Some(request) = h.out.pop() {
            match request {
                Request::Reply { to, result } => {
                    assert_eq!(to.into_token(), token);
                    h.domain.reclaim();
                    return result;
                }
                hook @ Request::Hook { .. } => h.hooks.push(hook),
            }
        } else {
            h.env.now = h.domain.next_deadline().expect("pending copy alarm");
            fire(&mut h.domain, &h.env, &mut h.out);
        }
    }
    panic!("copy must answer");
}

fn empty(h: &Harness) {
    h.domain.inspect(&h.env.limits, COPY, &Read::Locate).expect("copy is visible");
    assert!(h.domain.branches(COPY).is_empty());
    assert!(h.domain.has(COPY).is_empty());
    assert_eq!(h.domain.permission(COPY, ENGINE), Ok(Permission::Admin));
    assert_eq!(h.domain.permission(COPY, PERSON), Ok(Permission::None));
    assert_eq!(h.domain.permission(COPY, STRANGER), Ok(Permission::None));
}

fn copied(result: Result<Answer, Error>) -> u32 {
    let Answer::Copied { repository, default_branch } = result.expect("copy succeeds") else {
        panic!("copy answer");
    };
    assert_eq!(&*default_branch, MAIN);
    repository
}

#[test]
fn a_copy_is_there_and_empty_from_its_call_and_whole_at_its_end() {
    let mut h = Harness::new(calm());
    let commit = h.commit(FIRST, &[(b"second", b"contents")]);
    h.push(ENGINE, b"work", commit).unwrap();
    let objects = h.domain.room().commits;
    let token = begin(&mut h, REPOSITORY, ENGINE, copying(Fill::Whole));
    empty(&h);
    h.wait(Duration::from_millis(500));
    empty(&h);
    let repository = copied(finish(&mut h, token));
    assert_ne!(repository, 2);
    assert_eq!(h.domain.branches(COPY).len(), 2);
    assert_eq!(h.domain.branch(COPY, MAIN), Some(FIRST));
    assert_eq!(h.domain.branch(COPY, b"work"), Some(commit));
    assert_eq!(h.domain.metadata(COPY).unwrap().default_branch.as_ref(), MAIN);
    assert_eq!(h.domain.has(COPY).len(), h.domain.has(REPOSITORY).len());
    for &id in h.domain.has(REPOSITORY) {
        assert!(h.domain.has(COPY).contains(&id));
    }
    assert_eq!(h.domain.room().commits, objects, "copies allocate no commits");
    h.push(ENGINE, b"later", commit).unwrap();
    assert_eq!(h.domain.branch(COPY, b"later"), None, "copy is not a mirror");
}

#[test]
fn a_slow_copy_lasts_its_fill() {
    let mut h = Harness::new(calm());
    let token = begin(&mut h, REPOSITORY, ENGINE, copying(Fill::Slow(Duration::from_secs(10))));
    h.until(Time::ZERO.saturating_add(Duration::from_secs(10)));
    empty(&h);
    assert!(h.answered.is_empty(), "latency plus fill has not elapsed");
    copied(finish(&mut h, token));
    assert_eq!(h.env.now, Time::ZERO.saturating_add(Duration::from_secs(11)));
    assert_eq!(h.domain.branch(COPY, MAIN), Some(FIRST));
}

#[test]
fn a_cut_copy_times_out_and_stays_empty() {
    let mut h = Harness::new(calm());
    assert_eq!(h.call(ENGINE, copying(Fill::Cut)), Err(Error::Timeout));
    empty(&h);
    h.settle();
    empty(&h);
    assert_eq!(h.domain.tally().timeouts, 1);
}

#[test]
fn a_copy_takes_the_domains_faults() {
    let mut h = Harness::new(Config { unavailable: 1000, ..calm() });
    assert_eq!(h.call(ENGINE, copying(Fill::Whole)), Err(Error::Unavailable));
    assert_eq!(h.domain.inspect(&calm(), COPY, &Read::Locate), Err(Error::Missing(What::Repository)));
    assert_eq!(h.domain.repositories.len(), 1);
    assert_eq!(h.domain.tally().unavailable, 1);
    h.env.limits.unavailable = 0;
    h.env.limits.timeouts = 1000;
    assert_eq!(h.call(ENGINE, copying(Fill::Whole)), Err(Error::Timeout));
    empty(&h);
    h.settle();
    empty(&h);
    assert_eq!(h.domain.tally().timeouts, 1);
    let mut h = Harness::new(Config { landing: 1000, ..calm() });
    copied(h.call(ENGINE, copying(Fill::Whole)));
    h.settle();
    assert_eq!(h.domain.tally().landed, 0, "copy uses fill, never late landing");
}

#[test]
fn a_copy_of_a_name_taken_is_exists_and_makes_nothing() {
    let mut h = Harness::new(calm());
    let id = copied(h.call(ENGINE, copying(Fill::Whole)));
    let objects = h.domain.room().commits;
    assert_eq!(h.call(ENGINE, copying(Fill::Whole)), Err(Error::Exists));
    assert_eq!(h.domain.inspect(&calm(), COPY, &Read::Locate), Ok(Answer::Located { id }));
    assert_eq!(h.domain.repositories.len(), 2);
    assert_eq!(h.domain.room().commits, objects);
    assert_eq!(h.domain.branch(COPY, MAIN), Some(FIRST));
    let other = write(Write::Copy { owner: copy_of(b"org"), name: copy_of(b"other"), fill: Fill::Whole });
    assert_eq!(h.call(ENGINE, other), Err(Error::Full));
    assert_eq!(h.domain.inspect(&calm(), b"org/other", &Read::Locate), Err(Error::Missing(What::Repository)));
}

#[test]
fn the_source_hears_nothing() {
    let mut h = Harness::new(calm());
    copied(h.call(ENGINE, copying(Fill::Whole)));
    // Even refused copies must not produce a refusal naming the source.
    assert_eq!(h.call(ENGINE, copying(Fill::Whole)), Err(Error::Exists));
    h.settle();
    assert_eq!(h.domain.branch(REPOSITORY, MAIN), Some(FIRST));
    assert_eq!(h.domain.branches(REPOSITORY).len(), 1);
    assert_eq!(h.domain.has(REPOSITORY).len(), 1);
    assert!(h.domain.pop_observation().is_none());
    assert!(h.hooks.is_empty());
    assert_eq!(h.domain.tally().verdicts, 0);
}

#[test]
fn a_deleted_repository_frees_its_name_and_reads_nothing() {
    let mut config = calm();
    config.limits.repositories = 3;
    let mut h = Harness::new(config.clone());
    let id = copied(h.call(ENGINE, copying(Fill::Whole)));
    let slot = h.domain.id(COPY);
    assert_eq!(h.call_on(COPY, PERSON, write(Write::DeleteRepository)), Err(Error::Forbidden));
    assert_eq!(h.call_on(COPY, ENGINE, write(Write::DeleteRepository)), Ok(Answer::Done));
    assert_eq!(h.domain.inspect(&config, COPY, &Read::Locate), Err(Error::Missing(What::Repository)));
    assert_eq!(h.domain.metadata(COPY), Err(Error::Missing(What::Repository)));
    assert_eq!(h.call_on(COPY, ENGINE, read(Read::Locate)), Err(Error::Missing(What::Repository)));
    assert_eq!(crate::reads::read(&h.domain, &config, slot, &Read::Locate), Err(Error::Missing(What::Repository)));
    assert!(h.domain.repositories.get(slot).unwrap().gone);
    let new = copied(h.call(ENGINE, copying(Fill::Whole)));
    assert_ne!(new, id);
    assert_ne!(h.domain.id(COPY), slot);
    assert_eq!(h.domain.branch(COPY, MAIN), Some(FIRST));
    assert_eq!(crate::reads::read(&h.domain, &config, slot, &Read::Locate), Err(Error::Missing(What::Repository)));
}

#[test]
fn a_deleted_copy_ends_its_call() {
    let mut h = Harness::new(calm());
    let token = begin(&mut h, REPOSITORY, ENGINE, copying(Fill::Slow(Duration::from_secs(10))));
    let slot = h.domain.id(COPY);
    assert_eq!(h.call_on(COPY, ENGINE, write(Write::DeleteRepository)), Ok(Answer::Done));
    assert_eq!(finish(&mut h, token), Err(Error::Missing(What::Repository)));
    h.settle();
    assert_eq!(h.domain.calls(), 0);
    assert!(h.domain.repositories.get(slot).unwrap().branches.is_empty());
    assert!(h.domain.repositories.get(slot).unwrap().has.is_empty());
    assert_eq!(h.domain.inspect(&calm(), COPY, &Read::Locate), Err(Error::Missing(What::Repository)));
}

#[test]
fn a_deleted_source_is_passed_over_by_pending_alarms() {
    let mut h = Harness::new(calm());
    h.push(ENGINE, b"work", FIRST).unwrap();
    while h.domain.pop_observation().is_some() {}
    assert_eq!(h.call(ADMIN, write(Write::DeleteRepository)), Ok(Answer::Done));
    let verdicts = h.domain.tally().verdicts;
    h.settle();
    assert_eq!(h.domain.tally().verdicts, verdicts);
    assert!(h.domain.pop_observation().is_none());
    assert!(h.hooks.is_empty());
    assert_eq!(h.domain.deliveries(), 0);
}
