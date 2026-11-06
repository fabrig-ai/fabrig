#![expect(clippy::disallowed_methods, reason = "ordinary assertions inspect captured requests")]
#![expect(clippy::match_like_matches_macro, reason = "explicit test patterns follow the step subset")]
use super::*;
use skein_lib::Wall;

const FORGE: u16 = 1;
const OTHER_FORGE: u16 = 2;
pub(crate) fn row(number: u32) -> RepositoryMaking {
    RepositoryMaking {
        key: MakingKey { forge: FORGE, org: Box::from(b"owner".as_slice()), name: number.to_be_bytes().into() },
        from: MakeFrom::Copy { address: Box::from(b"https://source.invalid/owner/repo".as_slice()) },
        into: Some(MakeInto { project: 1, home: true, host: Box::from(b"forge.invalid".as_slice()) }),
        asked: Wall::from_nanos(u64::from(number)),
        copying: true,
        state: MakingState::Waiting,
    }
}
fn restore(d: &mut Domain, row: RepositoryMaking) {
    let requests = outputs(d, Event::Restore { record: Stored::Making(row) });
    assert!(requests.is_empty());
}
#[test]
fn makings_restore_at_the_bound() {
    let mut d = domain();
    for number in 1..=LIMITS.makings {
        restore(&mut d, row(number));
    }
    assert_eq!(d.restore_fit_failure(), None);
    let mut numbers = 1..=LIMITS.makings;
    for making in d.makings(FORGE) {
        let number = numbers.next().expect("no row past the bound");
        assert_eq!(making, &row(number));
    }
    assert_eq!(numbers.next(), None);
    assert_eq!(d.makings(OTHER_FORGE).count(), 0);
}
#[test]
fn a_making_past_the_bound_is_refused_at_restart() {
    let mut d = domain();
    for number in 1..=LIMITS.makings + 1 {
        restore(&mut d, row(number));
    }
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Making),
            set: RestoreSet::Rows(RecordKind::Making),
            held: u64::from(LIMITS.makings),
            additional: 1,
            limit: u64::from(LIMITS.makings)
        })
    );
    assert_eq!(d.making(&row(LIMITS.makings + 1).key), None);
}
#[test]
fn a_row_making_at_restart_comes_back_waiting_with_its_mark() {
    let mut d = domain();
    let expected = row(1);
    let mut making = expected.clone();
    making.state = MakingState::Making;
    restore(&mut d, making);
    assert_eq!(d.making(&expected.key), Some(&expected));
}
#[test]
fn next_making_is_the_oldest_waiting() {
    let mut d = domain();
    let mut older = row(2);
    older.asked = Wall::EPOCH;
    restore(&mut d, row(1));
    restore(&mut d, older.clone());
    assert_eq!(d.next_making(FORGE), Some(&older));
    assert_eq!(d.next_making(OTHER_FORGE), None);
    let mut tied = row(1);
    tied.asked = Wall::EPOCH;
    restore(&mut d, tied.clone());
    assert_eq!(d.next_making(FORGE), Some(&tied));
    tied.state = MakingState::Made { id: 42, branch: Box::from(b"main".as_slice()) };
    restore(&mut d, tied.clone());
    assert_eq!(d.next_making(FORGE), Some(&older));
    assert_eq!(d.making_due(FORGE), Some(tied.key));
    assert_eq!(d.making_due(OTHER_FORGE), None);
}

fn kept() -> Domain {
    let mut d = domain();
    restore_forge(&mut d, removal::record(FORGE, Mode::Owned));
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}
fn restore_forge(d: &mut Domain, record: ForgeRecord) {
    outputs(d, Event::Restore { record: Stored::Forge(record) });
}
fn intent() -> RepositoryMaking {
    let mut value = row(1);
    value.key.org = b"org".as_slice().into();
    value.key.name = b"repo".as_slice().into();
    value.copying = false;
    value
}
fn ask(d: &mut Domain, value: &RepositoryMaking, role: Option<jig_domain_people::Role>) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(101),
            token: Token::new(201),
            retention: Duration::from_secs(60),
            role,
            carried: None,
            ask: Ask::MakeRepository {
                forge: value.key.forge,
                org: value.key.org.clone(),
                name: value.key.name.clone(),
                from: value.from.clone(),
                into: value.into.clone(),
            },
        },
    )
}
fn owner(d: &mut Domain, value: &RepositoryMaking) -> Box<[Request]> {
    ask(d, value, Some(jig_domain_people::Role::Owner))
}
fn answer(d: &Domain, value: MakeAnswer) {
    assert_eq!(d.application_answer(Token::new(201)), Some(Answered::Make(value)));
}
fn failure(reason: MakeFailure) -> MakingFailed {
    MakingFailed { step: MakeStep::Repository, reason, words: Box::from(b"forge refused".as_slice()) }
}
fn saved(requests: &[Request]) -> &RepositoryMaking {
    requests
        .iter()
        .find_map(
            |request| {
                if let Request::Save { record: Stored::Making(row), .. } = request { Some(row) } else { None }
            },
        )
        .expect("making saved")
}
fn erased(requests: &[Request], key: &MakingKey) {
    assert!(requests.iter().any(|request| if let Request::Erase { key: Key::Making(erased) } = request {
        erased == key
    } else {
        false
    }));
}
fn made(d: &mut Domain, value: &RepositoryMaking) -> Box<[Request]> {
    outputs(
        d,
        Event::MakingMade {
            key: value.key.clone(),
            id: u64::from(REPO.repository),
            branch: b"trunk".as_slice().into(),
        },
    )
}
fn act(d: &mut Domain, value: &RepositoryMaking) -> ([u8; 16], Adoption) {
    let requests = outputs(d, Event::MakingAdopt { key: value.key.clone() });
    let [Request::MakingAct { key, act, adoption, repository }] = requests.as_ref() else { panic!("one act") };
    assert_eq!(key, &value.key);
    assert_eq!(*repository, REPO);
    (*act, adoption.clone())
}
#[test]
fn only_an_owner_may_make() {
    for role in [
        None,
        Some(jig_domain_people::Role::Observer),
        Some(jig_domain_people::Role::Member),
        Some(jig_domain_people::Role::Maintainer),
        Some(jig_domain_people::Role::Owner),
    ] {
        let mut d = kept();
        owner_or_refuse(&mut d, role);
    }
}
fn owner_or_refuse(d: &mut Domain, role: Option<jig_domain_people::Role>) {
    ask(d, &intent(), role);
    let expected = if role == Some(jig_domain_people::Role::Owner) {
        MakeAnswer::Kept
    } else {
        MakeAnswer::Refused(MakeRefusal::NotOwner)
    };
    answer(d, expected);
    assert_eq!(d.making(&intent().key).is_some(), expected == MakeAnswer::Kept);
}
#[test]
fn a_forge_not_kept_or_ended_is_no_such_forge() {
    for ended in [false, true] {
        let mut d = domain();
        if ended {
            let mut record = removal::record(FORGE, Mode::Owned);
            record.state = ForgeState::Ended { secrets_erased: false, box_removed: false };
            restore_forge(&mut d, record);
        }
        owner(&mut d, &intent());
        answer(&d, MakeAnswer::Refused(MakeRefusal::NoSuchForge));
        assert!(d.making(&intent().key).is_none());
    }
}
#[test]
fn github_and_a_connected_forgejo_are_not_offered() {
    for (kind, mode, expected) in [
        (Kind::GitHub, Mode::Owned, false),
        (Kind::Forgejo, Mode::Connected, false),
        (Kind::Forgejo, Mode::Fake, false),
        (Kind::Forgejo, Mode::Owned, true),
        (Kind::Forgejo, Mode::Managed, true),
    ] {
        let mut d = kept();
        let mut record = removal::record(FORGE, mode);
        record.kind = kind;
        restore_forge(&mut d, record);
        owner(&mut d, &intent());
        answer(&d, if expected { MakeAnswer::Kept } else { MakeAnswer::Refused(MakeRefusal::NotOffered) });
    }
}
#[test]
fn a_name_or_address_breaking_the_grammar_is_invalid() {
    for (org, name, address) in [
        (b"-org".as_slice(), b"repo".as_slice(), b"host.invalid/org/repo".as_slice()),
        (b"org", b"..", b"host.invalid/org/repo"),
        (b"org", b"a/b", b"host.invalid/org/repo"),
        (b"org", b"repo", b"http://host.invalid/org/repo"),
        (b"org", b"repo", b"host.invalid/a/b/c"),
        (b"org", b"repo", b"git@host.invalid:a/b"),
        ([b'a'; 41].as_slice(), b"repo", b"host.invalid/org/repo"),
        (b"org", [b'a'; 101].as_slice(), b"host.invalid/org/repo"),
        (b"org", b"repo", [b'a'; 129].as_slice()),
    ] {
        let mut d = kept();
        let mut value = intent();
        value.key.org = org.into();
        value.key.name = name.into();
        value.from = MakeFrom::Copy { address: address.into() };
        owner(&mut d, &value);
        answer(&d, MakeAnswer::Refused(MakeRefusal::Invalid));
        assert!(d.making(&value.key).is_none());
    }
}
#[test]
fn a_home_in_a_project_with_one_is_home_held() {
    let mut d = kept();
    let mut home = adopted();
    home.project = 1;
    outputs(&mut d, Event::Restore { record: Stored::Repository(home) });
    owner(&mut d, &intent());
    answer(&d, MakeAnswer::Refused(MakeRefusal::HomeHeld));
    assert!(d.making(&intent().key).is_none());
}
#[test]
fn past_makings_is_full() {
    let mut d = kept();
    for number in 1..=LIMITS.makings {
        restore(&mut d, row(number));
    }
    owner(&mut d, &intent());
    answer(&d, MakeAnswer::Refused(MakeRefusal::Full));
    assert!(d.making(&intent().key).is_none());
}
#[test]
fn a_kept_making_writes_a_waiting_row() {
    let mut d = kept();
    let value = intent();
    let requests = owner(&mut d, &value);
    answer(&d, MakeAnswer::Kept);
    let expected = RepositoryMaking { asked: env().wall, ..value };
    assert_eq!(saved(&requests), &expected);
    assert_eq!(d.making(&expected.key), Some(&expected));
    assert!(requests.iter().any(|r| if let Request::MarkedForges = r { true } else { false }));
    assert!(requests.iter().any(|r| {
        if let Request::Save { record: Stored::Application { outcome: ApplicationOutcome::MakeKept, .. }, .. } = r {
            true
        } else {
            false
        }
    }));
    let Some(Request::ApplicationDone { .. }) = requests.last() else { panic!("answer follows saves") };
    let mut restart = kept();
    for request in &requests {
        if let Request::Save { record: record @ Stored::Application { .. }, .. } = request {
            outputs(&mut restart, Event::Restore { record: record.clone() });
        }
    }
    answer(&restart, MakeAnswer::Kept);
    let mut saturated = kept();
    for number in 0..LIMITS.adoptions {
        outputs(
            &mut saturated,
            Event::MakingAsked {
                request: Token::new(400 + u64::from(number)),
                adoption: fixtures::adoption(),
                repository: skein_forge::api::Repository { forge: FORGE, repository: number + 2 },
            },
        );
    }
    assert_eq!(saturated.adoptions.len(), LIMITS.adoptions);
    let requests = owner(&mut saturated, &intent());
    answer(&saturated, MakeAnswer::Kept);
    assert_eq!(saved(&requests).state, MakingState::Waiting);
}
#[test]
fn a_second_request_is_answered_as_the_first() {
    for state in [
        MakingState::Waiting,
        MakingState::Making,
        MakingState::Made { id: 2, branch: b"main".as_slice().into() },
        MakingState::Adopting { id: 2, branch: b"main".as_slice().into() },
    ] {
        let mut d = kept();
        let mut value = intent();
        value.state = state;
        // A live Making restores as Waiting; admission sees the live transition too.
        restore(&mut d, value.clone());
        if value.state == MakingState::Making {
            outputs(&mut d, Event::MakingStarted { key: value.key.clone() });
        }
        let mut changed = value.clone();
        changed.from = MakeFrom::Empty;
        changed.into = None;
        let requests = owner(&mut d, &changed);
        answer(&d, MakeAnswer::Kept);
        assert_eq!(d.making(&value.key), Some(&value));
        assert!(!requests.iter().any(|r| if let Request::Save { record: Stored::Making(_), .. } = r {
            true
        } else {
            false
        }));
    }
}
#[test]
fn a_failed_row_asked_again_is_replaced_keeping_its_mark_for_the_same_address_alone() {
    for (from, expected) in [
        (intent().from, true),
        (MakeFrom::Copy { address: b"https://source.invalid/owner/other".as_slice().into() }, false),
        (MakeFrom::Empty, false),
    ] {
        let mut d = kept();
        let mut failed = intent();
        failed.copying = true;
        failed.state = MakingState::Failed(failure(MakeFailure::Other));
        restore(&mut d, failed);
        let mut value = intent();
        value.from = from;
        let requests = owner(&mut d, &value);
        answer(&d, MakeAnswer::Kept);
        assert_eq!(saved(&requests), &RepositoryMaking { copying: expected, asked: env().wall, ..value });
    }
}
#[test]
fn the_mark_is_saved_and_told_after() {
    let mut d = kept();
    let value = intent();
    restore(&mut d, value.clone());
    let started = outputs(&mut d, Event::MakingStarted { key: value.key.clone() });
    assert_eq!(saved(&started).state, MakingState::Making);
    let requests = outputs(&mut d, Event::MakingCopying { key: value.key.clone() });
    assert!(saved(&requests).copying);
    assert!(d.making(&value.key).unwrap().copying);
    let saved_at = requests
        .iter()
        .position(|r| if let Request::Save { record: Stored::Making(_), .. } = r { true } else { false })
        .unwrap();
    let told_at = requests
        .iter()
        .position(|r| if let Request::MakingCopied { key } = r { key == &value.key } else { false })
        .unwrap();
    assert!(saved_at < told_at);
}
#[test]
fn a_stop_at_taken_clears_the_mark() {
    for reason in [MakeFailure::Taken, MakeFailure::Other] {
        let mut d = kept();
        let mut value = intent();
        value.copying = true;
        restore(&mut d, value.clone());
        let mut failed = failure(reason.clone());
        failed.words = Box::from([b'x'; 300]);
        let requests = outputs(&mut d, Event::MakingStopped { key: value.key.clone(), failed });
        assert_eq!(saved(&requests).copying, reason != MakeFailure::Taken);
        let MakingState::Failed(failed) = &saved(&requests).state else { panic!("failed row") };
        assert_eq!(failed.words.len(), 256);
    }
    assert!(
        outputs(&mut kept(), Event::MakingStopped { key: intent().key, failed: failure(MakeFailure::Taken) })
            .is_empty()
    );
}
#[test]
fn made_with_no_into_is_erased() {
    let mut d = kept();
    let mut value = intent();
    value.into = None;
    restore(&mut d, value.clone());
    let requests = made(&mut d, &value);
    erased(&requests, &value.key);
    assert!(d.making(&value.key).is_none());
}
#[test]
fn made_with_into_is_adopting() {
    let mut d = kept();
    let value = intent();
    restore(&mut d, value.clone());
    let requests = made(&mut d, &value);
    assert_eq!(saved(&requests).state, MakingState::Adopting { id: 2, branch: b"trunk".as_slice().into() });
    assert_eq!(d.making(&value.key), Some(saved(&requests)));
}
#[test]
fn making_adopt_asks_the_act_once() {
    let mut d = kept();
    let value = intent();
    restore(&mut d, value.clone());
    made(&mut d, &value);
    let (key, adoption) = act(&mut d, &value);
    assert_eq!(key, [0x2e, 0x27, 0xef, 0xde, 0x86, 0x67, 0x90, 0x19, 0x64, 0xe8, 0xd4, 0x17, 0x50, 0xab, 0x02, 0x40]);
    assert_eq!(
        adoption,
        Adoption {
            project: 1,
            home: true,
            forge: FORGE,
            host: b"forge.invalid".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: Role::Owned,
            landing: b"trunk".as_slice().into(),
            ci: true,
            checks: Box::new([])
        }
    );
    assert!(outputs(&mut d, Event::MakingAdopt { key: value.key.clone() }).is_empty());
    assert_eq!(d.making_due(FORGE), None);
    outputs(&mut d, Event::MakingAgain { key: value.key.clone() });
    assert_eq!(d.making_due(FORGE), Some(value.key.clone()));
    assert_eq!(act(&mut d, &value).0, key);
}
#[test]
fn the_key_is_the_same_after_a_restart() {
    let mut d = kept();
    let value = intent();
    restore(&mut d, value.clone());
    made(&mut d, &value);
    let before = act(&mut d, &value);
    let row = d.making(&value.key).unwrap().clone();
    let mut restart = kept();
    restore(&mut restart, row.clone());
    assert_eq!(act(&mut restart, &value), before);
    let mut later = kept();
    restore(&mut later, RepositoryMaking { asked: Wall::from_nanos(2), ..row });
    assert_ne!(act(&mut later, &value).0, before.0);
}
fn begin_act(d: &mut Domain) -> (RepositoryMaking, Adoption) {
    let value = intent();
    restore(d, value.clone());
    made(d, &value);
    let (_, adoption) = act(d, &value);
    outputs(d, Event::MakingAsked { request: Token::new(301), adoption: adoption.clone(), repository: REPO });
    (value, adoption)
}
#[test]
fn making_asked_adopts_with_no_locate() {
    use skein_forge::api;
    let mut d = kept();
    let (value, adoption) = begin_act(&mut d);
    assert_eq!(d.adoptions.len(), 1);
    let requests = ready(&mut d);
    let [Request::Call { call, repository, op }] = requests.as_ref() else { panic!("one permission read") };
    assert_eq!(*repository, REPO);
    assert_eq!(*op, api::Op::Read(api::Read::Permission { user: 7 }));
    fixtures::answer(
        &mut d,
        *call,
        Ok(api::Answer::Permission(api::UserPermission {
            user: 1,
            login: Box::from(b"".as_slice()),
            permission: api::Permission::Write,
        })),
    );
    for result in
        [api::Answer::Branches(Box::new([])), api::Answer::Settings(adopted().settings), api::Answer::Protection(None)]
    {
        fixtures::answer_next(&mut d, Ok(result));
    }
    let requests =
        fixtures::answer_next(&mut d, Ok(api::Answer::Collaborators { collaborators: Box::new([]), more: false }));
    let Some(Request::Adopted { reply_to, result: Ok(result) }) = requests.last() else {
        panic!("act adoption answered")
    };
    assert_eq!(*reply_to, Token::new(301));
    assert_eq!(result.repository.provider, REPO);
    assert_eq!(result.repository.project, adoption.project);
    assert_eq!(result.repository.prefix, adoption.prefix);
    assert!(d.application_outcome(Token::new(301)).is_none());
    assert_eq!(d.adoptions.len(), 0);
    assert!(d.making(&value.key).is_some());

    let mut full = kept();
    for number in 0..LIMITS.adoptions {
        let requests = outputs(
            &mut full,
            Event::MakingAsked {
                request: Token::new(400 + u64::from(number)),
                adoption: adoption.clone(),
                repository: api::Repository { forge: FORGE, repository: number + 2 },
            },
        );
        assert!(!requests.iter().any(|r| if let Request::ApplicationAgain { .. } = r { true } else { false }));
    }
    let requests = outputs(
        &mut full,
        Event::MakingAsked { request: Token::new(499), adoption: adoption.clone(), repository: REPO },
    );
    assert_eq!(
        requests.as_ref(),
        &[Request::ApplicationAgain { request: Token::new(499), why: jig_domain_people::Again::Busy }]
    );
    assert_eq!(full.adoptions.len(), LIMITS.adoptions);
    let requests = outputs(
        &mut full,
        Event::Application {
            request: Token::new(500),
            token: Token::new(600),
            retention: Duration::from_secs(60),
            role: Some(jig_domain_people::Role::Owner),
            carried: None,
            ask: Ask::Adopt(adoption),
        },
    );
    assert_eq!(
        requests.as_ref(),
        &[Request::ApplicationAgain { request: Token::new(500), why: jig_domain_people::Again::Busy }]
    );
}
#[test]
fn adopted_is_erased() {
    let mut d = kept();
    let value = intent();
    restore(&mut d, value.clone());
    made(&mut d, &value);
    act(&mut d, &value);
    let requests = outputs(&mut d, Event::MakingActed { key: value.key.clone(), outcome: None });
    erased(&requests, &value.key);
    assert!(d.making(&value.key).is_none());
    assert_eq!(d.making_due(FORGE), None);
}
#[test]
fn refused_is_failed_at_adoption_and_the_repository_stays() {
    use skein_forge::api;
    for reason in [MakeFailure::NotAllowed, MakeFailure::Refused { status: 403 }] {
        let mut d = kept();
        let (value, _) = begin_act(&mut d);
        let requests = fixtures::answer_next(&mut d, Err(api::Error::Forbidden));
        assert!(if let Some(Request::Adopted { reply_to, result: Err(api::Error::Forbidden) }) = requests.last() {
            *reply_to == Token::new(301)
        } else {
            false
        });
        let failed = MakingFailed { step: MakeStep::Adoption, reason, words: b"not allowed".as_slice().into() };
        let requests = outputs(&mut d, Event::MakingActed { key: value.key.clone(), outcome: Some(failed.clone()) });
        assert_eq!(saved(&requests).state, MakingState::Failed(failed));
        assert!(d.making(&value.key).is_some());
        assert!(d.repository(REPO).is_none());
        assert!(!requests.iter().any(|r| if let Request::Call { .. } = r { true } else { false }));
    }
}
#[test]
fn a_removal_erases_the_forges_rows_in_its_decision() {
    let mut d = kept();
    let value = intent();
    restore(&mut d, value.clone());
    let mut other = row(2);
    other.key.forge = OTHER_FORGE;
    restore(&mut d, other.clone());
    let requests = outputs(
        &mut d,
        Event::Application {
            request: Token::new(101),
            token: Token::new(201),
            retention: Duration::from_secs(60),
            ask: Ask::RemoveForge { forge: FORGE },
            role: Some(jig_domain_people::Role::Owner),
            carried: None,
        },
    );
    erased(&requests, &value.key);
    assert!(d.making(&value.key).is_none());
    assert_eq!(d.making(&other.key), Some(&other));
}
#[test]
fn rows_read_in_the_forges_document_in_key_order() {
    let mut d = kept();
    let mut first = intent();
    first.key.name = b"a".as_slice().into();
    let mut last = first.clone();
    last.key.name = b"z".as_slice().into();
    restore(&mut d, last.clone());
    restore(&mut d, first.clone());
    let document = d.forges();
    assert_eq!(document.rows[0].makings.as_ref(), &[first, last]);
}

#[test]
fn copy_address_admission_uses_512_bytes_instead_of_name_bytes() {
    // The 100-byte address is longer than this deployment's 32-byte names.
    const ADDRESS: usize = 100;
    let prefix = b"https://source.example/owner/";
    let mut address = skein_lib::Writer::new(ADDRESS);
    address.put(prefix).unwrap();
    for _ in prefix.len()..ADDRESS {
        address.put(b"r").unwrap();
    }
    let address = address.finish();
    let mut d = kept();
    let mut value = intent();
    value.into = None;
    value.from = MakeFrom::Copy { address };
    let mut env = env();
    env.limits.name_bytes = 32;
    let mut out = Queue::uncapped();
    step(
        &mut d,
        &env,
        Event::Application {
            request: Token::new(101),
            token: Token::new(201),
            retention: Duration::from_secs(60),
            role: Some(jig_domain_people::Role::Owner),
            carried: None,
            ask: Ask::MakeRepository {
                forge: FORGE,
                org: value.key.org,
                name: value.key.name,
                from: value.from,
                into: None,
            },
        },
        &mut out,
    );
    answer(&d, MakeAnswer::Kept);
}
