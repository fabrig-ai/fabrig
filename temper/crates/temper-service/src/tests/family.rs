use super::fixtures;
use crate::family;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use skein_lib::{List, Reader, ReplyTo, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain as root;
use temper_engine_domain::ways;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
const REPLY: ReplyTo = ReplyTo::new(Token::new(19));
const REQUEST_KEY: [u8; 16] = [5; 16];
const WATCHER: Token = Token::new(7);
fn configured(git: &[u8]) -> Box<[crate::settings::Forge]> {
    let mut forges = fixtures::forges();
    forges.get_mut(0).unwrap().git = git.into();
    forges
}
fn adopt() -> w::Adopt {
    let mut checks = List::with_capacity(2);
    checks.push(2).unwrap();
    checks.push(3).unwrap();
    w::Adopt::new(
        &w::CEILINGS,
        w::AdoptParts {
            project: 7,
            home: true,
            forge: 1,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            prefix: b"repo".as_slice().into(),
            role: w::Role::Adopted,
            landing: b"main".as_slice().into(),
            ci: true,
            checks,
        },
    )
    .unwrap()
}
fn exchange(family: u16, version: u32, record: Box<[u8]>) -> kit::ToApplication {
    kit::ToApplication::Request {
        reply_to: REPLY,
        sign_in: 3,
        key: REQUEST_KEY,
        family,
        version,
        record,
        digest: [7; 32],
        carried: None,
        scope: people::ApplicationScope::Project(7),
    }
}
#[test]
fn forge_list_encodes_the_repository_resource_host() {
    let (limits, _, _) = fixtures::store_limits();
    let configured = forge::Configured {
        number: 3,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Managed,
        git: b"https://git.example:3000/git".as_slice().into(),
        web: Some(b"https://web.example/shop".as_slice().into()),
        owned: None,
        admin_connect: false,
        bot: Box::new([]),
        read_only: false,
        asked: false,
    };
    let domain = forge::Domain::new(
        &limits.forge,
        4,
        skein_forge_client::Config { namespace: b"test".as_slice().into(), writers: Box::new([]) },
        &[],
        &[configured],
        false,
    )
    .unwrap();
    let bytes = family::forges(domain.forges(), false, &w::CEILINGS).unwrap();
    let rows = wire::decode_forges(&bytes, &w::CEILINGS).expect("forge rows");
    assert_eq!(rows.forges().get(0).unwrap().host(), b"git.example:3000", "resource host missing from wire");
    assert_eq!(rows.forges().get(0).unwrap().host(), family::host(b"https://git.example:3000/git").as_ref());
}

#[test]
fn an_adopt_reaches_the_root_with_its_forges_host() {
    let forges = configured(b"https://forge.example:3000/git");
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&requesting())),
        &fixtures::forge_table(&forges),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application { reply_to, sign_in, key, digest, request, .. }) = event
    else {
        panic!("application request")
    };
    assert_eq!((reply_to, sign_in, key, digest), (REPLY, 3, [5; 16], [7; 32]));
    assert_eq!(
        request,
        root::Application::Forge(forge::Ask::Adopt(forge::Adoption {
            project: 7,
            home: true,
            forge: 1,
            host: b"forge.example:3000".as_slice().into(),
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            prefix: b"repo".as_slice().into(),
            role: forge::Role::Adopted,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([2, 3]),
        }))
    );
}
#[test]
fn an_adopt_naming_no_configured_forge_reaches_the_root_with_no_host() {
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&requesting())),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application { request, .. }) = event else { panic!("adoption") };
    let root::Application::Forge(forge::Ask::Adopt(request)) = request else { panic!("adoption") };
    assert!(request.host.is_empty());
    assert_eq!(request.forge, 1);
}
#[test]
fn an_adopt_of_another_version_is_answered_undecodable() {
    for (row, family, version, record, expected) in [
        ("family", 2, wire::CLIENT_VERSION, wire::requesting(&requesting()), REPLY),
        ("version", 1, wire::CLIENT_VERSION + 1, wire::requesting(&requesting()), REPLY),
        ("malformed", 1, wire::CLIENT_VERSION, Box::new([]), REPLY),
        ("trailing", 1, wire::CLIENT_VERSION, trailing_adopt(), REPLY),
    ] {
        let Err(answer) = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange(family, version, record),
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        ) else {
            panic!("{row}: undecodable")
        };
        let kit::Event::Application(kit::FromApplication::Undecodable { to }) = *answer else {
            panic!("{row}: undecodable reply")
        };
        assert_eq!(to, expected, "{row}");
    }
}
fn trailing_adopt() -> Box<[u8]> {
    let record = wire::requesting(&requesting());
    let mut bytes = skein_lib::Writer::new(record.len().checked_add(1).unwrap());
    bytes.put(&record).unwrap();
    bytes.put(&[0]).unwrap();
    bytes.finish()
}
#[test]
fn an_unconfigured_or_unknown_family_read_has_no_record() {
    for (row, family, version, record, scope, expected) in [
        (
            "forge",
            wire::FORGE,
            wire::CLIENT_VERSION,
            Box::<[u8]>::default(),
            people::ApplicationScope::Project(7),
            (REPLY, wire::FORGE, None),
        ),
        ("other", 2, 3, Box::new([]), people::ApplicationScope::Project(7), (REPLY, 2, None)),
        (
            "demo, unconfigured",
            wire::FORGE,
            wire::CLIENT_VERSION,
            wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground }).unwrap()),
            people::ApplicationScope::Deployment,
            (REPLY, wire::FORGE, None),
        ),
    ] {
        let input = kit::ToApplication::Read {
            reply: REPLY,
            sign_in: 3,
            family,
            version,
            record,
            scope,
            window: jig_domain_core::documents::Window { rows: 2, after: None },
        };
        let Err(answer) = family::to_root(
            &jig_protocol_config::Directories::working(),
            input,
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        ) else {
            panic!("{row}: unknown read")
        };
        let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *answer else {
            panic!("{row}: read")
        };
        assert_eq!((reply, family, record), expected, "{row}");
    }
}
fn read_input(scope: people::ApplicationScope) -> kit::ToApplication {
    kit::ToApplication::Read {
        reply: REPLY,
        sign_in: 3,
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        record: wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground }).unwrap()),
        scope,
        window: jig_domain_core::documents::Window { rows: 2, after: None },
    }
}
#[test]
fn a_forge_family_watch_is_refused_unknown() {
    for (row, family, version, expected) in [
        ("forge", wire::FORGE, wire::CLIENT_VERSION, (Token::new(7), jig_domain_core::WatchRefusal::Unknown)),
        ("other", 2, 3, (Token::new(7), jig_domain_core::WatchRefusal::Unknown)),
    ] {
        let input = kit::ToApplication::Watch {
            watcher: WATCHER,
            sign_in: 3,
            family,
            version,
            record: Box::new([]),
            scope: people::ApplicationScope::Project(7),
            window: jig_domain_core::documents::Window { rows: 2, after: None },
        };
        let Err(answer) = family::to_root(
            &jig_protocol_config::Directories::working(),
            input,
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        ) else {
            panic!("{row}: refused watch")
        };
        let kit::Event::WatchRefused { watcher, why } = *answer else { panic!("{row}: refusal") };
        assert_eq!((watcher, why), expected, "{row}");
    }
}
#[test]
fn a_git_base_with_a_port_keeps_it_in_the_host() {
    for (git, expected) in [
        (b"https://forge.example/git".as_slice(), b"forge.example".as_slice()),
        (b"http://127.0.0.1:3000/".as_slice(), b"127.0.0.1:3000".as_slice()),
        (b"https://[::1]:3000/git".as_slice(), b"[::1]:3000".as_slice()),
        (b"https://forge.example".as_slice(), b"forge.example".as_slice()),
        (b"https://user@forge.example:3000/git".as_slice(), b"forge.example:3000".as_slice()),
    ] {
        assert_eq!(family::host(git).as_ref(), expected, "{git:?}");
    }
}
fn encoded(outcome: Option<forge::ApplicationOutcome>) -> w::Outcome {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    let outcome = match outcome {
        Some(outcome) => Some(temper_engine_domain::ApplicationAnswer::Adopt(outcome)),
        None => None,
    };
    let bytes = family::answer(outcome, &w::CEILINGS, &jig).unwrap();
    let mut reader = Reader::new(&bytes);
    let answer = w::Answer::decode(&w::CEILINGS, &mut reader).unwrap();
    assert!(reader.is_empty());
    answer.into_parts().outcome
}
#[test]
fn each_outcome_is_encoded_as_the_forge_familys_answer() {
    for (row, input, expected) in [
        ("no forge", forge::AdoptionRefusal::NoSuchForge, w::ForgeReason::NoSuchForge),
        ("missing", forge::AdoptionRefusal::NotFound, w::ForgeReason::NotFound),
        ("access", forge::AdoptionRefusal::NoAccess, w::ForgeReason::NoAccess),
        ("refused", forge::AdoptionRefusal::Refused, w::ForgeReason::Refused),
        ("in flight", forge::AdoptionRefusal::InFlight, w::ForgeReason::InFlight),
        ("too large", forge::AdoptionRefusal::TooLarge, w::ForgeReason::TooLarge),
        ("unreachable", forge::AdoptionRefusal::Unreachable, w::ForgeReason::Unreachable),
        ("invalid", forge::AdoptionRefusal::InvalidAnswer, w::ForgeReason::InvalidAnswer),
        ("another project", forge::AdoptionRefusal::AnotherProject, w::ForgeReason::AnotherProject),
        ("home held", forge::AdoptionRefusal::HomeHeld, w::ForgeReason::HomeHeld),
        ("prefix occupied", forge::AdoptionRefusal::PrefixOccupied, w::ForgeReason::PrefixOccupied),
    ] {
        let w::Outcome::ForgeRefused(answer) = encoded(Some(forge::ApplicationOutcome::ForgeRefused(input))) else {
            panic!("{row}: forge refusal")
        };
        assert_eq!(answer.into_parts().reason, expected, "{row}");
    }
}
#[test]
fn an_outcome_past_its_retention_is_encoded_unknown() {
    assert_eq!(encoded(None), w::Outcome::Unknown);
}
#[test]
fn a_core_refusal_uses_jigs_refusal_record() {
    use jig_client_protocol_wire::v19 as j;
    let limits = fixtures::engine().jig.limits.clients.wire;
    for (row, input, expected) in [
        ("unknown", people::Refusal::Unknown, j::Refusal::Unknown),
        ("name taken", people::Refusal::NameTaken, j::Refusal::NameTaken),
        ("not offered", people::Refusal::NotOffered, j::Refusal::NotOffered),
        (
            "taken",
            people::Refusal::Taken { by: 7 },
            j::Refusal::Taken(j::RefusalTaken::new(&limits, j::RefusalTakenParts { by: 7 }).unwrap()),
        ),
        (
            "authority",
            people::Refusal::Authority { lacking: people::Lack::Writer, proposable: Some(people::Accepter::Role(1)) },
            j::Refusal::Authority(
                j::RefusalAuthority::new(
                    &limits,
                    j::RefusalAuthorityParts { lacking: j::Lacking::Writer, proposable: Some(true) },
                )
                .unwrap(),
            ),
        ),
    ] {
        let w::Outcome::CoreRefused(record) = encoded(Some(forge::ApplicationOutcome::CoreRefused(input))) else {
            panic!("{row}: core refusal")
        };
        let mut reader = Reader::new(record.refusal());
        assert_eq!(j::Refusal::decode(&limits, &mut reader).unwrap(), expected, "{row}");
        assert!(reader.is_empty(), "{row}");
    }
}
fn adopted(protection: forge::Protection) -> forge::ApplicationOutcome {
    forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
        repository: skein_forge::api::Repository { forge: 1, repository: 42 },
        owner: b"acme".as_slice().into(),
        name: b"shop".as_slice().into(),
        role: forge::Role::Fork,
        home: true,
        kinds: forge::Kinds {
            read: true,
            push: false,
            open: true,
            land: false,
            review: true,
            status: false,
            comment: true,
            issue: false,
            branch: true,
        },
        protection,
    })
}
#[test]
fn an_adopted_repository_keeps_its_facts_in_the_answer() {
    let w::Outcome::Adopted(record) = encoded(Some(adopted(forge::Protection::Absent))) else { panic!("adopted") };
    let a = record.into_parts();
    assert_eq!((a.forge, a.repository, a.home, a.role), (1, 42, true, w::Role::Fork));
    assert_eq!((a.owner.as_ref(), a.name.as_ref()), (b"acme".as_slice(), b"shop".as_slice()));
    let k = a.kinds;
    assert_eq!(
        [k.read(), k.push(), k.open(), k.land(), k.review(), k.status(), k.comment(), k.issue(), k.branch()],
        [true, false, true, false, true, false, true, false, true]
    );
    assert_eq!(a.protection, w::Protection::Absent);
}
#[test]
fn an_adopted_repository_keeps_its_protection_rule() {
    let outcome = adopted(forge::Protection::Rule(skein_forge::api::Protection {
        queue: false,
        signatures: false,
        linear: false,
        code_owners: false,
        bypass: skein_forge::api::Bypass::Never,

        branch: b"main".as_slice().into(),
        contexts: Box::new([b"ci/build".as_slice().into()]),
        approvals: 2,
        dismiss_stale: true,
    }));
    let w::Outcome::Adopted(record) = encoded(Some(outcome)) else { panic!("adopted") };
    let w::Protection::Rule(rule) = record.into_parts().protection else { panic!("rule") };
    let rule = rule.into_parts();
    assert_eq!(rule.branch.as_ref(), b"main");
    assert_eq!(rule.contexts.as_slice(), &[b"ci/build".as_slice().into()]);
    assert_eq!((rule.approvals, rule.dismiss_stale), (2, true));
}

#[test]
fn a_request_under_an_owner_without_records_is_answered_undecodable() {
    for owner in [wire::Owner::Pages, wire::Owner::Runner] {
        let number = wire::number(owner);
        let Err(answer) = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange(number, wire::CLIENT_VERSION, wire::requesting(&requesting())),
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        ) else {
            panic!("owner is not served: {owner:?}")
        };
        let kit::Event::Application(kit::FromApplication::Undecodable { to }) = *answer else {
            panic!("undecodable answer: {owner:?}")
        };
        assert_eq!(to, REPLY, "{owner:?}");
    }
}

const NOT_OFFERED_RECORD: &[u8] = &[10];
const NO_ESCALATION_RECORD: &[u8] = &[12];
const PROPOSABLE_WRITER_RECORD: &[u8] = &[5, 12, 1, 1];
const MALFORMED_POSITION: u32 = 3;
const MALFORMED_RECORD: &[u8] = &[0, 0, 0, 0, 3, 3];
fn refusal_bytes(refusal: people::Refusal) -> Box<[u8]> {
    let w::Outcome::CoreRefused(record) = encoded(Some(forge::ApplicationOutcome::CoreRefused(refusal))) else {
        panic!("core refusal")
    };
    record.into_parts().refusal
}
#[test]
fn a_core_refusal_is_carried_as_jigs_refusal_record() {
    assert_eq!(refusal_bytes(people::Refusal::NotOffered).as_ref(), NOT_OFFERED_RECORD);
}
#[test]
fn a_refusal_naming_no_one_to_escalate_to_is_carried_with_its_facts() {
    assert_eq!(refusal_bytes(people::Refusal::NoEscalationHolder).as_ref(), NO_ESCALATION_RECORD);
}
#[test]
fn an_authority_refusal_keeps_whom_a_proposal_could_go_to() {
    let refusal =
        people::Refusal::Authority { lacking: people::Lack::Writer, proposable: Some(people::Accepter::Role(1)) };
    assert_eq!(refusal_bytes(refusal).as_ref(), PROPOSABLE_WRITER_RECORD);
}
#[test]
fn a_malformed_refusal_encodes_its_fields() {
    let refusal = people::Refusal::NotWellFormed { at: MALFORMED_POSITION, why: people::Malformed::Cycle };
    assert_eq!(refusal_bytes(refusal).as_ref(), MALFORMED_RECORD);
}
#[test]
fn a_family_request_keeps_its_supplied_admission_scope() {
    let mut input = exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&requesting()));
    let kit::ToApplication::Request { scope, .. } = &mut input else { panic!("request") };
    *scope = people::ApplicationScope::Deployment;
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        input,
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application { scope, .. }) = event else { panic!("application") };
    assert_eq!(scope, people::ApplicationScope::Deployment);
}

#[test]
fn not_installed_projects_to_no_access_until_temper_07_adds_its_wire_reason() {
    let w::Outcome::ForgeRefused(answer) =
        encoded(Some(forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::NotInstalled)))
    else {
        panic!("forge refusal")
    };
    assert_eq!(answer.reason(), &w::ForgeReason::NoAccess);
}

const CHANGE_TASK: u64 = 51;
fn watching(task: u64) -> Box<[u8]> {
    wire::watching(
        &w::Watching::new(
            &w::CEILINGS,
            w::WatchingParts {
                watch: w::Watch::Change(w::ChangeSubject::new(&w::CEILINGS, w::ChangeSubjectParts { task }).unwrap()),
            },
        )
        .unwrap(),
    )
}
fn watch_exchange(family: u16, version: u32, record: Box<[u8]>) -> kit::ToApplication {
    kit::ToApplication::Watch {
        watcher: WATCHER,
        sign_in: 3,
        family,
        version,
        record,
        scope: people::ApplicationScope::Project(7),
        window: jig_domain_core::documents::Window { rows: 2, after: None },
    }
}
#[test]
fn a_change_watch_reaches_the_root_with_its_scope_subject_and_window() {
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        watch_exchange(wire::FORGE, wire::CLIENT_VERSION, watching(CHANGE_TASK)),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Watch { watcher, sign_in, subject, window }) = event else {
        panic!("root watch")
    };
    assert_eq!((watcher, sign_in), (WATCHER, 3));
    assert_eq!(
        subject,
        jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(7),
            connector: wire::FORGE,
            subject: Token::new(CHANGE_TASK),
        }
    );
    assert_eq!(window, jig_domain_core::documents::Window { rows: 2, after: None });
}
#[test]
fn invalid_change_watches_are_refused_unknown_without_the_root() {
    let record = watching(CHANGE_TASK);
    let mut trailing = skein_lib::Writer::new(record.len().checked_add(1).unwrap());
    trailing.put(&record).unwrap();
    trailing.put(&[0]).unwrap();
    for (row, family, version, record) in [
        ("family", wire::number(wire::Owner::Pages), wire::CLIENT_VERSION, watching(CHANGE_TASK)),
        ("version", wire::FORGE, wire::CLIENT_VERSION + 1, watching(CHANGE_TASK)),
        ("malformed", wire::FORGE, wire::CLIENT_VERSION, Box::new([])),
        ("trailing", wire::FORGE, wire::CLIENT_VERSION, trailing.finish()),
        ("zero task", wire::FORGE, wire::CLIENT_VERSION, watching(0)),
    ] {
        let Err(answer) = family::to_root(
            &jig_protocol_config::Directories::working(),
            watch_exchange(family, version, record),
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        ) else {
            panic!("{row}: refused watch")
        };
        let kit::Event::WatchRefused { watcher, why } = *answer else { panic!("{row}: unknown") };
        assert_eq!((watcher, why), (WATCHER, jig_domain_core::WatchRefusal::Unknown), "{row}");
    }
}
fn panel() -> forge::Panel {
    forge::Panel {
        way: change::Way::Queue,
        in_landing: None,
        task: CHANGE_TASK,
        forge: 1,
        owner: b"acme".as_slice().into(),
        name: b"shop".as_slice().into(),
        branch: b"temper/51".as_slice().into(),
        landing: b"main".as_slice().into(),
        head: Some([4; 32]),
        pull: Some(123),
        step: forge::PanelStep::Queued { place: 3, ahead: Box::new([49, 47]) },
        hold: Some(change::Hold::Updates),
        runs: Some(81),
        ci_required: true,
        ci: change::Status::Pending,
        approvals: Box::new([]),
        unmapped_reviews: Box::new([]),
        gates: Box::new([
            forge::PanelGate {
                number: 1,
                kind: change::GateKind::Agent,
                blocking: true,
                verdict: change::Status::Passed,
            },
            forge::PanelGate {
                number: 2,
                kind: change::GateKind::Person,
                blocking: true,
                verdict: change::Status::Failed,
            },
            forge::PanelGate {
                number: 3,
                kind: change::GateKind::Check,
                blocking: false,
                verdict: change::Status::Pending,
            },
            forge::PanelGate {
                number: 4,
                kind: change::GateKind::Check,
                blocking: true,
                verdict: change::Status::Unknown,
            },
        ]),
        repairs: forge::PanelUsed { used: 1, limit: 3 },
        resolutions: forge::PanelUsed { used: 2, limit: 4 },
        updates: forge::PanelUsed { used: 3, limit: 5 },
        questions: forge::PanelQuestions {
            route: temper_engine_domain_forge_change::Route::None,
            taken: forge::PanelUsed { used: 0, limit: 4 },
            open: Box::new([]),
        },
    }
}
fn decoded_panel(value: forge::Panel, forges: &crate::forges::Forges) -> w::ChangePanel {
    let bytes = family::panel(value, forges, &w::CEILINGS).unwrap();
    wire::decode_change_panel(&bytes, &w::CEILINGS).expect("whole panel at CLIENT_VERSION")
}
fn queued(place: u32, tasks: &[u64]) -> w::Step {
    let mut ahead = List::with_capacity(wire::TASKS_AHEAD);
    for task in tasks {
        ahead.push(*task).unwrap();
    }
    w::Step::Queued(w::Queued::new(&w::CEILINGS, w::QueuedParts { place, ahead }).unwrap())
}
#[test]
fn a_panel_encodes_and_decodes_back_whole() {
    let mut gates = List::with_capacity(wire::GATES);
    for (number, kind, blocks, verdict) in [
        (1, w::GateKind::Review, true, w::Verdict::Approved),
        (2, w::GateKind::Approval, true, w::Verdict::Changes),
        (3, w::GateKind::Check, false, w::Verdict::Pending),
        (4, w::GateKind::Check, true, w::Verdict::None),
    ] {
        gates.push(w::Gate::new(&w::CEILINGS, w::GateParts { number, kind, blocks, verdict }).unwrap()).unwrap();
    }
    let expected = w::ChangePanel::new(
        &w::CEILINGS,
        w::ChangePanelParts {
            way: w::Way::Queue,
            in_landing: None,
            task: CHANGE_TASK,
            forge: 1,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            branch: b"temper/51".as_slice().into(),
            landing: b"main".as_slice().into(),
            head: Some([4; 32]),
            pull: Some(w::Pull::new(&w::CEILINGS, w::PullParts { number: 123, address: Box::new([]) }).unwrap()),
            step: queued(3, &[49, 47]),
            hold: Some(w::Hold::Updates),
            runs: Some(81),
            ci_required: true,
            ci: w::Ci::Pending,
            gates,
            approvals: List::with_capacity(0),
            unmapped_reviews: List::with_capacity(0),
            repairs: w::Used::new(&w::CEILINGS, w::UsedParts { used: 1, limit: 3 }).unwrap(),
            resolutions: w::Used::new(&w::CEILINGS, w::UsedParts { used: 2, limit: 4 }).unwrap(),
            updates: w::Used::new(&w::CEILINGS, w::UsedParts { used: 3, limit: 5 }).unwrap(),
            questions: w::Questions::new(
                &w::CEILINGS,
                w::QuestionsParts {
                    route: w::Route::None,
                    taken: w::Used::new(&w::CEILINGS, w::UsedParts { used: 0, limit: 4 }).expect("question limit"),
                    open: List::with_capacity(0),
                },
            )
            .expect("no questions"),
        },
    )
    .unwrap();
    assert_eq!(decoded_panel(panel(), &fixtures::forge_table(&[])), expected);
}
fn repairing(reason: w::Reason) -> w::Step {
    w::Step::Repairing(w::Repairing::new(&w::CEILINGS, w::RepairingParts { reason }).unwrap())
}
#[test]
fn every_panel_step_keeps_its_payload_on_the_wire() {
    for (input, expected) in [
        (forge::PanelStep::Ended, w::Step::Ended),
        (
            forge::PanelStep::Ready { landing: Some(61) },
            w::Step::Ready(w::InLanding::new(&w::CEILINGS, w::InLandingParts { landing: Some(61) }).unwrap()),
        ),
        (
            forge::PanelStep::Ready { landing: None },
            w::Step::Ready(w::InLanding::new(&w::CEILINGS, w::InLandingParts { landing: None }).unwrap()),
        ),
        (
            forge::PanelStep::Preparing { landing: Some(61) },
            w::Step::Preparing(w::InLanding::new(&w::CEILINGS, w::InLandingParts { landing: Some(61) }).unwrap()),
        ),
        (forge::PanelStep::Producing, w::Step::Producing),
        (forge::PanelStep::Opening, w::Step::Opening),
        (forge::PanelStep::Recreating, w::Step::Recreating),
        (forge::PanelStep::Reopening, w::Step::Reopening),
        (forge::PanelStep::Checking, w::Step::Checking),
        (forge::PanelStep::Gating, w::Step::Gating),
        (forge::PanelStep::Queued { place: 3, ahead: Box::new([49, 47]) }, queued(3, &[49, 47])),
        (forge::PanelStep::First, w::Step::First),
        (forge::PanelStep::Updating, w::Step::Updating),
        (forge::PanelStep::Resolving, w::Step::Resolving),
        (forge::PanelStep::Repairing(change::Repair::Ci), repairing(w::Reason::Ci)),
        (
            forge::PanelStep::Repairing(change::Repair::Gate(9)),
            repairing(w::Reason::Gate(w::GateReason::new(&w::CEILINGS, w::GateReasonParts { gate: 9 }).unwrap())),
        ),
        (forge::PanelStep::Repairing(change::Repair::Semantic), repairing(w::Reason::Semantic)),
        (forge::PanelStep::Repairing(change::Repair::Review { head: [7; 32] }), repairing(w::Reason::Review)),
        (forge::PanelStep::Landing, w::Step::Landing),
        (
            forge::PanelStep::Landed { merge: [8; 32] },
            w::Step::Landed(w::Landed::new(&w::CEILINGS, w::LandedParts { merge: [8; 32] }).unwrap()),
        ),
    ] {
        let mut value = panel();
        for (way, expected_way) in [(change::Way::Queue, w::Way::Queue), (change::Way::Turn, w::Way::Turn)] {
            for landing in [None, Some(61)] {
                value.step = input.clone();
                value.way = way;
                value.in_landing = landing;
                let got = decoded_panel(value.clone(), &fixtures::forge_table(&[]));
                assert_eq!(*got.step(), expected);
                assert_eq!(*got.way(), expected_way);
                assert_eq!(*got.in_landing(), landing);
            }
        }
    }
}
#[test]
fn every_hold_and_ci_verdict_keeps_its_meaning_on_the_wire() {
    for (input, expected) in [
        (None, None),
        (Some(change::Hold::Stalled), Some(w::Hold::Stalled)),
        (Some(change::Hold::Repairs), Some(w::Hold::Repairs)),
        (Some(change::Hold::Questions), Some(w::Hold::Questions)),
        (Some(change::Hold::Resolutions), Some(w::Hold::Resolutions)),
        (Some(change::Hold::Updates), Some(w::Hold::Updates)),
        (Some(change::Hold::BranchMoved), Some(w::Hold::Moved)),
        (Some(change::Hold::BranchMissing), Some(w::Hold::Missing)),
        (Some(change::Hold::PullClosed), Some(w::Hold::Closed)),
        (Some(change::Hold::Retargeted), Some(w::Hold::Retargeted)),
        (Some(change::Hold::Failed), Some(w::Hold::Failed)),
        (Some(change::Hold::Rejected), Some(w::Hold::Rejected)),
    ] {
        let mut value = panel();
        value.hold = input;
        assert_eq!(*decoded_panel(value, &fixtures::forge_table(&[])).hold(), expected);
    }
    for (input, expected) in [
        (change::Status::Unknown, w::Ci::Unknown),
        (change::Status::Pending, w::Ci::Pending),
        (change::Status::Passed, w::Ci::Passed),
        (change::Status::Failed, w::Ci::Failed),
    ] {
        let value = panel();
        for required in [false, true] {
            let mut value = value.clone();
            value.ci = input;
            value.ci_required = required;
            let decoded = decoded_panel(value, &fixtures::forge_table(&[]));
            assert_eq!(*decoded.ci(), expected);
            assert_eq!(decoded.ci_required(), required);
        }
    }
}
#[test]
fn a_panels_pull_address_uses_its_configured_forge_kind_and_base() {
    use crate::settings::Kind;
    for (row, kind, git, web, number, expected) in [
        (
            "github",
            Kind::GitHub,
            b"https://git.example".as_slice(),
            Some(b"https://github.example".as_slice()),
            1,
            b"https://github.example/acme/shop/pull/123".as_slice(),
        ),
        (
            "github trailing slash",
            Kind::GitHub,
            b"https://git.example".as_slice(),
            Some(b"https://github.example/".as_slice()),
            1,
            b"https://github.example/acme/shop/pull/123".as_slice(),
        ),
        (
            "forgejo",
            Kind::Forgejo,
            b"https://forge.example/base".as_slice(),
            None,
            1,
            b"https://forge.example/base/acme/shop/pulls/123".as_slice(),
        ),
        (
            "forgejo trailing slash",
            Kind::Forgejo,
            b"https://forge.example/base/".as_slice(),
            None,
            1,
            b"https://forge.example/base/acme/shop/pulls/123".as_slice(),
        ),
        ("no web", Kind::GitHub, b"".as_slice(), None, 1, b"".as_slice()),
        ("no git", Kind::Forgejo, b"".as_slice(), None, 1, b"".as_slice()),
        ("another forge", Kind::Forgejo, b"https://other.example".as_slice(), None, 2, b"".as_slice()),
    ] {
        let mut forges = fixtures::forges();
        let forge = forges.get_mut(0).unwrap();
        forge.kind = kind;
        forge.number = number;
        forge.git = git.into();
        forge.web = match web {
            Some(web) => Some(web.into()),
            None => None,
        };
        let got = decoded_panel(panel(), &fixtures::forge_table(&forges)).into_parts().pull.unwrap();
        assert_eq!(got.number(), 123, "{row}");
        assert_eq!(got.address(), expected, "{row}");
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test spells out the expected address")]
fn a_panels_pull_address_past_its_bound_is_left_out_and_the_panel_still_delivered() {
    use crate::settings::Kind;
    let web = b"https://forge.example".as_slice();
    let owner = core::iter::repeat_n(b'o', 1024).collect::<Box<[u8]>>();
    for (kind, route) in [(Kind::GitHub, b"/pull/".as_slice()), (Kind::Forgejo, b"/pulls/".as_slice())] {
        // The longest name whose address, BASE/OWNER/NAME + route + "123", fills
        // ADDRESS_BYTES exactly; one byte more passes the bound.
        let fits = usize::try_from(wire::ADDRESS_BYTES).unwrap() - (web.len() + 1 + owner.len() + 1 + route.len() + 3);
        for (row, length, shown) in [("at the bound", fits, true), ("past the bound", fits + 1, false)] {
            let name = core::iter::repeat_n(b'n', length).collect::<Box<[u8]>>();
            let mut forges = fixtures::forges();
            let forge = forges.get_mut(0).unwrap();
            forge.kind = kind;
            forge.git = web.into();
            forge.web = match kind {
                Kind::GitHub => Some(web.into()),
                Kind::Forgejo => None,
            };
            let mut value = panel();
            value.owner = owner.clone();
            value.name = name.clone();
            let decoded = decoded_panel(value, &fixtures::forge_table(&forges));
            assert_eq!(decoded.owner(), &*owner, "{row}");
            assert_eq!(decoded.name(), &*name, "{row}");
            let pull = decoded.into_parts().pull.unwrap();
            assert_eq!(pull.number(), 123, "{row}");
            if shown {
                assert_eq!(pull.address(), [web, b"/", &owner, b"/", &name, route, b"123"].concat(), "{row}");
            } else {
                assert_eq!(pull.address(), b"", "{row}");
            }
        }
    }
}
#[test]
fn a_panel_before_the_first_head_has_no_pull_or_running_delegate() {
    let mut value = panel();
    value.head = None;
    value.pull = None;
    value.runs = None;
    value.hold = None;
    value.step = forge::PanelStep::Producing;
    value.ci = change::Status::Unknown;
    value.gates = Box::new([]);
    let got = decoded_panel(value, &fixtures::forge_table(&fixtures::forges())).into_parts();
    assert_eq!((got.head, got.pull, got.runs, got.hold), (None, None, None, None));
    assert_eq!(got.step, w::Step::Producing);
    assert_eq!(got.ci, w::Ci::Unknown);
    assert!(got.gates.is_empty());
}

fn requesting() -> w::Requesting {
    w::Requesting::new(&w::CEILINGS, w::RequestingParts { request: w::Request::Adopt(adopt()) }).unwrap()
}

#[test]
fn setup_requests_preserve_the_keyed_envelope_and_forge() {
    let request = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::SetUp(
                w::SetUp::new(
                    &w::CEILINGS,
                    w::SetUpParts {
                        forge: w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge: 3 }).unwrap()),
                        first: w::SetUpFirst::Empty(
                            w::FirstEmpty::new(
                                &w::CEILINGS,
                                w::FirstEmptyParts {
                                    playground: false,
                                    org: b"work".as_slice().into(),
                                    repository: b"hello".as_slice().into(),
                                },
                            )
                            .expect("setup names fit wire ceilings"),
                        ),
                    },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap();
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&request)),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application { reply_to, sign_in, key, digest, request, .. }) = event
    else {
        panic!("setup application")
    };
    assert_eq!(
        (reply_to, sign_in, key, digest, request),
        (
            REPLY,
            3,
            [5; 16],
            [7; 32],
            root::Application::Forge(forge::Ask::SetUp {
                forge: forge::Making::Kept(3),
                first: forge::SetUpFirst::Empty {
                    playground: false,
                    org: b"work".as_slice().into(),
                    repository: b"hello".as_slice().into()
                }
            })
        )
    );
}

#[test]
fn forges_watches_name_the_deployment_subject() {
    let watching = w::Watching::new(&w::CEILINGS, w::WatchingParts { watch: w::Watch::Forges }).unwrap();
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        watch_exchange(wire::FORGE, wire::CLIENT_VERSION, wire::watching(&watching)),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Watch { watcher, subject, .. }) = event else { panic!("forges watch") };
    assert_eq!(
        (watcher, subject),
        (
            WATCHER,
            jig_domain_core::documents::Watched::Application {
                scope: people::ApplicationScope::Deployment,
                connector: wire::FORGE,
                subject: Token::new(0),
            }
        )
    );
}

#[test]
fn setup_outcomes_use_the_family_answer_including_unknown() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    for (outcome, expected) in [
        (None, w::Outcome::Unknown),
        (Some(forge::SetUpAnswer::SettingUp), w::Outcome::SettingUp),
        (
            Some(forge::SetUpAnswer::Refused(forge::SetUpRefusal::NotOffered)),
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::NotOffered }).unwrap(),
            ),
        ),
        (
            Some(forge::SetUpAnswer::Refused(forge::SetUpRefusal::NotUnderTheBox)),
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::NotUnderTheBox })
                    .unwrap(),
            ),
        ),
        (
            Some(forge::SetUpAnswer::Refused(forge::SetUpRefusal::Busy)),
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
            ),
        ),
    ] {
        let outcome = match outcome {
            Some(outcome) => Some(temper_engine_domain::ApplicationAnswer::SetUp(outcome)),
            None => None,
        };
        let bytes = family::answer(outcome, &w::CEILINGS, &jig).unwrap();
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome, expected);
    }
}
fn forge_row(
    number: u16,
    kind: forge::Kind,
    mode: forge::Mode,
    connection: forge::Connection,
    offer: forge::Offer,
) -> forge::Row {
    forge::Row {
        starts_with: forge::StartsWith::Repository,
        makings: Box::new([]),
        host: b"git.example:3000".as_slice().into(),
        number,
        kind,
        mode,
        connection,
        failed: None,
        fetched: None,
        offer,
        first: None,
        installations: Box::new([]),
        ended: None,
        app: None,
        web: None,
        repositories: Box::new([]),
        owned: None,
        admin: false,
        read_only: false,
        ended_stop: None,
    }
}
fn configured_forge_rows() -> forge::Forges {
    forge::Forges {
        rows: Box::new([
            forge::Row {
                starts_with: forge::StartsWith::Repository,
                makings: Box::new([]),
                host: b"git.example:3000".as_slice().into(),
                number: 1,
                kind: forge::Kind::Forgejo,
                mode: forge::Mode::Owned,
                connection: forge::Connection::SettingUp { step: Some(forge::SetUpStep::Prepare) },
                failed: None,
                fetched: None,
                offer: forge::Offer::Offered,
                first: None,
                installations: Box::new([]),
                ended: None,
                app: None,
                web: None,
                repositories: Box::new([]),
                owned: Some(forge::Owned {
                    forgejo: b"17.0.0".as_slice().into(),
                    runner: Some(b"13.2.0".as_slice().into()),
                    password: b"/fake/forge/person_password".as_slice().into(),
                }),
                admin: false,
                read_only: false,
                ended_stop: None,
            },
            forge_row(
                2,
                forge::Kind::GitHub,
                forge::Mode::Connected,
                forge::Connection::NotConnected,
                forge::Offer::NotOffered,
            ),
            forge_row(
                3,
                forge::Kind::Forgejo,
                forge::Mode::Managed,
                forge::Connection::NotSetUp,
                forge::Offer::NotUnderTheBox,
            ),
            forge_row(
                4,
                forge::Kind::Forgejo,
                forge::Mode::Fake,
                forge::Connection::Connecting,
                forge::Offer::NotOffered,
            ),
        ]),
    }
}
#[test]
fn a_row_s_fetched_reaches_the_wire() {
    const OWNED: u16 = 1;
    for (binary, expected_binary, total) in [
        (forge::SetUpBinary::Forgejo, w::SetUpBinary::Forgejo, Some(100)),
        (forge::SetUpBinary::Runner, w::SetUpBinary::Runner, None),
        (forge::SetUpBinary::Forgejo, w::SetUpBinary::Forgejo, None),
        (forge::SetUpBinary::Runner, w::SetUpBinary::Runner, Some(100)),
    ] {
        let mut row = forge_row(
            OWNED,
            forge::Kind::Forgejo,
            forge::Mode::Owned,
            forge::Connection::SettingUp { step: Some(forge::SetUpStep::Fetch) },
            forge::Offer::Offered,
        );
        row.fetched = Some(forge::SetUpFetched { binary, done: 10, total });
        let bytes = family::forges(forge::Forges { rows: Box::new([row]) }, false, &w::CEILINGS).unwrap();
        let rows = wire::decode_forges(&bytes, &w::CEILINGS).unwrap();
        let w::Connection::SettingUp(setup) = rows.forges().get(0).unwrap().connection() else { panic!("setting up") };
        assert_eq!(setup.step(), &Some(w::SetUpStep::Fetch));
        assert_eq!(
            setup.fetched(),
            &Some(
                w::SetUpFetched::new(&w::CEILINGS, w::SetUpFetchedParts { binary: expected_binary, done: 10, total })
                    .unwrap()
            )
        );
    }
}

#[test]
fn forges_encode_the_domains_rows_without_settings() {
    let value = configured_forge_rows();
    for (failed, expected) in [
        (None, None),
        (Some(forge::FailedStep::Fetch), Some(w::FailedStep::Fetch)),
        (Some(forge::FailedStep::Prepare), Some(w::FailedStep::Prepare)),
        (Some(forge::FailedStep::Run), Some(w::FailedStep::Run)),
        (Some(forge::FailedStep::Connect), Some(w::FailedStep::Connect)),
    ] {
        let mut failure = value.clone();
        failure.rows[0].connection = forge::Connection::SetUpFailed { why: b"stopped".as_slice().into() };
        failure.rows[0].failed = failed;
        let bytes = family::forges(failure, false, &w::CEILINGS).unwrap();
        let rows = wire::decode_forges(&bytes, &w::CEILINGS).unwrap().into_parts().forges;
        let w::Connection::SetUpFailed(failure) = rows.get(0).unwrap().connection() else { panic!("setup failure") };
        assert_eq!(failure.at(), &expected);
        assert_eq!(failure.why(), b"stopped");
    }
    let bytes = family::forges(value, false, &w::CEILINGS).unwrap();
    let rows = temper_client_protocol_wire::decode_forges(&bytes, &w::CEILINGS).unwrap().into_parts().forges;
    let expected = [
        (
            1,
            w::Kind::Forgejo,
            w::Mode::Owned,
            w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Prepare), fetched: None })
                    .unwrap(),
            ),
            w::Offer::Offered,
        ),
        (2, w::Kind::Github, w::Mode::Connected, w::Connection::NotConnected, w::Offer::NotOffered),
        (3, w::Kind::Forgejo, w::Mode::Managed, w::Connection::NotSetUp, w::Offer::NotUnderTheBox),
        (4, w::Kind::Forgejo, w::Mode::Fake, w::Connection::Connecting, w::Offer::NotOffered),
    ];
    assert_eq!(rows.len(), u32::try_from(expected.len()).unwrap());
    for (row, (number, kind, mode, connection, offer)) in rows.into_iter().zip(expected) {
        assert_eq!(row, &expected_forge_row(number, kind, mode, connection, offer));
    }
}

fn expected_forge_row(
    number: u16,
    kind: w::Kind,
    mode: w::Mode,
    connection: w::Connection,
    offer: w::Offer,
) -> w::ForgeRow {
    w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            starts_with: w::StartsWith::Repository,
            makings: List::with_capacity(0),
            host: b"git.example:3000".as_slice().into(),
            number,
            kind,
            mode,
            connection,
            offer,
            first: None,
            installations: List::with_capacity(0),
            ended: None,
            app: None,
            web: None,
            repositories: List::with_capacity(0),
            owned: if number == 1 {
                Some(
                    w::OwnedForge::new(
                        &w::CEILINGS,
                        w::OwnedForgeParts {
                            forgejo: b"17.0.0".as_slice().into(),
                            runner: Some(b"13.2.0".as_slice().into()),
                            password: b"/fake/forge/person_password".as_slice().into(),
                        },
                    )
                    .unwrap(),
                )
            } else {
                None
            },
            admin: false,
            read_only: false,
            ended_stop: None,
        },
    )
    .unwrap()
}

#[test]
fn an_exceeded_hold_refusal_keeps_each_bound_and_its_configured_value() {
    use jig_client_protocol_wire::v19 as j;
    for (bound, expected) in [
        (people::Bound::Reasoning, j::Bound::Reasoning),
        (people::Bound::Metadata, j::Bound::Metadata),
        (people::Bound::Items, j::Bound::Items),
        (people::Bound::Answer, j::Bound::Answer),
        (people::Bound::Tools, j::Bound::Tools),
        (people::Bound::HistoryItems, j::Bound::HistoryItems),
        (people::Bound::History, j::Bound::History),
        (people::Bound::Window, j::Bound::Window),
        (people::Bound::Output, j::Bound::Output),
    ] {
        let bytes = refusal_bytes(people::Refusal::Moved {
            now: people::Moved::Hold {
                revision: 7,
                reason: people::HoldReason::Exceeded(people::Exceeded { bound, value: u64::MAX }),
            },
        });
        let mut reader = Reader::new(&bytes);
        let j::Refusal::Moved(moved) = j::Refusal::decode(&j::CEILINGS, &mut reader).expect("jig refusal") else {
            panic!("moved")
        };
        let j::Moved::Hold(hold) = moved.what() else { panic!("hold") };
        let Some(j::CurrentHold::Reason(reason)) = hold.reason() else { panic!("hold reason") };
        let j::HoldCategory::Exceeded(exceeded) = reason.value() else { panic!("exceeded") };
        assert_eq!(exceeded.bound(), &expected);
        assert_eq!(exceeded.value(), u64::MAX);
        assert_eq!(moved.now().value(), 7);
        assert!(reader.is_empty());
    }
}

#[test]
fn setup_and_first_repository_states_translate_without_losing_fields() {
    let record = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::SetUp(
                w::SetUp::new(
                    &w::CEILINGS,
                    w::SetUpParts {
                        forge: w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge: 3 }).unwrap()),
                        first: w::SetUpFirst::Empty(
                            w::FirstEmpty::new(
                                &w::CEILINGS,
                                w::FirstEmptyParts {
                                    playground: false,
                                    org: b"team".as_slice().into(),
                                    repository: b"widget".as_slice().into(),
                                },
                            )
                            .expect("setup names fit wire ceilings"),
                        ),
                    },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap();
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&record)),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application { request, .. }) = event else { panic!("application") };
    assert_eq!(
        request,
        root::Application::Forge(forge::Ask::SetUp {
            forge: forge::Making::Kept(3),
            first: forge::SetUpFirst::Empty {
                playground: false,
                org: b"team".as_slice().into(),
                repository: b"widget".as_slice().into()
            }
        })
    );
    for state in [
        forge::FirstState::Making,
        forge::FirstState::Made { id: 42 },
        forge::FirstState::Failed { step: forge::CreateStep::Admin },
        forge::FirstState::Failed { step: forge::CreateStep::Org },
        forge::FirstState::Failed { step: forge::CreateStep::Repository },
        forge::FirstState::Failed { step: forge::CreateStep::File },
    ] {
        let first = forge::First {
            playground: false,
            org: b"team".as_slice().into(),
            repository: b"widget".as_slice().into(),
            state,
            label: Box::from(b"temper-host".as_slice()),
        };
        let mut row = forge_row(
            3,
            forge::Kind::Forgejo,
            forge::Mode::Owned,
            forge::Connection::Connecting,
            forge::Offer::NotOffered,
        );
        row.first = Some(first);
        let value = forge::Forges { rows: Box::new([row]) };
        let bytes = family::forges(value, false, &w::CEILINGS).unwrap();
        let rows = wire::decode_forges(&bytes, &w::CEILINGS).unwrap().into_parts().forges;
        let first = rows.get(0).unwrap().first().as_ref().unwrap();
        assert_eq!((first.org(), first.repository()), (b"team".as_slice(), b"widget".as_slice()));
        match (state, first.state()) {
            (forge::FirstState::Making, w::FirstState::Making) => {}
            (forge::FirstState::Made { id }, w::FirstState::Made(value)) => assert_eq!(id, value.id()),
            (forge::FirstState::Failed { step }, w::FirstState::Failed(value)) => assert_eq!(
                value.step(),
                &match step {
                    forge::CreateStep::Admin => w::CreateStep::Admin,
                    forge::CreateStep::Org => w::CreateStep::Org,
                    forge::CreateStep::Repository => w::CreateStep::Repository,
                    forge::CreateStep::File => w::CreateStep::File,
                }
            ),
            _ => panic!("first state changed"),
        }
    }
}
#[test]
fn invalid_setup_refusals_translate_to_the_wire() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    let bytes = family::answer(
        Some(temper_engine_domain::ApplicationAnswer::SetUp(forge::SetUpAnswer::Refused(forge::SetUpRefusal::Invalid))),
        &w::CEILINGS,
        &jig,
    )
    .unwrap();
    let w::Outcome::SetUpRefused(value) =
        w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome
    else {
        panic!("refusal")
    };
    assert_eq!(value.why(), &w::SetUpRefusal::Invalid);
}

#[test]
fn playground_start_outcomes_encode_every_failure_and_unknown() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    let group = w::GroupFailed::new(&w::CEILINGS, w::GroupFailedParts { why: b"prepare 1 exited 1".as_slice().into() })
        .unwrap();
    for (answer, expected) in [
        (None, w::Outcome::Unknown),
        (Some(forge::StartAnswer::Running), w::Outcome::Running),
        (
            Some(forge::StartAnswer::NotOffered),
            w::Outcome::StartRefused(
                w::StartRefused::new(&w::CEILINGS, w::StartRefusedParts { why: w::StartRefusal::NotOffered }).unwrap(),
            ),
        ),
        (
            Some(forge::StartAnswer::Failed(forge::StartFailure::Account)),
            w::Outcome::StartFailed(
                w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Account }).unwrap(),
            ),
        ),
        (
            Some(forge::StartAnswer::Failed(forge::StartFailure::Adoption)),
            w::Outcome::StartFailed(
                w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Adoption }).unwrap(),
            ),
        ),
        (
            Some(forge::StartAnswer::Failed(forge::StartFailure::NotConnected)),
            w::Outcome::StartFailed(
                w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::NotConnected }).unwrap(),
            ),
        ),
        (
            Some(forge::StartAnswer::Failed(forge::StartFailure::Group {
                why: b"prepare 1 exited 1".as_slice().into(),
            })),
            w::Outcome::StartFailed(
                w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Group(group) }).unwrap(),
            ),
        ),
    ] {
        let (first, playground) = match answer {
            Some(answer) => (
                Some(root::ApplicationAnswer::StartPlayground(answer.clone())),
                Some(root::ApplicationAnswer::StartPlayground(answer)),
            ),
            None => (None, None),
        };
        for answer in [first, playground] {
            let bytes = family::answer(answer, &w::CEILINGS, &jig).unwrap();
            assert_eq!(
                w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome,
                expected
            );
        }
    }
}

#[test]
fn install_and_manifest_paths_are_bounded_visible_paths_on_this_listener() {
    let mut at_bound = [b'a'; 512];
    at_bound[0] = b'/';
    let mut past_bound = [b'a'; 513];
    past_bound[0] = b'/';
    for (name, path, accepted) in [
        ("bound", at_bound.as_slice(), true),
        ("past bound", past_bound.as_slice(), false),
        ("empty", b"".as_slice(), false),
        ("root", b"/", true),
        ("authority", b"//x", false),
        ("backslash", b"/\\x", false),
        ("space", b"/a b", false),
        ("scheme", b"http://x", false),
        ("control", b"/a\n", false),
        ("non ASCII", b"/\x80", false),
        ("delete", b"/\x7f", false),
        ("connect", b"/accounts/connect?provider=1&way=sign-in&source=7", true),
        ("welcome", b"/welcome?at=ready", true),
        ("path colon", b"/a:b?c=d#e", true),
        ("path scheme", b"/https://x", true),
        ("forges", b"/forges?x=1&y=2", true),
    ] {
        assert_eq!(family::return_path(path), accepted, "{name}");
        // Oversized values cannot pass the wire's own bound; all other rows also cross decode.
        if path.len() <= 512 {
            for request in [
                w::Request::Install(
                    w::Install::new(&w::CEILINGS, w::InstallParts { forge: 1, back: Some(path.into()) }).unwrap(),
                ),
                w::Request::Connect(
                    w::Connect::new(
                        &w::CEILINGS,
                        w::ConnectParts {
                            forge: w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge: 1 }).unwrap()),
                            how: w::How::GithubManifest(
                                w::GitHubManifest::new(
                                    &w::CEILINGS,
                                    w::GitHubManifestParts { back: Some(path.into()) },
                                )
                                .unwrap(),
                            ),
                        },
                    )
                    .unwrap(),
                ),
            ] {
                let request = w::Requesting::new(&w::CEILINGS, w::RequestingParts { request }).unwrap();
                let result = family::to_root(
                    &jig_protocol_config::Directories::working(),
                    exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&request)),
                    &fixtures::forge_table(&[]),
                    None,
                    &family::Deployment { welcome: false, owned: false, program_login: None },
                    None,
                    &w::CEILINGS,
                );
                if accepted {
                    let root::Event::Party(root::PartyInput::Application { request, .. }) = result.unwrap() else {
                        panic!("{name}: application")
                    };
                    let actual = match request {
                        root::Application::Try { .. }
                        | root::Application::Dismiss { .. }
                        | root::Application::Looked { .. }
                        | root::Application::SetAway { .. }
                        | root::Application::Overturn { .. } => {
                            panic!("manifest or install")
                        }
                        root::Application::Forge(ask) => {
                            let (forge::Ask::Install { forge, back: Some(back) }
                            | forge::Ask::Connect {
                                forge: forge::Making::Kept(forge),
                                how: forge::How::GitHubManifest { back: Some(back) },
                            }) = ask
                            else {
                                panic!("{name}: install")
                            };
                            assert_eq!(forge, 1, "{name}");
                            back
                        }
                    };
                    assert_eq!(actual.as_ref(), path, "{name}");
                } else {
                    let kit::Event::Application(kit::FromApplication::Undecodable { to }) = *result.unwrap_err() else {
                        panic!("{name}: undecodable")
                    };
                    assert_eq!(to, REPLY, "{name}");
                }
            }
        }
    }
}

#[test]
fn install_without_a_back_is_admitted() {
    let request = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::Install(
                w::Install::new(&w::CEILINGS, w::InstallParts { forge: 1, back: None }).unwrap(),
            ),
        },
    )
    .unwrap();
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&request)),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application {
        request: root::Application::Forge(forge::Ask::Install { forge: 1, back: None }),
        ..
    }) = event
    else {
        panic!("install without back")
    };
}

#[test]
fn installations_and_the_last_connect_end_encode_unchanged() {
    let bytes = family::forges(
        forge::Forges {
            rows: Box::new([forge::Row {
                starts_with: forge::StartsWith::Repository,
                makings: Box::new([]),
                host: b"git.example:3000".as_slice().into(),
                number: 1,
                kind: forge::Kind::GitHub,
                mode: forge::Mode::Connected,
                connection: forge::Connection::NotConnected,
                failed: None,
                fetched: None,
                offer: forge::Offer::NotOffered,
                first: None,
                installations: Box::new([
                    (
                        11,
                        forge::Installation {
                            account: Some((111, Box::from(b"acme".as_slice()))),
                            suspended: true,
                            stray: false,
                        },
                    ),
                    (22, forge::Installation { account: None, suspended: false, stray: true }),
                ]),
                ended: Some(Box::from(b"the pasted value changed".as_slice())),
                app: None,
                web: None,
                repositories: Box::new([]),
                owned: None,
                admin: true,
                read_only: false,
                ended_stop: Some(forge::AdminStop::Refused { status: 403, stage: forge::SetupStage::Token }),
            }]),
        },
        false,
        &w::CEILINGS,
    )
    .unwrap();
    let rows = wire::decode_forges(&bytes, &w::CEILINGS).unwrap().into_parts().forges;
    let row = rows.get(0).unwrap();
    assert!(row.admin());
    assert_eq!(
        row.ended_stop(),
        &Some(w::AdminStop::Refused(
            w::AdminRefused::new(&w::CEILINGS, w::AdminRefusedParts { status: 403, stage: w::SetupStage::Token })
                .unwrap()
        ))
    );
    let installation = row.installations().get(0).unwrap();
    assert_eq!((installation.id(), installation.suspended(), installation.stray()), (11, true, false));
    let account = installation.account().as_ref().unwrap();
    assert_eq!((account.id(), account.login()), (111, b"acme".as_slice()));
    let installation = row.installations().get(1).unwrap();
    assert_eq!(
        (installation.id(), installation.account(), installation.suspended(), installation.stray()),
        (22, &None, false, true)
    );
    assert_eq!(row.ended().as_ref().unwrap().why(), b"the pasted value changed");
}

#[test]
fn embedded_client_refusals_keep_their_bytes_at_client_sixteen() {
    // Embedded jig refusal tags keep their bytes at this forge family version.
    let mut after = skein_lib::Writer::new(1);
    jig_client_protocol_wire::v19::Refusal::Unknown.encode(&mut after).unwrap();
    assert_eq!(after.finish().as_ref(), &[9]);
    assert_eq!(wire::CLIENT_VERSION, 16);
}

#[test]
fn owned_details_fit_their_bounds_and_an_overlong_field_drops_only_the_details() {
    for (forgejo, runner, password, kept) in [
        (64, Some(64), 512, true),
        (64, None, 512, true),
        (65, Some(64), 512, false),
        (64, Some(65), 512, false),
        (64, Some(64), 513, false),
    ] {
        let value = forge::Forges {
            rows: Box::new([forge::Row {
                starts_with: forge::StartsWith::Repository,
                makings: Box::new([]),
                host: b"git.example:3000".as_slice().into(),
                number: 1,
                kind: forge::Kind::Forgejo,
                mode: forge::Mode::Owned,
                connection: forge::Connection::Connecting,
                failed: None,
                fetched: None,
                offer: forge::Offer::NotOffered,
                first: None,
                installations: Box::new([]),
                ended: None,
                app: None,
                web: None,
                repositories: Box::new([]),
                owned: Some(forge::Owned {
                    forgejo: Box::from(&[b'v'; 65][..forgejo]),
                    runner: match runner {
                        Some(len) => Some(Box::from(&[b'r'; 65][..len])),
                        None => None,
                    },
                    password: Box::from(&[b'/'; 513][..password]),
                }),
                admin: false,
                read_only: false,
                ended_stop: None,
            }]),
        };
        let bytes = family::forges(value, false, &w::CEILINGS).expect("the row is still available");
        let rows = temper_client_protocol_wire::decode_forges(&bytes, &w::CEILINGS).unwrap();
        assert_eq!(rows.forges().len(), 1);
        let owned = rows.forges().as_slice()[0].owned();
        assert_eq!(owned.is_some(), kept);
        if let Some(owned) = owned {
            assert_eq!(owned.forgejo().len(), forgejo);
            assert_eq!(
                match owned.runner() {
                    Some(runner) => Some(runner.len()),
                    None => None,
                },
                runner
            );
            assert_eq!(owned.password().len(), password);
        }
    }
}

#[test]
fn accept_decodes_only_a_nonzero_installation_and_keeps_its_id() {
    for installation in [0, 22, u64::MAX] {
        let request = w::Requesting::new(
            &w::CEILINGS,
            w::RequestingParts {
                request: w::Request::Accept(
                    w::Accept::new(&w::CEILINGS, w::AcceptParts { forge: 1, installation }).unwrap(),
                ),
            },
        )
        .unwrap();
        let result = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&request)),
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        );
        if installation == 0 {
            assert!(result.is_err());
        } else {
            let root::Event::Party(root::PartyInput::Application {
                request: root::Application::Forge(forge::Ask::Accept { forge: 1, installation: id }),
                ..
            }) = result.unwrap()
            else {
                panic!("accept")
            };
            assert_eq!(id.get(), installation);
        }
    }
}
#[test]
fn accept_answers_use_the_appended_success_and_existing_refusals() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    for (answer, expected) in [
        (
            forge::AcceptAnswer::Accepted { installation: 22 },
            w::Outcome::Accepted(
                w::AcceptedInstallation::new(&w::CEILINGS, w::AcceptedInstallationParts { installation: 22 }).unwrap(),
            ),
        ),
        (
            forge::AcceptAnswer::Refused(forge::ForgeRefusal::Role),
            w::Outcome::ConnectRefused(
                w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::Role }).unwrap(),
            ),
        ),
        (
            forge::AcceptAnswer::ForgeRefused(forge::AdoptionRefusal::NotFound),
            w::Outcome::ForgeRefused(
                w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::NotFound }).unwrap(),
            ),
        ),
    ] {
        let bytes = family::answer(Some(root::ApplicationAnswer::Accept(answer)), &w::CEILINGS, &jig).unwrap();
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome, expected);
    }
}

#[test]
fn a_fake_forges_panel_has_no_address_and_real_forgejos_keep_theirs() {
    for mode in [forge::Mode::Fake, forge::Mode::Owned, forge::Mode::Connected] {
        let mut forges = configured(b"http://forge.example");
        forges[0].mode = mode;
        let got = decoded_panel(panel(), &fixtures::forge_table(&forges)).into_parts().pull.unwrap();
        assert_eq!(
            got.address(),
            if mode == forge::Mode::Fake {
                b"".as_slice()
            } else {
                b"http://forge.example/acme/shop/pulls/123".as_slice()
            }
        );
    }
}

fn landing_view() -> forge::LandingView {
    forge::LandingView {
        task: 61,
        project: 7,
        phase: forge::LandingPhase::Proving,
        preparation: 2,
        stale_proofs: 3,
        turn: Some(62),
        members: Box::new([forge::LandingMember {
            change: CHANGE_TASK,
            forge: 1,
            owner: Box::from(b"acme".as_slice()),
            name: Box::from(b"shop".as_slice()),
            branch: Box::from(b"temper/51".as_slice()),
            landing: Box::from(b"main".as_slice()),
            pull: Some(123),
            base: Some([2; 32]),
            candidate: Some([3; 32]),
            landed: None,

            gates: Box::new([forge::LandingGate {
                name: Box::from(b"gate".as_slice()),
                blocking: true,
                key: Some([4; 32]),
                faults: 1,
                proof: forge::GateProof::Passed(forge::ReceiptRun {
                    id: 7,
                    number: 3,
                    url: Box::from(b"https://forge.example/run/7".as_slice()),
                }),
            }]),
        }]),
    }
}
#[test]
fn a_landing_view_encodes_to_its_panel() {
    let mut forges = fixtures::forges();
    forges[0].kind = crate::settings::Kind::Forgejo;
    forges[0].git = Box::from(b"https://git.example".as_slice());
    forges[0].web = Some(Box::from(b"https://forge.example".as_slice()));
    let bytes =
        family::document(forge::Document::Landing(landing_view()), &fixtures::forge_table(&forges), &w::CEILINGS)
            .unwrap();
    let got = wire::decode_landing_panel(&bytes, &w::CEILINGS).unwrap();
    assert_eq!(
        (got.task(), got.project(), got.phase(), got.preparation(), got.stale_proofs(), got.turn()),
        (61, 7, &w::LandingPhase::Proving, 2, 3, &Some(62))
    );
    let member = got.members().get(0).unwrap();
    assert_eq!(
        (member.change(), member.forge(), member.base(), member.candidate(), member.landed()),
        (51, 1, &Some([2; 32]), &Some([3; 32]), &None)
    );
    assert_eq!(member.owner(), b"acme");
    assert_eq!(member.name(), b"shop");
    assert_eq!(member.branch(), b"temper/51");
    assert_eq!(member.landing(), b"main");
    assert_eq!(member.pull().as_ref().unwrap().address(), b"https://git.example/acme/shop/pulls/123");
    let gate = member.gates().get(0).unwrap();
    assert_eq!(gate.key(), &Some([4; 32]));
    assert!(gate.blocking());
    assert_eq!(gate.faults(), 1);
    let w::Proof::Passed(run) = gate.proof() else { panic!("receipt") };
    assert_eq!((run.run(), run.number(), run.address()), (7, 3, b"https://forge.example/run/7".as_slice()));
}
#[test]
fn a_landings_view_encodes_to_its_list() {
    let value = forge::LandingsView {
        project: 7,
        turn: Some(61),
        landings: Box::new([forge::LandingLine {
            task: 61,
            phase: forge::LandingPhase::Turn { at: skein_lib::Wall::EPOCH },
            changes: Box::new([51]),
            repositories: Box::new([Box::from(b"acme/shop".as_slice())]),
            landed: Box::new([[3; 32]]),
        }]),
    };
    let bytes = family::document(forge::Document::Landings(value), &fixtures::forge_table(&[]), &w::CEILINGS).unwrap();
    let got = wire::decode_landings_panel(&bytes, &w::CEILINGS).unwrap();
    assert_eq!((got.project(), got.turn()), (7, &Some(61)));
    assert_eq!(got.landings().len(), 1);
    let row = got.landings().get(0).unwrap();
    assert_eq!((row.task(), row.phase()), (61, &w::LandingPhase::Turn));
    assert_eq!(row.changes().as_slice(), &[51]);
    assert_eq!(row.repositories().as_slice(), &[Box::<[u8]>::from(b"acme/shop".as_slice())]);
    assert_eq!(row.landed().as_slice(), &[[3; 32]]);
}
#[test]
fn the_landings_subject_names_no_task() {
    for project in [0, 7, u32::MAX] {
        for ended in [false, true] {
            let record = wire::watching(
                &w::Watching::new(
                    &w::CEILINGS,
                    w::WatchingParts {
                        watch: w::Watch::Landings(
                            w::LandingsSubject::new(&w::CEILINGS, w::LandingsSubjectParts { project, ended }).unwrap(),
                        ),
                    },
                )
                .unwrap(),
            );
            let got = family::to_root(
                &jig_protocol_config::Directories::working(),
                watch_exchange(wire::FORGE, wire::CLIENT_VERSION, record),
                &fixtures::forge_table(&[]),
                None,
                &family::Deployment { welcome: false, owned: false, program_login: None },
                None,
                &w::CEILINGS,
            )
            .unwrap();
            let root::Event::Party(root::PartyInput::Watch {
                subject: jig_domain_core::documents::Watched::Application { subject, .. },
                ..
            }) = got
            else {
                panic!("application watch")
            };
            assert_eq!(subject.raw(), (1_u64 << 62_u32) | (u64::from(ended) << 32_u32) | u64::from(project));
            assert!(subject.raw() >= (1_u64 << 62_u32));
        }
    }
    let record = wire::watching(
        &w::Watching::new(
            &w::CEILINGS,
            w::WatchingParts {
                watch: w::Watch::Landing(
                    w::LandingSubject::new(&w::CEILINGS, w::LandingSubjectParts { task: 61 }).unwrap(),
                ),
            },
        )
        .unwrap(),
    );
    let got = family::to_root(
        &jig_protocol_config::Directories::working(),
        watch_exchange(wire::FORGE, wire::CLIENT_VERSION, record),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Watch { subject, .. }) = got else { panic!("landing watch") };
    assert_eq!(
        subject,
        jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(7),
            connector: wire::FORGE,
            subject: Token::new(61)
        }
    );
    let record = wire::watching(
        &w::Watching::new(
            &w::CEILINGS,
            w::WatchingParts {
                watch: w::Watch::Landing(
                    w::LandingSubject::new(&w::CEILINGS, w::LandingSubjectParts { task: 0 }).unwrap(),
                ),
            },
        )
        .unwrap(),
    );
    let Err(refused) = family::to_root(
        &jig_protocol_config::Directories::working(),
        watch_exchange(wire::FORGE, wire::CLIENT_VERSION, record),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    ) else {
        panic!("zero landing task is refused")
    };
    let kit::Event::WatchRefused { watcher, why } = *refused else { panic!("unknown watch") };
    assert_eq!((watcher, why), (WATCHER, jig_domain_core::WatchRefusal::Unknown));
}

#[test]
fn forge_removal_and_playground_requests_keep_their_envelopes() {
    for (request, expected) in [
        (
            w::Request::Dismiss(
                w::Dismiss::new(
                    &w::CEILINGS,
                    w::DismissParts {
                        what: w::PersonFact::ToBegin(
                            w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: 2 }).unwrap(),
                        ),
                    },
                )
                .unwrap(),
            ),
            root::Application::Dismiss { what: forge::PersonFact::ToBegin { project: 2 } },
        ),
        (
            w::Request::Dismiss(
                w::Dismiss::new(
                    &w::CEILINGS,
                    w::DismissParts {
                        what: w::PersonFact::KeepAsking(
                            w::KeepAsking::new(
                                &w::CEILINGS,
                                w::KeepAskingParts {
                                    project: 2,
                                    subject: temper_protocol_reaches::subject_record(
                                        temper_protocol_reaches::Subject::Kind(temper_protocol_reaches::Kind::Tests),
                                    ),
                                    at: 17,
                                },
                            )
                            .unwrap(),
                        ),
                    },
                )
                .unwrap(),
            ),
            root::Application::Dismiss {
                what: forge::PersonFact::KeepAsking {
                    project: 2,
                    subject: temper_protocol_reaches::Subject::Kind(temper_protocol_reaches::Kind::Tests),
                    at: 17,
                },
            },
        ),
        (
            w::Request::RemoveForge(w::RemoveForge::new(&w::CEILINGS, w::RemoveForgeParts { forge: 9 }).unwrap()),
            root::Application::Forge(forge::Ask::RemoveForge { forge: 9 }),
        ),
        (w::Request::StartPlayground, root::Application::Forge(forge::Ask::StartPlayground)),
        (w::Request::EndPlayground, root::Application::Forge(forge::Ask::EndPlayground)),
    ] {
        let record = w::Requesting::new(&w::CEILINGS, w::RequestingParts { request }).unwrap();
        let event = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&record)),
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        )
        .unwrap();
        let root::Event::Party(root::PartyInput::Application {
            reply_to, sign_in, key, digest, scope, request, ..
        }) = event
        else {
            panic!("application request")
        };
        assert_eq!(
            (reply_to, sign_in, key, digest, scope),
            (REPLY, 3, REQUEST_KEY, [7; 32], people::ApplicationScope::Project(7))
        );
        assert_eq!(request, expected);
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table checks both public acts across every forge place")]
fn new_forge_places_reach_both_setup_and_connect_whole() {
    let managed = w::ForgeAsk::Managed(
        w::ManagedForge::new(
            &w::CEILINGS,
            w::ManagedForgeParts {
                git_program: Some(b"bin/git".as_slice().into()),
                host: b"forge.example".as_slice().into(),
                port: 8443,
                tls: Some(b"forge.example".as_slice().into()),
                git: b"https://forge.example/git".as_slice().into(),
                web: Some(b"https://forge.example".as_slice().into()),
            },
        )
        .unwrap(),
    );
    let place = forge::Place {
        api: forge::Address {
            host: b"forge.example".as_slice().into(),
            port: 8443,
            tls: Some(b"forge.example".as_slice().into()),
        },
        git: b"https://forge.example/git".as_slice().into(),
        web: Some(b"https://forge.example".as_slice().into()),
    };
    for (asked, expected) in [
        (w::ForgeAsk::Github, forge::ForgeAsk::GitHub),
        (w::ForgeAsk::Owned, forge::ForgeAsk::Owned),
        (
            {
                let w::ForgeAsk::Managed(value) = managed.clone() else { unreachable!() };
                w::ForgeAsk::Connected(value)
            },
            forge::ForgeAsk::Connected { place: place.clone(), git_program: Some(b"/data/bin/git".as_slice().into()) },
        ),
        (managed, forge::ForgeAsk::Managed { place, git_program: Some(b"/data/bin/git".as_slice().into()) }),
    ] {
        let making = w::Making::New(w::NewForge::new(&w::CEILINGS, w::NewForgeParts { forge: asked }).unwrap());
        for (request, expected) in [
            (
                w::Request::SetUp(
                    w::SetUp::new(
                        &w::CEILINGS,
                        w::SetUpParts {
                            forge: making.clone(),
                            first: w::SetUpFirst::Empty(
                                w::FirstEmpty::new(
                                    &w::CEILINGS,
                                    w::FirstEmptyParts {
                                        org: b"work".as_slice().into(),
                                        repository: b"hello".as_slice().into(),
                                        playground: false,
                                    },
                                )
                                .expect("setup names fit wire ceilings"),
                            ),
                        },
                    )
                    .unwrap(),
                ),
                forge::Ask::SetUp {
                    forge: forge::Making::New(expected.clone()),
                    first: forge::SetUpFirst::Empty {
                        org: b"work".as_slice().into(),
                        repository: b"hello".as_slice().into(),
                        playground: false,
                    },
                },
            ),
            (
                w::Request::Connect(
                    w::Connect::new(
                        &w::CEILINGS,
                        w::ConnectParts {
                            forge: making,
                            how: w::How::ForgejoAdmin(
                                w::ForgejoAdmin::new(&w::CEILINGS, w::ForgejoAdminParts { first: None }).unwrap(),
                            ),
                        },
                    )
                    .unwrap(),
                ),
                forge::Ask::Connect {
                    forge: forge::Making::New(expected),
                    how: forge::How::ForgejoAdmin { first: None },
                },
            ),
        ] {
            let record = w::Requesting::new(&w::CEILINGS, w::RequestingParts { request }).unwrap();
            let event = family::to_root(
                &jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap(),
                exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&record)),
                &fixtures::forge_table(&[]),
                None,
                &family::Deployment { welcome: false, owned: false, program_login: None },
                None,
                &w::CEILINGS,
            )
            .unwrap();
            let root::Event::Party(root::PartyInput::Application { request, .. }) = event else {
                panic!("application request")
            };
            assert_eq!(request, root::Application::Forge(expected));
        }
    }
}
#[test]
fn removal_and_playground_answers_use_their_own_outcomes() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    for (answer, expected) in [
        (root::ApplicationAnswer::Try(root::TryAnswer::Tried), w::Outcome::Tried),
        (root::ApplicationAnswer::Try(root::TryAnswer::NotTheirs), w::Outcome::NotTheirs),
        (root::ApplicationAnswer::Dismiss(forge::DismissAnswer::Dismissed), w::Outcome::Dismissed),
        (
            root::ApplicationAnswer::Dismiss(forge::DismissAnswer::TooLarge),
            w::Outcome::ForgeRefused(
                w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::TooLarge }).unwrap(),
            ),
        ),
        (root::ApplicationAnswer::RemoveForge(forge::RemoveAnswer::Removed), w::Outcome::Removed),
        (
            root::ApplicationAnswer::RemoveForge(forge::RemoveAnswer::Refused(forge::ForgeRefusal::Role)),
            w::Outcome::RemoveRefused(
                w::RemoveRefused::new(&w::CEILINGS, w::RemoveRefusedParts { why: w::ForgeRefusal::Role }).unwrap(),
            ),
        ),
        (root::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running), w::Outcome::Running),
        (root::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset), w::Outcome::Reset),
        (
            root::ApplicationAnswer::EndPlayground(forge::EndAnswer::NotOffered),
            w::Outcome::EndRefused(
                w::EndRefused::new(&w::CEILINGS, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).unwrap(),
            ),
        ),
    ] {
        let bytes = family::answer(Some(answer), &w::CEILINGS, &jig).unwrap();
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome, expected);
    }
}

fn playground_read(row: Option<&forge::Playground>, demo: Option<&crate::settings::Demo>) -> Option<Box<[u8]>> {
    let mut input = read_input(people::ApplicationScope::Deployment);
    if let kit::ToApplication::Read { record, .. } = &mut input {
        *record = wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Playground }).unwrap());
    }
    let Err(answer) = family::to_root(
        &jig_protocol_config::Directories::working(),
        input,
        &fixtures::forge_table(&[]),
        demo,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        row,
        &w::CEILINGS,
    ) else {
        panic!("playground read")
    };
    let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *answer else {
        panic!("playground answer")
    };
    assert_eq!((reply, family), (REPLY, wire::FORGE));
    record
}
#[test]
fn the_playground_read_answers_from_the_row_and_the_settings() {
    let mut service = fixtures::demo_service();
    let demo = service.engine.as_mut().unwrap().demo.as_mut().unwrap();
    demo.owner = b"playground".as_slice().into();
    for (state, project, expected) in [
        (forge::PlaygroundState::Starting, None, w::PlaygroundState::Starting),
        (forge::PlaygroundState::Running, Some(13), w::PlaygroundState::Running),
        (forge::PlaygroundState::Resetting(forge::ResetStep::Project), Some(13), w::PlaygroundState::Resetting),
        (forge::PlaygroundState::Resetting(forge::ResetStep::Forge), Some(13), w::PlaygroundState::Resetting),
        (forge::PlaygroundState::Resetting(forge::ResetStep::Account), Some(13), w::PlaygroundState::Resetting),
    ] {
        let row = forge::Playground { forge: 9, project, account: Some(7), state };
        let record = playground_read(Some(&row), Some(demo)).unwrap();
        let answer = wire::decode_playground_answer(&record, &w::CEILINGS).unwrap();
        assert_eq!((answer.forge(), *answer.project(), answer.provider()), (9, project, demo.provider));
        assert_eq!(answer.state(), &expected);
        assert_eq!(answer.owner(), b"playground");
        assert_eq!(answer.name(), b"widget");
        assert_eq!(answer.cues().as_slice(), demo.cues.as_ref());
    }
}
#[test]
fn no_row_is_the_empty_answer() {
    let service = fixtures::demo_service();
    assert!(playground_read(None, service.engine.as_ref().unwrap().demo.as_ref()).is_none());
    assert!(playground_read(None, None).is_none());
}
#[test]
fn end_playground_answers_reset() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    let bytes =
        family::answer(Some(root::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset)), &w::CEILINGS, &jig)
            .unwrap();
    assert_eq!(
        w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome,
        w::Outcome::Reset
    );
}

#[test]
fn after_landing_steps_translate_all_tier_lines_and_uncleared() {
    let lines: Box<[Box<[u8]>]> = Box::new([Box::from(b"fast run 2".as_slice()), Box::new([b'x'; 256])]);
    for input in [
        forge::PanelStep::Verifying { merge: [8; 32], tiers: lines.clone() },
        forge::PanelStep::Red { merge: [8; 32], tiers: lines.clone() },
        forge::PanelStep::Cleared { merge: [8; 32], tiers: lines.clone(), uncleared: false },
        forge::PanelStep::Cleared { merge: [8; 32], tiers: lines.clone(), uncleared: true },
    ] {
        let mut value = panel();
        value.step = input.clone();
        let got = decoded_panel(value, &fixtures::forge_table(&[]));
        match (input, got.step()) {
            (forge::PanelStep::Verifying { merge, tiers }, w::Step::Verifying(value)) => {
                assert_eq!(*value.merge(), merge);
                assert_eq!(value.tiers().as_slice(), tiers.as_ref());
            }
            (forge::PanelStep::Red { merge, tiers }, w::Step::Red(value)) => {
                assert_eq!(*value.merge(), merge);
                assert_eq!(value.tiers().as_slice(), tiers.as_ref());
            }
            (forge::PanelStep::Cleared { merge, tiers, uncleared }, w::Step::Cleared(value)) => {
                assert_eq!(*value.merge(), merge);
                assert_eq!(value.tiers().as_slice(), tiers.as_ref());
                assert_eq!(value.uncleared(), uncleared);
            }
            _ => panic!("step preserved"),
        }
    }
}

#[test]
fn tokens_connect_and_replace_keep_their_carried_values_and_envelopes() {
    for (request, expected) in [
        (
            w::Request::Connect(
                w::Connect::new(
                    &w::CEILINGS,
                    w::ConnectParts {
                        forge: w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge: 9 }).unwrap()),
                        how: w::How::ForgejoTokens(
                            w::ForgejoTokens::new(&w::CEILINGS, w::ForgejoTokensParts {}).unwrap(),
                        ),
                    },
                )
                .unwrap(),
            ),
            forge::Ask::Connect { forge: forge::Making::Kept(9), how: forge::How::ForgejoTokens },
        ),
        (
            w::Request::Replace(w::Replace::new(&w::CEILINGS, w::ReplaceParts { forge: 9 }).unwrap()),
            forge::Ask::Replace { forge: 9 },
        ),
    ] {
        let record = w::Requesting::new(&w::CEILINGS, w::RequestingParts { request }).unwrap();
        let carried = Some(jig_domain_core::Carried { number: 27, digest: [8; 32] });
        let event = family::to_root(
            &jig_protocol_config::Directories::working(),
            kit::ToApplication::Request {
                reply_to: REPLY,
                sign_in: 3,
                key: REQUEST_KEY,
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                record: wire::requesting(&record),
                digest: [7; 32],
                carried,
                scope: people::ApplicationScope::Deployment,
            },
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        )
        .unwrap();
        let root::Event::Party(root::PartyInput::Application {
            reply_to,
            sign_in,
            key,
            digest,
            scope,
            carried: got,
            request,
        }) = event
        else {
            panic!("application request")
        };
        assert_eq!(
            (reply_to, sign_in, key, digest, scope, got),
            (REPLY, 3, REQUEST_KEY, [7; 32], people::ApplicationScope::Deployment, carried)
        );
        assert_eq!(request, root::Application::Forge(expected));
    }
}

#[test]
fn connect_and_replace_answers_share_the_connect_outcomes() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    for (answer, expected) in [
        (Some(root::ApplicationAnswer::Connect(forge::ConnectAnswer::Connecting)), w::Outcome::Connecting),
        (
            Some(root::ApplicationAnswer::Connect(forge::ConnectAnswer::Refused(forge::ForgeRefusal::NotTaken))),
            w::Outcome::ConnectRefused(
                w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::NotTaken })
                    .unwrap(),
            ),
        ),
        (None, w::Outcome::Unknown),
    ] {
        let bytes = family::answer(answer, &w::CEILINGS, &jig).unwrap();
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome, expected);
    }
}

#[test]
fn forge_settings_reads_reach_the_root_with_their_authorization_envelope() {
    for (read, query, expected_scope) in [
        (
            w::Read::Setting(
                w::SettingRead::new(&w::CEILINGS, w::SettingReadParts { kind: 2, scope: 7, number: 3 }).unwrap(),
            ),
            root::ForgeRead::Setting { kind: 2, scope: 7, number: 3 },
            people::ApplicationScope::Deployment,
        ),
        (
            w::Read::ProposalItems(
                w::ProposalItemsRead::new(&w::CEILINGS, w::ProposalItemsReadParts { key: u64::MAX }).unwrap(),
            ),
            root::ForgeRead::ProposalItems { key: u64::MAX },
            people::ApplicationScope::Deployment,
        ),
        (
            w::Read::View(
                w::ViewRead::new(
                    &w::CEILINGS,
                    w::ViewReadParts { project: 7, address: b"proposal t12/3".as_slice().into() },
                )
                .unwrap(),
            ),
            root::ForgeRead::View { project: 7, address: b"proposal t12/3".as_slice().into() },
            people::ApplicationScope::Project(7),
        ),
    ] {
        let record = wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read }).unwrap());
        let exchange = kit::ToApplication::Read {
            reply: REPLY,
            sign_in: 3,
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            record,
            scope: expected_scope,
            window: jig_domain_core::documents::Window { rows: 1, after: None },
        };
        let root::Event::Party(root::PartyInput::ForgeRead { reply, sign_in, scope, query: actual }) = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange,
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        )
        .unwrap() else {
            panic!("typed forge read")
        };
        assert_eq!((reply, sign_in, scope, actual), (REPLY, 3, expected_scope, query));
    }
}
#[test]
fn forge_settings_answers_use_the_kinds_codec_and_preserve_proposal_edits() {
    use forge::settings::{Edit, SettingKey, SettingSlot, SizedValue, Value};
    let value = SizedValue { size: 4, value: Value::Charter(6) };
    let record = family::forge_read(root::ForgeReadAnswer::Setting(Some(value.clone())), &[], &w::CEILINGS).unwrap();
    assert_eq!(
        wire::decode_setting_answer(&record, &w::CEILINGS).unwrap().value().as_deref(),
        Some(temper_engine_smith::kinds::encode(&value.value).as_ref())
    );
    let empty = family::forge_read(root::ForgeReadAnswer::Setting(None), &[], &w::CEILINGS).unwrap();
    assert!(wire::decode_setting_answer(&empty, &w::CEILINGS).unwrap().value().is_none());
    let items = Box::new([
        (SettingKey { kind: 3, scope: 0, number: 1, slot: SettingSlot::Proposal(7) }, Edit::Set(value.clone())),
        (SettingKey { kind: 2, scope: 9, number: 2, slot: SettingSlot::Proposal(7) }, Edit::Remove),
        (SettingKey { kind: 1, scope: 0, number: 3, slot: SettingSlot::Proposal(7) }, Edit::Reset),
    ]);
    let record =
        family::forge_read(root::ForgeReadAnswer::ProposalItems { items, owned_approval: true }, &[], &w::CEILINGS)
            .unwrap();
    let decoded = wire::decode_proposal_items_answer(&record, &w::CEILINGS).unwrap();
    assert_eq!(decoded.items().len(), 3);
    assert!(decoded.owned_approval());
    for (item, (kind, scope, number)) in decoded.items().iter().zip([(3, 0, 1), (2, 9, 2), (1, 0, 3)]) {
        assert_eq!((item.kind(), item.scope(), item.number()), (kind, scope, number));
    }
    let w::SettingEdit::Set(set) = decoded.items().as_slice()[0].edit() else { panic!("set value") };
    assert_eq!(set.value(), temper_engine_smith::kinds::encode(&value.value).as_ref());
    assert_eq!(*decoded.items().as_slice()[1].edit(), w::SettingEdit::Removed);
    assert_eq!(*decoded.items().as_slice()[2].edit(), w::SettingEdit::Reset);
}

#[test]
fn a_panels_questions_map_field_for_field() {
    for (route, expected_route) in [
        (change::Route::None, w::Route::None),
        (change::Route::Foreman, w::Route::Foreman),
        (change::Route::Up, w::Route::Up),
    ] {
        let mut value = panel();
        let mut open = List::with_capacity(5);
        let mut expected = List::with_capacity(5);
        for (index, (stands, wire)) in [
            (forge::PanelStands::Taken, w::Stands::Taken),
            (
                forge::PanelStands::Foreman(None),
                w::Stands::Foreman(w::Foreman::new(&w::CEILINGS, w::ForemanParts { task: None }).unwrap()),
            ),
            (
                forge::PanelStands::Foreman(Some(15)),
                w::Stands::Foreman(w::Foreman::new(&w::CEILINGS, w::ForemanParts { task: Some(15) }).unwrap()),
            ),
            (forge::PanelStands::Asked, w::Stands::Asked),
            (forge::PanelStands::Held, w::Stands::Held),
        ]
        .into_iter()
        .enumerate()
        {
            let asker = 71_u64.checked_add(u64::try_from(index).unwrap()).unwrap();
            let number = 1_u64.checked_add(u64::try_from(index).unwrap()).unwrap();
            let line = b"Which branch?".as_slice();
            open.push(forge::PanelQuestion { asker, number, line: Box::from(line), stands }).unwrap();
            expected
                .push(
                    w::Question::new(
                        &w::CEILINGS,
                        w::QuestionParts { asker, number, line: Box::from(line), stands: wire },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        value.questions =
            forge::PanelQuestions { route, taken: forge::PanelUsed { used: 2, limit: 4 }, open: open.into_boxed() };
        let got = decoded_panel(value, &fixtures::forge_table(&[]));
        assert_eq!(
            got.questions(),
            &w::Questions::new(
                &w::CEILINGS,
                w::QuestionsParts {
                    route: expected_route,
                    taken: w::Used::new(&w::CEILINGS, w::UsedParts { used: 2, limit: 4 }).unwrap(),
                    open: expected
                }
            )
            .unwrap()
        );
    }
}

#[test]
fn the_deployment_read_stays_answered_before_any_party() {
    let service = fixtures::demo_service();
    let demo = service.engine.as_ref().unwrap().demo.as_ref().unwrap();
    let row =
        forge::Playground { state: forge::PlaygroundState::Running, forge: 1, project: Some(1), account: Some(1) };
    for deployment in [
        &family::Deployment { welcome: false, owned: false, program_login: None },
        &family::Deployment { welcome: false, owned: true, program_login: None },
        &family::Deployment { welcome: true, owned: false, program_login: None },
        &family::Deployment { welcome: true, owned: true, program_login: None },
    ] {
        for (demo, playground) in [(None, None), (Some(demo), None), (None, Some(&row)), (Some(demo), Some(&row))] {
            let mut input = read_input(people::ApplicationScope::Deployment);
            if let kit::ToApplication::Read { record, sign_in, .. } = &mut input {
                *sign_in = 0;
                *record = wire::reading(
                    &w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Deployment }).unwrap(),
                );
            }
            let Err(answer) = family::to_root(
                &jig_protocol_config::Directories::working(),
                input,
                &fixtures::forge_table(&[]),
                demo,
                deployment,
                playground,
                &w::CEILINGS,
            ) else {
                panic!("deployment read answers directly")
            };
            let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *answer else {
                panic!("deployment answer")
            };
            assert_eq!((reply, family), (REPLY, wire::FORGE));
            let answer = wire::decode_deployment_answer(&record.expect("every deployment answers"), &w::CEILINGS)
                .expect("deployment record");
            assert_eq!((answer.welcome(), answer.owned()), (deployment.welcome, deployment.owned));
        }
    }
}

#[test]
fn deployment_owned_needs_the_document_and_the_box() {
    for (document_owned, under_box, expected) in
        [(false, false, false), (false, true, false), (true, false, false), (true, true, true)]
    {
        assert_eq!(crate::system::deployment_owned(document_owned, under_box), expected);
    }
}

#[test]
fn playground_config_needs_the_welcome() {
    let service = fixtures::demo_service();
    let demo = service.engine.as_ref().unwrap().demo.as_ref().unwrap();
    let deadline = skein_lib::Duration::from_secs(10);
    for (demo, welcome, offered) in
        [(None, false, false), (None, true, false), (Some(demo), false, false), (Some(demo), true, true)]
    {
        let config = crate::system::playground_config(demo, welcome, deadline);
        assert_eq!(config.is_some(), offered);
        if let Some(config) = config {
            assert_eq!(config.provider, demo.unwrap().provider);
            assert_eq!(config.deadline, deadline);
        }
    }
}

fn make_request(from: w::MakeFrom, into: bool) -> w::Requesting {
    w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::MakeRepository(
                w::MakeRepository::new(
                    &w::CEILINGS,
                    w::MakeRepositoryParts {
                        forge: 1,
                        org: b"work".as_slice().into(),
                        name: b"copied".as_slice().into(),
                        from,
                        into: if into {
                            Some(w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: 7, home: true }).unwrap())
                        } else {
                            None
                        },
                    },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap()
}
#[test]
fn make_repository_reaches_the_root_with_its_host_and_kept_source() {
    let forges = configured(b"https://forge.invalid:3000/git");
    for (from, expected, into) in [
        (w::MakeFrom::Empty, forge::MakeFrom::Empty, false),
        (
            w::MakeFrom::Copy(
                w::MakeCopy::new(
                    &w::CEILINGS,
                    w::MakeCopyParts { address: b"source.invalid/upstream/starter.git".as_slice().into() },
                )
                .unwrap(),
            ),
            forge::MakeFrom::Copy { address: b"https://source.invalid/upstream/starter".as_slice().into() },
            true,
        ),
    ] {
        let event = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&make_request(from, into))),
            &fixtures::forge_table(&forges),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        )
        .unwrap();
        let root::Event::Party(root::PartyInput::Application {
            reply_to, sign_in, key, digest, scope, request, ..
        }) = event
        else {
            panic!("application request")
        };
        assert_eq!(
            (reply_to, sign_in, key, digest, scope),
            (REPLY, 3, REQUEST_KEY, [7; 32], people::ApplicationScope::Project(7))
        );
        assert_eq!(
            request,
            root::Application::Forge(forge::Ask::MakeRepository {
                forge: 1,
                org: b"work".as_slice().into(),
                name: b"copied".as_slice().into(),
                from: expected,
                into: if into {
                    Some(forge::MakeInto { project: 7, home: true, host: b"forge.invalid:3000".as_slice().into() })
                } else {
                    None
                },
            })
        );
    }
}
#[test]
fn make_repository_without_a_configured_forge_has_no_host() {
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&make_request(w::MakeFrom::Empty, true))),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application {
        request: root::Application::Forge(forge::Ask::MakeRepository { into: Some(into), .. }),
        ..
    }) = event
    else {
        panic!("making request")
    };
    assert!(into.host.is_empty());
}
#[test]
fn make_repository_refuses_a_source_the_parser_rejects() {
    for address in [b"ssh://source.invalid/upstream/starter".as_slice(), b"source.invalid/too/many/segments"] {
        let request = make_request(
            w::MakeFrom::Copy(w::MakeCopy::new(&w::CEILINGS, w::MakeCopyParts { address: address.into() }).unwrap()),
            false,
        );
        let answer = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&request)),
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        )
        .unwrap_err();
        let kit::Event::Application(kit::FromApplication::Undecodable { to }) = *answer else {
            panic!("bad source is undecodable")
        };
        assert_eq!(to, REPLY);
    }
}
#[test]
fn make_repository_answers_encode_every_outcome() {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    for (answer, expected) in [
        (forge::MakeAnswer::Kept, w::Outcome::MakeKept),
        (forge::MakeAnswer::Refused(forge::MakeRefusal::NoSuchForge), make_refused(w::MakeRefusal::NoSuchForge)),
        (forge::MakeAnswer::Refused(forge::MakeRefusal::NotOffered), make_refused(w::MakeRefusal::NotOffered)),
        (forge::MakeAnswer::Refused(forge::MakeRefusal::NotOwner), make_refused(w::MakeRefusal::NotOwner)),
        (forge::MakeAnswer::Refused(forge::MakeRefusal::Invalid), make_refused(w::MakeRefusal::Invalid)),
        (forge::MakeAnswer::Refused(forge::MakeRefusal::HomeHeld), make_refused(w::MakeRefusal::HomeHeld)),
        (forge::MakeAnswer::Refused(forge::MakeRefusal::Full), make_refused(w::MakeRefusal::Full)),
    ] {
        let bytes = family::answer(Some(root::ApplicationAnswer::Make(answer)), &w::CEILINGS, &jig).unwrap();
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome, expected);
    }
}
fn make_refused(why: w::MakeRefusal) -> w::Outcome {
    w::Outcome::MakeRefused(w::MakeRefused::new(&w::CEILINGS, w::MakeRefusedParts { why }).unwrap())
}
#[test]
fn forge_rows_encode_the_makings_and_all_their_states() {
    // Four live states and five steps with six failure reasons.
    const STATES: u32 = 34;
    let made =
        w::MakingMade::new(&w::CEILINGS, w::MakingMadeParts { id: 23, branch: b"trunk".as_slice().into() }).unwrap();
    let mut states = List::with_capacity(STATES);
    for state in [
        (forge::MakingState::Waiting, w::MakingState::Waiting),
        (forge::MakingState::Making, w::MakingState::Making),
        (forge::MakingState::Made { id: 23, branch: b"trunk".as_slice().into() }, w::MakingState::Made(made.clone())),
        (forge::MakingState::Adopting { id: 23, branch: b"trunk".as_slice().into() }, w::MakingState::Adopting(made)),
    ] {
        states.push(state).unwrap();
    }
    for (step, expected_step) in [
        (forge::MakeStep::Admin, w::MakeStep::Admin),
        (forge::MakeStep::Org, w::MakeStep::Org),
        (forge::MakeStep::Repository, w::MakeStep::Repository),
        (forge::MakeStep::File, w::MakeStep::File),
        (forge::MakeStep::Adoption, w::MakeStep::Adoption),
    ] {
        for (reason, expected_reason) in [
            (forge::MakeFailure::Taken, w::MakeFailure::Taken),
            (forge::MakeFailure::Empty, w::MakeFailure::Empty),
            (forge::MakeFailure::TimedOut, w::MakeFailure::TimedOut),
            (
                forge::MakeFailure::Refused { status: 422 },
                w::MakeFailure::Refused(w::Refused::new(&w::CEILINGS, w::RefusedParts { status: 422 }).unwrap()),
            ),
            (forge::MakeFailure::NotAllowed, w::MakeFailure::NotAllowed),
            (forge::MakeFailure::Other, w::MakeFailure::Other),
        ] {
            states
                .push((
                    forge::MakingState::Failed(forge::MakingFailed {
                        step,
                        reason,
                        words: b"stopped".as_slice().into(),
                    }),
                    w::MakingState::Failed(
                        w::MakingFailed::new(
                            &w::CEILINGS,
                            w::MakingFailedParts {
                                step: expected_step.clone(),
                                reason: expected_reason,
                                words: b"stopped".as_slice().into(),
                            },
                        )
                        .unwrap(),
                    ),
                ))
                .unwrap();
        }
    }
    for (state, expected) in states.into_boxed() {
        let mut row = forge_row(
            1,
            forge::Kind::Forgejo,
            forge::Mode::Owned,
            forge::Connection::Connecting,
            forge::Offer::NotOffered,
        );
        row.makings = Box::new([forge::RepositoryMaking {
            key: forge::MakingKey { forge: 1, org: b"work".as_slice().into(), name: b"copied".as_slice().into() },
            from: forge::MakeFrom::Copy { address: b"https://source.invalid/upstream/starter".as_slice().into() },
            into: Some(forge::MakeInto { project: 7, home: true, host: b"forge.invalid".as_slice().into() }),
            asked: skein_lib::Wall::EPOCH,
            copying: true,
            state,
        }]);
        let bytes = family::forges(forge::Forges { rows: Box::new([row]) }, false, &w::CEILINGS).unwrap();
        let forges = wire::decode_forges(&bytes, &w::CEILINGS).unwrap();
        let making = forges.forges().get(0).unwrap().makings().get(0).unwrap();
        assert_eq!((making.org(), making.name()), (b"work".as_slice(), b"copied".as_slice()));
        assert_eq!(
            making.from(),
            &w::MakeFrom::Copy(
                w::MakeCopy::new(
                    &w::CEILINGS,
                    w::MakeCopyParts { address: b"https://source.invalid/upstream/starter".as_slice().into() }
                )
                .unwrap()
            )
        );
        let into = making.into().as_ref().unwrap();
        assert_eq!((into.project(), into.home()), (7, true));
        assert_eq!(making.state(), &expected);
    }
}

#[test]
fn dismissed_and_trials_watches_name_the_partys_own_subjects() {
    for (watch, expected) in [
        (w::Watch::Dismissed, forge::DISMISSED_SUBJECT),
        (w::Watch::Trials, root::TRIALS_SUBJECT),
        (w::Watch::Approvals, root::APPROVALS_SUBJECT),
    ] {
        let watching = w::Watching::new(&w::CEILINGS, w::WatchingParts { watch }).unwrap();
        let event = family::to_root(
            &jig_protocol_config::Directories::working(),
            watch_exchange(wire::FORGE, wire::CLIENT_VERSION, wire::watching(&watching)),
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        )
        .unwrap();
        let root::Event::Party(root::PartyInput::Watch { watcher, subject, .. }) = event else {
            panic!("dismissed watch")
        };
        assert_eq!(
            (watcher, subject),
            (
                WATCHER,
                jig_domain_core::documents::Watched::Application {
                    scope: people::ApplicationScope::Own,
                    connector: wire::FORGE,
                    subject: Token::new(expected),
                }
            )
        );
    }
}

#[test]
fn dismissals_encode_the_partys_rows() {
    for projects in [[].as_slice(), [3, 7].as_slice()] {
        let mut rows = List::with_capacity(2);
        for project in projects {
            rows.push(forge::PersonFact::ToBegin { project: *project }).unwrap();
        }
        let rows = rows.into_boxed();
        let bytes =
            family::document(forge::Document::Dismissals(rows), &fixtures::forge_table(&[]), &w::CEILINGS).unwrap();
        let got = wire::decode_dismissals(&bytes, &w::CEILINGS).unwrap();
        let mut expected = List::with_capacity(wire::DISMISSED);
        for project in projects {
            expected
                .push(w::PersonFact::ToBegin(
                    w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: *project }).unwrap(),
                ))
                .unwrap();
        }
        assert_eq!(got, w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed: expected }).unwrap());
    }
    for subject in [
        temper_protocol_reaches::Subject::Kind(temper_protocol_reaches::Kind::Tests),
        temper_protocol_reaches::Subject::Set(6),
    ] {
        let what = forge::PersonFact::KeepAsking { project: 2, subject, at: u64::MAX };
        let bytes =
            family::document(forge::Document::Dismissals(Box::new([what])), &fixtures::forge_table(&[]), &w::CEILINGS)
                .unwrap();
        let got = wire::decode_dismissals(&bytes, &w::CEILINGS).unwrap();
        assert_eq!(got.dismissed().len(), 1);
        assert_eq!(
            got.dismissed().iter().next(),
            Some(&w::PersonFact::KeepAsking(
                w::KeepAsking::new(
                    &w::CEILINGS,
                    w::KeepAskingParts {
                        project: 2,
                        subject: temper_protocol_reaches::subject_record(subject),
                        at: u64::MAX,
                    }
                )
                .unwrap()
            ))
        );
    }
}

#[test]
fn setup_copy_addresses_are_canonicalised_before_admission() {
    let mut addresses = List::with_capacity(1);
    addresses.push(Box::from(b"source.example/owner/repo.git".as_slice())).unwrap();
    let first =
        w::SetUpFirst::Copies(w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses }).unwrap());
    let request = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::SetUp(
                w::SetUp::new(
                    &w::CEILINGS,
                    w::SetUpParts {
                        forge: w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge: 3 }).unwrap()),
                        first,
                    },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap();
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&request)),
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application {
        request: root::Application::Forge(forge::Ask::SetUp { first, .. }),
        ..
    }) = event
    else {
        panic!("setup admission")
    };
    assert_eq!(
        first,
        forge::SetUpFirst::Copies { addresses: Box::new([Box::from(b"https://source.example/owner/repo".as_slice())]) }
    );
}

#[test]
fn changes_document_encodes_and_decodes() {
    let value = forge::ChangesView {
        landed_default: Box::new([forge::ChangesLanded {
            task: 49,
            title: Box::from(b"Default landing".as_slice()),
            at: skein_lib::Wall::from_nanos(900),
            commit: [0xcd; 32],
            default_branch: true,
        }]),
        landed_default_more: 2,
        project: 7,
        landing: Box::from(b"main".as_slice()),
        more: 3,
        landed_more: 4,
        queues: Box::new([forge::ChangesQueue {
            repository: Box::from(b"acme/shop".as_slice()),
            branch: Box::from(b"main".as_slice()),
            red: true,
            changes: Box::new([
                forge::ChangesQueued {
                    task: 51,
                    title: Box::from(b"First".as_slice()),
                    ready_since: skein_lib::Wall::from_nanos(123),
                    checks_since: Some(skein_lib::Wall::from_nanos(456)),
                },
                forge::ChangesQueued {
                    task: 52,
                    title: Box::from(b"Next".as_slice()),
                    ready_since: skein_lib::Wall::from_nanos(789),
                    checks_since: None,
                },
            ]),
        }]),
        landed: Box::new([forge::ChangesLanded {
            commit: [0xab; 32],
            default_branch: true,
            task: 50,
            title: Box::from(b"Landed".as_slice()),
            at: skein_lib::Wall::from_nanos(987),
        }]),
    };
    let bytes =
        family::document(forge::Document::Changes(value.clone()), &fixtures::forge_table(&[]), &w::CEILINGS).unwrap();
    let got = wire::decode_changes_panel(&bytes, &w::CEILINGS).unwrap();
    assert_eq!((got.project(), got.landing(), got.more(), got.landed_more()), (7, b"main".as_slice(), 3, 4));
    assert_eq!(got.queues().len(), 1);
    let queue = got.queues().get(0).unwrap();
    assert_eq!((queue.repository(), queue.branch(), queue.red()), (b"acme/shop".as_slice(), b"main".as_slice(), true));
    assert_eq!(queue.changes().len(), 2);
    for (wire, domain) in queue.changes().iter().zip(value.queues[0].changes.iter()) {
        assert_eq!(
            (wire.task(), wire.title(), wire.ready_since(), *wire.checks_since()),
            (
                domain.task,
                domain.title.as_ref(),
                domain.ready_since.as_nanos(),
                match domain.checks_since {
                    Some(at) => Some(at.as_nanos()),
                    None => None,
                }
            )
        );
    }
    assert_eq!(got.landed().len(), 1);
    let landed = got.landed().get(0).unwrap();
    assert_eq!((landed.task(), landed.title(), landed.at()), (50, b"Landed".as_slice(), 987));
    assert_eq!(*landed.commit(), [0xab; 32]);
    assert!(landed.default_branch());
    assert_eq!(got.landed_default_more(), 2);
    assert_eq!(got.landed_default().len(), 1);
    let default = got.landed_default().get(0).unwrap();
    assert_eq!((default.task(), default.title(), default.at()), (49, b"Default landing".as_slice(), 900));
    assert_eq!(*default.commit(), [0xcd; 32]);
    assert!(default.default_branch());
}
#[expect(clippy::too_many_lines, reason = "one row table pairs engine and wire stops, including their annotations")]
fn ways_path() -> (ways::Path, w::BranchPath) {
    let facts = Box::new([
        ways::Fact { label: b"Model".as_slice().into(), value: b"a model".as_slice().into() },
        ways::Fact { label: b"Can".as_slice().into(), value: b"read the code".as_slice().into() },
        ways::Fact { label: b"Asked to".as_slice().into(), value: b"Make the change.".as_slice().into() },
        ways::Fact { label: b"Charter".as_slice().into(), value: b"charter 3".as_slice().into() },
    ]);
    let card = ways::Card { who: b"an agent".as_slice().into(), name: b"the coder".as_slice().into(), facts };
    let mut wire_facts = List::with_capacity(4);
    for fact in &card.facts {
        wire_facts
            .push(
                w::Fact::new(&w::CEILINGS, w::FactParts { label: fact.label.clone(), value: fact.value.clone() })
                    .unwrap(),
            )
            .unwrap();
    }
    let wire_card =
        w::Card::new(&w::CEILINGS, w::CardParts { who: card.who.clone(), name: card.name.clone(), facts: wire_facts })
            .unwrap();
    let mut stops = List::uncapped();
    let mut wire_stops = List::with_capacity(7);
    for (actor, step, words, opens, wire_actor, wire_step, wire_opens, needed, over) in [
        (
            ways::Actor::Person,
            ways::Step::Ask,
            b"you ask".as_slice(),
            ways::Opens::Nothing,
            w::Actor::Person,
            w::PathStep::Ask,
            w::Opens::Nothing,
            false,
            b"".as_slice(),
        ),
        (
            ways::Actor::Agent,
            ways::Step::Produce,
            b"produce".as_slice(),
            ways::Opens::Charter(3),
            w::Actor::Agent,
            w::PathStep::Produce,
            w::Opens::Charter(w::OpensCharter::new(&w::CEILINGS, w::OpensCharterParts { charter: 3 }).unwrap()),
            false,
            b"".as_slice(),
        ),
        (
            ways::Actor::Code,
            ways::Step::Merge,
            b"merge".as_slice(),
            ways::Opens::Queue,
            w::Actor::Code,
            w::PathStep::Merge,
            w::Opens::Queue,
            false,
            b"into goal".as_slice(),
        ),
        (
            ways::Actor::Code,
            ways::Step::Conflict,
            b"conflict".as_slice(),
            ways::Opens::Queue,
            w::Actor::Code,
            w::PathStep::Conflict,
            w::Opens::Queue,
            true,
            b"".as_slice(),
        ),
        (
            ways::Actor::Agent,
            ways::Step::Repair,
            b"repair".as_slice(),
            ways::Opens::Nothing,
            w::Actor::Agent,
            w::PathStep::Repair,
            w::Opens::Nothing,
            true,
            b"tests, review".as_slice(),
        ),
        (
            ways::Actor::Code,
            ways::Step::CatchUp,
            b"catch up".as_slice(),
            ways::Opens::Nothing,
            w::Actor::Code,
            w::PathStep::CatchUp,
            w::Opens::Nothing,
            true,
            b"".as_slice(),
        ),
        (
            ways::Actor::Bar,
            ways::Step::Branch,
            b"main".as_slice(),
            ways::Opens::Nothing,
            w::Actor::Bar,
            w::PathStep::Branch,
            w::Opens::Nothing,
            false,
            b"".as_slice(),
        ),
    ] {
        stops
            .push(ways::Stop { needed, over: over.into(), actor, step, words: words.into(), opens, card: card.clone() })
            .unwrap();
        wire_stops
            .push(
                w::Stop::new(
                    &w::CEILINGS,
                    w::StopParts {
                        needed,
                        over: over.into(),
                        actor: wire_actor,
                        step: wire_step,
                        words: words.into(),
                        opens: wire_opens,
                        card: wire_card.clone(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let repository = Some(Box::from(b"acme/shop".as_slice()));
    let branch: Box<[u8]> = Box::from(b"main".as_slice());
    (
        ways::Path { repository: repository.clone(), branch: branch.clone(), stops: stops.into_boxed() },
        w::BranchPath::new(&w::CEILINGS, w::BranchPathParts { repository, branch, stops: wire_stops }).unwrap(),
    )
}

fn ways_document(path: &ways::Path) -> ways::WaysDocument {
    let mut entries = List::uncapped();
    for (name, delegation) in
        [(b"Ask first".as_slice(), false), (b"Review first".as_slice(), true), (b"Draft first".as_slice(), false)]
    {
        entries
            .push(ways::WayEntry {
                name: name.into(),
                headline: b"A way to work.".as_slice().into(),
                line: b"How it lands".as_slice().into(),
                path: path.clone(),
                delegation,
                items: Box::new([
                    ways::Item {
                        kind: 3,
                        scope: 7,
                        number: 1,
                        edit: ways::Edit::Set(forge::settings::Value::Charter(3)),
                    },
                    ways::Item { kind: 2, scope: 7, number: 4, edit: ways::Edit::Remove },
                    ways::Item {
                        kind: 8,
                        scope: 7,
                        number: 1,
                        edit: ways::Edit::Set(forge::settings::Value::Classes(
                            temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::AgentsReview),
                        )),
                    },
                ]),
            })
            .unwrap();
    }
    ways::WaysDocument {
        project: 7,
        ways: entries.into_boxed(),
        current: Some(1),
        own: path.clone(),
        words: Some(forge::settings::ProjectWords {
            headline: b"Agents review everything, then one of you signs off".as_slice().into(),
            lede: b"Agents make and review each change; an owner approves it before it lands.".as_slice().into(),
        }),
        may_change: true,
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one mapping table checks the paths, role cards and setting items together")]
fn ways_and_view_answers_preserve_paths_cards_and_items() {
    let (path, expected) = ways_path();
    let document = ways_document(&path);
    let record = family::ways_panel(document.clone(), &w::CEILINGS).unwrap();
    let panel = wire::decode_ways_panel(&record, &w::CEILINGS).unwrap();
    assert_eq!((panel.project(), *panel.current(), panel.may_change()), (7, Some(1), true));
    assert_eq!(panel.own(), &expected);
    let words = document.words.as_ref().unwrap();
    let panel_words = panel.words().as_ref().expect("the project's words are sent");
    assert_eq!((panel_words.headline(), panel_words.lede()), (words.headline.as_ref(), words.lede.as_ref()));
    let mut without_words = document.clone();
    without_words.words = None;
    let record = family::ways_panel(without_words, &w::CEILINGS).unwrap();
    assert!(wire::decode_ways_panel(&record, &w::CEILINGS).unwrap().words().is_none());
    assert_eq!(panel.ways().len(), 3);
    for (got, entry) in panel.ways().iter().zip(document.ways.iter()) {
        assert_eq!(
            (got.name(), got.headline(), got.line(), got.delegation()),
            (entry.name.as_ref(), entry.headline.as_ref(), entry.line.as_ref(), entry.delegation)
        );
        assert_eq!(got.path(), &expected);
        assert_eq!(got.items().len(), 3);
        let set = got.items().get(0).unwrap();
        assert_eq!((set.kind(), set.scope(), set.number()), (3, 7, 1));
        let w::SettingEdit::Set(value) = set.edit() else { panic!("set item") };
        assert_eq!(value.value(), temper_engine_smith::kinds::encode(&forge::settings::Value::Charter(3)).as_ref());
        let removed = got.items().get(1).unwrap();
        assert_eq!(
            (removed.kind(), removed.scope(), removed.number(), removed.edit()),
            (2, 7, 4, &w::SettingEdit::Removed)
        );
        let classes = got.items().get(2).unwrap();
        assert_eq!((classes.kind(), classes.scope(), classes.number()), (8, 7, 1));
        let w::SettingEdit::Set(value) = classes.edit() else { panic!("classes item") };
        assert_eq!(
            value.value(),
            temper_engine_smith::kinds::encode(&forge::settings::Value::Classes(
                temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::AgentsReview),
            ))
            .as_ref()
        );
    }
    let mut after = path.clone();
    after.repository = None;
    after.branch = b"next".as_slice().into();
    let mut expected_after = expected.clone().into_parts();
    expected_after.repository = None;
    expected_after.branch = b"next".as_slice().into();
    let expected_after = w::BranchPath::new(&w::CEILINGS, expected_after).unwrap();
    let card = path.stops[1].card.clone();
    let expected_card = expected.stops().get(1).unwrap().card().clone();
    let mut roles = List::uncapped();
    let mut expected_roles = List::with_capacity(3);
    for (number, name, who) in [
        (1, b"Chat".as_slice(), b"For each chat: talks with you and asks for the changes".as_slice()),
        (12, b"Drafter", b"When you start a draft: writes a draft with you, one question at a time, and starts nothing"),
        (9, b"Status agent", b"When you look at Home after landings: writes the few lines at the top of Home, from what landed and what waits"),
    ] {
        let mut role = card.clone();
        role.name = name.into();
        role.who = who.into();
        roles.push((number, role)).unwrap();
        let mut parts = expected_card.clone().into_parts();
        parts.name = name.into();
        parts.who = who.into();
        expected_roles.push(w::CharterView::new(&w::CEILINGS, w::CharterViewParts {
            charter: number, card: w::Card::new(&w::CEILINGS, parts).unwrap(),
        }).unwrap()).unwrap();
    }
    for (view, expected_view) in [
        (
            ways::View::Proposal { before: path.clone(), after },
            w::View::Proposal(
                w::ProposalView::new(
                    &w::CEILINGS,
                    w::ProposalViewParts { before: expected.clone(), after: expected_after },
                )
                .unwrap(),
            ),
        ),
        (ways::View::Path(path.clone()), w::View::Path(expected.clone())),
        (
            ways::View::Way {
                name: b"Ask first".as_slice().into(),
                line: b"How it lands".as_slice().into(),
                path: path.clone(),
                current: true,
            },
            w::View::Way(
                w::WayView::new(
                    &w::CEILINGS,
                    w::WayViewParts {
                        name: b"Ask first".as_slice().into(),
                        line: b"How it lands".as_slice().into(),
                        path: expected.clone(),
                        current: true,
                    },
                )
                .unwrap(),
            ),
        ),
        (
            ways::View::Charter { charter: 3, card: card.clone() },
            w::View::Charter(
                w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 3, card: expected_card.clone() })
                    .unwrap(),
            ),
        ),
        (
            ways::View::Roles(roles.into_boxed()),
            w::View::Roles(w::RolesView::new(&w::CEILINGS, w::RolesViewParts { roles: expected_roles }).unwrap()),
        ),
        (
            ways::View::NotDrawn(b"No such charter".as_slice().into()),
            w::View::NotDrawn(
                w::NotDrawn::new(&w::CEILINGS, w::NotDrawnParts { words: b"No such charter".as_slice().into() })
                    .unwrap(),
            ),
        ),
    ] {
        let record = family::forge_read(root::ForgeReadAnswer::View(Box::new(view)), &[], &w::CEILINGS).unwrap();
        assert_eq!(wire::decode_view_answer(&record, &w::CEILINGS).unwrap().view(), &expected_view);
    }
}

#[test]
fn ways_and_views_refuse_over_bound_values_without_cutting_them() {
    let (path, _) = ways_path();
    for limits in [
        w::Limits { branch_path_repository_some: 8, ..w::CEILINGS },
        w::Limits { branch_path_branch: 3, ..w::CEILINGS },
        w::Limits { branch_path_stops: 3, ..w::CEILINGS },
        w::Limits { stop_words: 6, ..w::CEILINGS },
        w::Limits { card_who: 7, ..w::CEILINGS },
        w::Limits { card_name: 8, ..w::CEILINGS },
        w::Limits { card_facts: 3, ..w::CEILINGS },
        w::Limits { fact_label: 7, ..w::CEILINGS },
        w::Limits { fact_value: 15, ..w::CEILINGS },
    ] {
        assert!(family::ways_panel(ways_document(&path), &limits).is_none(), "panel: {limits:?}");
        assert!(
            family::forge_read(
                root::ForgeReadAnswer::View(Box::new(ways::View::Proposal {
                    before: path.clone(),
                    after: path.clone(),
                })),
                &[],
                &limits
            )
            .is_none(),
            "view: {limits:?}"
        );
    }
    for limits in [
        w::Limits { stock_way_name: 8, ..w::CEILINGS },
        w::Limits { stock_way_line: 11, ..w::CEILINGS },
        w::Limits { stock_way_items: 1, ..w::CEILINGS },
        w::Limits { ways_panel_ways: 2, ..w::CEILINGS },
        w::Limits { setting_value_value: 3, ..w::CEILINGS },
        w::Limits { project_words_headline: 1, ..w::CEILINGS },
        w::Limits { project_words_lede: 1, ..w::CEILINGS },
    ] {
        assert!(family::ways_panel(ways_document(&path), &limits).is_none(), "panel: {limits:?}");
    }
}

#[test]
fn view_reads_refuse_scopes_other_than_the_named_project() {
    for scope in
        [people::ApplicationScope::Deployment, people::ApplicationScope::Own, people::ApplicationScope::Project(8)]
    {
        let read = w::Read::View(
            w::ViewRead::new(
                &w::CEILINGS,
                w::ViewReadParts { project: 7, address: b"proposal t12/3".as_slice().into() },
            )
            .unwrap(),
        );
        let exchange = kit::ToApplication::Read {
            reply: REPLY,
            sign_in: 3,
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            record: wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read }).unwrap()),
            scope,
            window: jig_domain_core::documents::Window { rows: 1, after: None },
        };
        let refusal = family::to_root(
            &jig_protocol_config::Directories::working(),
            exchange,
            &fixtures::forge_table(&[]),
            None,
            &family::Deployment { welcome: false, owned: false, program_login: None },
            None,
            &w::CEILINGS,
        )
        .unwrap_err();
        let kit::Event::Application(kit::FromApplication::Read { reply, family, record }) = *refusal else {
            panic!("read refusal")
        };
        assert_eq!((reply, family, record), (REPLY, wire::FORGE, None));
    }
}

#[test]
fn the_changes_watch_names_its_project_subject() {
    for project in [0, 7, u32::MAX] {
        for (watch, expected_subject, expected_scope) in [
            (
                w::Watch::Changes(w::ChangesSubject::new(&w::CEILINGS, w::ChangesSubjectParts { project }).unwrap()),
                forge::changes_subject(project),
                people::ApplicationScope::Project(7),
            ),
            (
                w::Watch::Ways(w::WaysSubject::new(&w::CEILINGS, w::WaysSubjectParts { project }).unwrap()),
                ways::subject(project),
                people::ApplicationScope::Project(project),
            ),
        ] {
            let record = wire::watching(&w::Watching::new(&w::CEILINGS, w::WatchingParts { watch }).unwrap());
            let got = family::to_root(
                &jig_protocol_config::Directories::working(),
                watch_exchange(wire::FORGE, wire::CLIENT_VERSION, record),
                &fixtures::forge_table(&[]),
                None,
                &family::Deployment { welcome: false, owned: false, program_login: None },
                None,
                &w::CEILINGS,
            )
            .unwrap();
            let root::Event::Party(root::PartyInput::Watch {
                subject: jig_domain_core::documents::Watched::Application { scope, subject, connector },
                ..
            }) = got
            else {
                panic!("application watch")
            };
            assert_eq!(subject.raw(), expected_subject);
            assert_eq!(connector, wire::FORGE);
            assert_eq!(scope, expected_scope);
        }
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "the family test covers Looked and its status document in both directions")]
fn looked_and_status_translate_both_ways() {
    const PROJECT: u32 = 7;
    let requesting = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::Looked(w::Looked::new(&w::CEILINGS, w::LookedParts { project: PROJECT }).unwrap()),
        },
    )
    .unwrap();
    let watching = w::Watching::new(
        &w::CEILINGS,
        w::WatchingParts {
            watch: w::Watch::Status(
                w::StatusSubject::new(&w::CEILINGS, w::StatusSubjectParts { project: PROJECT }).unwrap(),
            ),
        },
    )
    .unwrap();
    let forges = fixtures::forge_table(&[]);
    let mut carried_request = exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&requesting));
    let carried = jig_domain_core::Carried { number: 2, digest: [9; 32] };
    if let kit::ToApplication::Request { carried: value, .. } = &mut carried_request {
        *value = Some(carried);
    }
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        carried_request,
        &forges,
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Application {
        scope,
        reply_to,
        sign_in,
        key,
        digest,
        carried: got,
        request,
    }) = event
    else {
        panic!("looked application")
    };
    assert_eq!(scope, people::ApplicationScope::Project(PROJECT));
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
    assert_eq!(key, REQUEST_KEY);
    assert_eq!(digest, [7; 32]);
    assert_eq!(got, Some(carried));
    assert_eq!(request, root::Application::Looked { project: PROJECT });
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        watch_exchange(wire::FORGE, wire::CLIENT_VERSION, wire::watching(&watching)),
        &forges,
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::Watch { watcher, sign_in, subject, window }) = event else {
        panic!("status watch")
    };
    assert_eq!(watcher, WATCHER);
    assert_eq!(sign_in, 3);
    assert_eq!(
        subject,
        jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(PROJECT),
            connector: wire::FORGE,
            subject: Token::new(forge::status_subject(PROJECT)),
        }
    );
    assert_eq!(window, jig_domain_core::documents::Window { rows: 2, after: None });
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    for (answer, expected) in [
        (Some(root::ApplicationAnswer::Looked(forge::LookedAnswer::Looked)), w::Outcome::Looked),
        (
            Some(root::ApplicationAnswer::Looked(forge::LookedAnswer::TooLarge)),
            w::Outcome::ForgeRefused(
                w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::TooLarge }).unwrap(),
            ),
        ),
        (None, w::Outcome::Unknown),
    ] {
        let bytes = family::answer(answer, &w::CEILINGS, &jig).unwrap();
        assert_eq!(w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome, expected);
    }
    for (last_landed, brief, previous) in [
        (None, None, None),
        (
            Some(skein_lib::Wall::from_nanos(123)),
            Some(forge::Brief {
                sentence: b"A landing".as_slice().into(),
                lede: b"The work landed.".as_slice().into(),
                at: skein_lib::Wall::from_nanos(234),
                landed: 5,
            }),
            Some(skein_lib::Wall::from_nanos(345)),
        ),
    ] {
        let value = forge::StatusView {
            project: PROJECT,
            landed: 6,
            last_landed,
            brief: brief.clone(),
            previous,
            seen: 2,
            queued: 3,
            red: true,
            branch: b"main".as_slice().into(),
        };
        let bytes = family::document(forge::Document::Status(value), &forges, &w::CEILINGS).unwrap();
        let got = wire::decode_project_status(&bytes, &w::CEILINGS).unwrap();
        assert_eq!(
            (got.project(), got.landed(), got.seen(), got.queued(), got.red(), got.branch()),
            (PROJECT, 6, 2, 3, true, b"main".as_slice())
        );
        assert_eq!(
            *got.last_landed(),
            match last_landed {
                Some(at) => Some(at.as_nanos()),
                None => None,
            }
        );
        assert_eq!(
            *got.previous(),
            match previous {
                Some(at) => Some(at.as_nanos()),
                None => None,
            }
        );
        assert_eq!(
            *got.brief(),
            match brief {
                Some(brief) => Some(
                    w::StatusBrief::new(
                        &w::CEILINGS,
                        w::StatusBriefParts {
                            sentence: brief.sentence,
                            lede: brief.lede,
                            at: brief.at.as_nanos(),
                            landed: brief.landed,
                        }
                    )
                    .unwrap()
                ),
                None => None,
            }
        );
    }
}
#[test]
fn the_worst_small_changes_document_fits_its_decision() {
    const REPOSITORY_BYTES: usize = 2048; // ChangesQueue.repository, the wire ceiling.
    let limits = crate::limits::profile(crate::limits::Profile::Small);
    let mut queues = List::with_capacity(32);
    // Each live change may have its own queue; the view retains at most 32 queues.
    for task in 0..limits.forge.changes.min(32) {
        queues
            .push(forge::ChangesQueue {
                repository: Box::from([b'r'; REPOSITORY_BYTES]),
                branch: repeated(b'b', limits.forge.name_bytes),
                red: true,
                changes: Box::new([forge::ChangesQueued {
                    task: u64::from(task),
                    title: Box::from([b't'; forge::LOGGED_TITLE_BYTES]),
                    ready_since: skein_lib::Wall::EPOCH,
                    checks_since: Some(skein_lib::Wall::EPOCH),
                }]),
            })
            .unwrap();
    }
    let mut landed = List::with_capacity(limits.forge.changes_landed);
    for task in 0..limits.forge.changes_landed {
        landed
            .push(forge::ChangesLanded {
                commit: [0xab; 32],
                default_branch: true,
                task: u64::from(task),
                title: Box::from([b't'; forge::LOGGED_TITLE_BYTES]),
                at: skein_lib::Wall::EPOCH,
            })
            .unwrap();
    }
    let document = forge::Document::Changes(forge::ChangesView {
        landed_default: landed.clone().into_boxed(),
        landed_default_more: 0,
        project: 1,
        landing: repeated(b'b', limits.forge.name_bytes),
        queues: queues.into_boxed(),
        more: 0,
        landed: landed.into_boxed(),
        landed_more: 0,
    });
    let bytes = document.owned_bytes();
    assert!(
        bytes <= usize::try_from(limits.root.journal.result_bytes.max(limits.root.journal.transcript_bytes)).unwrap()
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "price the root's worst paths and full wire rules against one decision together"
)]
fn the_worst_small_ways_document_fits_its_decision() {
    // A stock headline's bound (temper/docs/design/protocol/configuration/settings.md, 8).
    const HEADLINE_BYTES: u32 = 120;
    let limits = crate::limits::profile(crate::limits::Profile::Small);
    let worst_view = w::ViewAnswer::encoded_bound(&w::CEILINGS).unwrap();
    let delivery_bytes = limits.root.journal.result_bytes.max(limits.root.journal.transcript_bytes);
    assert!(
        worst_view <= u64::from(delivery_bytes),
        "ViewAnswer: {worst_view} encoded bytes; decision allowance: {delivery_bytes}"
    );
    let value_bytes = u32::try_from(ways::VALUE).unwrap();
    // Approval and tier stops are smaller: words and name at most 64 bytes,
    // with one fact of VALUE bytes for a tier, or two short approval facts.
    let card = ways::Card {
        who: "an agent · repairs what a check or a review found, then goes back through them".as_bytes().into(),
        name: b"the coding agent".as_slice().into(),
        facts: Box::new([
            ways::Fact { label: b"Model".as_slice().into(), value: repeated(b'm', value_bytes) },
            ways::Fact { label: b"Can".as_slice().into(), value: ways::can(true, true, true, true, false).into() },
            ways::Fact {
                label: b"Asked to".as_slice().into(),
                value: repeated(b'a', u32::try_from(ways::ASKED).unwrap()),
            },
        ]),
    };
    let mut stops = List::uncapped();
    for _ in 0..ways::STOPS {
        stops
            .push(ways::Stop {
                needed: true,
                over: b"tests, review".as_slice().into(),
                actor: ways::Actor::Agent,
                step: ways::Step::ProduceMany,
                words: "produce ×n".as_bytes().into(),
                opens: ways::Opens::Charter(u32::MAX),
                card: card.clone(),
            })
            .unwrap();
    }
    let owner = repeated(b'o', limits.forge.name_bytes);
    let name = repeated(b'n', limits.forge.name_bytes);
    let path = ways::Path {
        repository: Some(skein_lib::bytes::concat(&[&owner, b"/", &name])),
        branch: repeated(b'b', limits.forge.name_bytes),
        stops: stops.into_boxed(),
    };
    let project = 7;
    let kind7 = forge::settings::ProjectWays { drafts: true, goals: true, changes: true, way: u32::MAX };
    let mut entries = List::uncapped();
    for way in crate::stock::ways() {
        let items = ways::items(&way, project, Some(&kind7));
        entries
            .push(ways::WayEntry {
                name: way.name,
                headline: repeated(b'h', HEADLINE_BYTES),
                line: way.line,
                path: path.clone(),
                items,
                delegation: way.delegation,
            })
            .unwrap();
    }
    let document = ways::WaysDocument {
        project,
        ways: entries.into_boxed(),
        current: None,
        own: path,
        words: Some(forge::settings::ProjectWords {
            headline: repeated(b'h', forge::settings::HEADLINE_BYTES),
            lede: repeated(b'l', forge::settings::LEDE_BYTES),
        }),
        may_change: true,
    };
    let bytes = family::ways_panel(document.clone(), &w::CEILINGS).unwrap();
    let panel = wire::decode_ways_panel(&bytes, &w::CEILINGS).unwrap();
    assert!(panel.rules().rows().is_empty());
    assert!(panel.rules().note().is_empty());
    // Price the root's worst panel with every wire rule bound filled.
    let row = w::PathRule::new(
        &w::CEILINGS,
        w::PathRuleParts {
            paths: repeated(b'p', w::CEILINGS.path_rule_paths),
            review: repeated(b'r', w::CEILINGS.path_rule_review),
            sign_off: repeated(b's', w::CEILINGS.path_rule_sign_off),
            lands: repeated(b'l', w::CEILINGS.path_rule_lands),
        },
    )
    .unwrap();
    let mut rows = List::with_capacity(w::CEILINGS.path_rules_rows);
    for _ in 0..w::CEILINGS.path_rules_rows {
        rows.push(row.clone()).unwrap();
    }
    let rules =
        w::PathRules::new(&w::CEILINGS, w::PathRulesParts { rows, note: repeated(b'n', w::CEILINGS.path_rules_note) })
            .unwrap();
    let panel = w::WaysPanel::new(&w::CEILINGS, w::WaysPanelParts { rules, ..panel.into_parts() }).unwrap();
    let bytes = wire::ways_panel(&panel);
    assert_eq!(wire::decode_ways_panel(&bytes, &w::CEILINGS), Some(panel));
    let mut journal = root::Journal::uncapped(1, 0);
    let mut decision = root::Decision::decide(&mut journal, &limits.root.journal);
    decision
        .deliver(&limits.root.journal, root::Delivery::Ways { watcher: WATCHER, document: Box::new(document) })
        .unwrap();
    let delivery_bytes = limits.root.journal.result_bytes.max(limits.root.journal.transcript_bytes);
    assert!(
        bytes.len() <= usize::try_from(delivery_bytes).unwrap(),
        "WaysPanel: {} encoded bytes; decision allowance: {delivery_bytes}",
        bytes.len()
    );
}

fn repeated(byte: u8, count: u32) -> Box<[u8]> {
    let mut bytes = List::with_capacity(count);
    for _ in 0..count {
        bytes.push(byte).unwrap();
    }
    bytes.into_boxed()
}

#[test]
fn a_whole_party_document_fits_the_wire() {
    let mut rows = List::with_capacity(wire::DISMISSED);
    for goal in 0..u64::from(wire::DISMISSED) {
        rows.push(forge::PersonFact::GoalFrom { project: 1, chat: 7, goal, at: 0 }).unwrap();
    }
    let bytes =
        family::document(forge::Document::Dismissals(rows.into_boxed()), &fixtures::forge_table(&[]), &w::CEILINGS)
            .unwrap();
    let document = wire::decode_dismissals(&bytes, &w::CEILINGS).unwrap();
    assert_eq!(document.dismissed().len(), wire::DISMISSED);
    for (goal, fact) in document.dismissed().iter().enumerate() {
        let w::PersonFact::GoalFrom(value) = fact else { panic!("goal fact") };
        assert_eq!((value.project(), value.chat(), value.goal()), (1, 7, u64::try_from(goal).unwrap()));
    }
}
fn away_family(exchange: kit::ToApplication) -> root::Event {
    family::to_root(
        &jig_protocol_config::Directories::working(),
        exchange,
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap()
}
fn away_request(request: w::Request) -> root::Application {
    let requesting = w::Requesting::new(&w::CEILINGS, w::RequestingParts { request }).unwrap();
    let root::Event::Party(root::PartyInput::Application { scope, reply_to, sign_in, key, digest, carried, request }) =
        away_family(exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&requesting)))
    else {
        panic!("a typed application")
    };
    assert_eq!(scope, people::ApplicationScope::Project(7));
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
    assert_eq!(key, REQUEST_KEY);
    assert_eq!(digest, [7; 32]);
    assert_eq!(carried, None);
    request
}
fn away_wall(nanos: Option<u64>) -> Option<skein_lib::Wall> {
    match nanos {
        Some(nanos) => Some(skein_lib::Wall::from_nanos(nanos)),
        None => None,
    }
}
fn away_answer(answer: Option<root::ApplicationAnswer>) -> w::Outcome {
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small).clients.wire;
    let bytes = family::answer(answer, &w::CEILINGS, &jig).unwrap();
    w::Answer::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts().outcome
}
#[test]
fn set_away_reads_as_the_forges_event() {
    const PROJECT: u32 = 9; // Deliberately differs from the envelope's project 7.
    for (on, until) in [(true, Some(123)), (false, None)] {
        assert_eq!(
            away_request(w::Request::SetAway(
                w::SetAway::new(&w::CEILINGS, w::SetAwayParts { project: PROJECT, on, until },).unwrap()
            )),
            root::Application::SetAway { project: PROJECT, on, until: away_wall(until) }
        );
    }
    for (answer, expected) in [
        (Some(root::ApplicationAnswer::SetAway(forge::SetAwayAnswer::Away)), w::Outcome::Away),
        (
            Some(root::ApplicationAnswer::SetAway(forge::SetAwayAnswer::TooLarge)),
            w::Outcome::ForgeRefused(
                w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason: w::ForgeReason::TooLarge }).unwrap(),
            ),
        ),
        (None, w::Outcome::Unknown),
    ] {
        assert_eq!(away_answer(answer), expected);
    }
}
#[test]
fn overturn_reads_as_the_forges_event() {
    const PROJECT: u32 = 9;
    const NUMBER: u32 = 12;
    assert_eq!(
        away_request(w::Request::Overturn(
            w::Overturn::new(&w::CEILINGS, w::OverturnParts { project: PROJECT, number: NUMBER },).unwrap()
        )),
        root::Application::Overturn { project: PROJECT, number: NUMBER }
    );
    for (answer, expected) in [
        (Some(root::ApplicationAnswer::Overturn(forge::OverturnAnswer::Overturned)), w::Outcome::Overturned),
        (
            Some(root::ApplicationAnswer::Overturn(forge::OverturnAnswer::Already)),
            w::Outcome::OverturnRefused(
                w::OverturnRefused::new(&w::CEILINGS, w::OverturnRefusedParts { reason: w::OverturnRefusal::Already })
                    .unwrap(),
            ),
        ),
        (
            Some(root::ApplicationAnswer::Overturn(forge::OverturnAnswer::Unknown)),
            w::Outcome::OverturnRefused(
                w::OverturnRefused::new(&w::CEILINGS, w::OverturnRefusedParts { reason: w::OverturnRefusal::Unknown })
                    .unwrap(),
            ),
        ),
        (None, w::Outcome::Unknown),
    ] {
        assert_eq!(away_answer(answer), expected);
    }
}
#[test]
fn away_and_decided_subjects_are_scoped() {
    const PROJECT: u32 = 9;
    for (watch, scope, subject) in [
        (w::Watch::Away, people::ApplicationScope::Own, forge::AWAY_SUBJECT),
        (
            w::Watch::Decided(
                w::DecidedSubject::new(&w::CEILINGS, w::DecidedSubjectParts { project: PROJECT }).unwrap(),
            ),
            people::ApplicationScope::Project(PROJECT),
            forge::decided_subject(PROJECT),
        ),
    ] {
        let watching = w::Watching::new(&w::CEILINGS, w::WatchingParts { watch }).unwrap();
        let root::Event::Party(root::PartyInput::Watch { watcher, sign_in, subject: got, window }) =
            away_family(watch_exchange(wire::FORGE, wire::CLIENT_VERSION, wire::watching(&watching)))
        else {
            panic!("an application watch")
        };
        assert_eq!(watcher, WATCHER);
        assert_eq!(sign_in, 3);
        assert_eq!(
            got,
            jig_domain_core::documents::Watched::Application {
                scope,
                connector: wire::FORGE,
                subject: Token::new(subject),
            }
        );
        assert_eq!(window, jig_domain_core::documents::Window { rows: 2, after: None });
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table checks Away and both decision lists through the family encoder")]
fn their_documents_encode() {
    const PROJECT: u32 = 9;
    const PARTY: u64 = 3;
    for (until, off) in [(None, None), (Some(22), Some(33))] {
        let row = forge::Away {
            party: PARTY,
            project: PROJECT,
            since: skein_lib::Wall::from_nanos(11),
            until: away_wall(until),
            landed: 4,
            off: away_wall(off),
        };
        let bytes = family::document(
            forge::Document::Aways(forge::AwaysView { rows: Box::new([row]) }),
            &fixtures::forge_table(&[]),
            &w::CEILINGS,
        )
        .unwrap();
        let got = wire::decode_aways(&bytes, &w::CEILINGS).unwrap().into_parts();
        assert_eq!(
            got.rows.as_slice(),
            &[w::AwayRow::new(&w::CEILINGS, w::AwayRowParts { project: PROJECT, since: 11, until, landed: 4, off })
                .unwrap()]
        );
    }
    for overturned in [false, true] {
        let row = forge::DecisionView {
            number: 12,
            task: 51,
            at: skein_lib::Wall::from_nanos(44),
            question: b"Which?".as_slice().into(),
            took: b"First".as_slice().into(),
            why: b"Fits".as_slice().into(),
            overturned,
            reach: temper_protocol_reaches::Reach {
                kind: temper_protocol_reaches::Kind::Wording,
                class: temper_protocol_reaches::Class::Decide,
                reason: temper_protocol_reaches::Reason::Kind,
                under: 0,
                sets: 0,
                matched: 0,
                away: false,
            },
        };
        let bytes = family::document(
            forge::Document::Decisions(forge::DecisionsView {
                project: PROJECT,
                rows: Box::new([row]),
                asked: Box::new([forge::AskedView {
                    number: 13,
                    task: 52,
                    at: skein_lib::Wall::from_nanos(45),
                    reach: temper_protocol_reaches::Reach {
                        kind: temper_protocol_reaches::Kind::Wording,
                        class: temper_protocol_reaches::Class::Decide,
                        reason: temper_protocol_reaches::Reason::Kind,
                        under: 0,
                        sets: 0,
                        matched: 0,
                        away: false,
                    },
                }]),
                cut: overturned,
            }),
            &fixtures::forge_table(&[]),
            &w::CEILINGS,
        )
        .unwrap();
        let got = wire::decode_decisions(&bytes, &w::CEILINGS).unwrap().into_parts();
        assert_eq!(got.project, PROJECT);
        assert_eq!(got.cut, overturned);
        assert_eq!(got.asked.len(), 1);
        assert_eq!(
            (got.asked.as_slice()[0].number(), got.asked.as_slice()[0].task(), got.asked.as_slice()[0].at()),
            (13, 52, 45)
        );
        assert_eq!(
            got.asked.as_slice()[0].reach(),
            &temper_protocol_reaches::reach_record(&temper_protocol_reaches::Reach {
                kind: temper_protocol_reaches::Kind::Wording,
                class: temper_protocol_reaches::Class::Decide,
                reason: temper_protocol_reaches::Reason::Kind,
                under: 0,
                sets: 0,
                matched: 0,
                away: false
            })
        );
        assert_eq!(
            got.rows.as_slice(),
            &[w::Decision::new(
                &w::CEILINGS,
                w::DecisionParts {
                    number: 12,
                    task: 51,
                    at: 44,
                    question: b"Which?".as_slice().into(),
                    took: b"First".as_slice().into(),
                    why: b"Fits".as_slice().into(),
                    overturned,
                    reach: w::Reach::new(
                        &w::CEILINGS,
                        w::ReachParts {
                            kind: w::ReachKind::Wording,
                            class: w::ReachClass::Decide,
                            reason: w::ReachReason::Kind,
                            under: 0,
                            sets: 0,
                            matched: 0,
                            away: false
                        }
                    )
                    .unwrap(),
                }
            )
            .unwrap()]
        );
    }
    let forges = fixtures::forge_table(&[]);
    let bytes =
        family::document(forge::Document::Aways(forge::AwaysView { rows: Box::new([]) }), &forges, &w::CEILINGS)
            .unwrap();
    assert!(wire::decode_aways(&bytes, &w::CEILINGS).unwrap().rows().is_empty());
    let bytes = family::document(
        forge::Document::Decisions(forge::DecisionsView {
            project: PROJECT,
            rows: Box::new([]),
            asked: Box::new([]),
            cut: false,
        }),
        &forges,
        &w::CEILINGS,
    )
    .unwrap();
    let got = wire::decode_decisions(&bytes, &w::CEILINGS).unwrap();
    assert_eq!(got.project(), PROJECT);
    assert!(got.rows().is_empty());
}

#[test]
fn the_deployment_command_is_published_only_by_a_standalone_start() {
    for (standalone, under_box) in [(true, false), (true, true), (false, false)] {
        let document = if standalone { fixtures::document() } else { fixtures::engine_alone_document() };
        let line = b"'/box/temper' program-login --config '/box/config' --data '/box/data' --cache '/box/cache'";
        let agent = fixtures::startup_agent();
        let service = crate::startup::with_setup_given(
            &document,
            &jig_protocol_config::Directories::working(),
            &fixtures::files(),
            if standalone { Some(&agent) } else { None },
            &fixtures::seeds(),
            None,
            if standalone { Some(fixtures::worker_runtime()) } else { None },
            fixtures::forge_runtime(),
            None,
            if under_box { Some(fixtures::system::BOX_BYTES) } else { None },
            Some(line.as_slice().into()),
            &crate::stock::Given::Stock,
            Box::new([]),
        )
        .unwrap();
        let deployment = &service.engine.as_ref().unwrap().clients.deployment;
        let mut input = read_input(people::ApplicationScope::Deployment);
        if let kit::ToApplication::Read { record, .. } = &mut input {
            *record =
                wire::reading(&w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Deployment }).unwrap());
        }
        let Err(answer) = family::to_root(
            &jig_protocol_config::Directories::working(),
            input,
            &fixtures::forge_table(&[]),
            None,
            deployment,
            None,
            &w::CEILINGS,
        ) else {
            panic!("deployment read")
        };
        let kit::Event::Application(kit::FromApplication::Read { record, .. }) = *answer else {
            panic!("deployment answer")
        };
        let answer = wire::decode_deployment_answer(&record.unwrap(), &w::CEILINGS).unwrap();
        assert_eq!(answer.program_login().as_deref(), if standalone { Some(line.as_slice()) } else { None });
    }
}
#[test]
fn the_program_environment_read_goes_to_the_party() {
    let mut input = read_input(people::ApplicationScope::Deployment);
    if let kit::ToApplication::Read { record, sign_in, .. } = &mut input {
        *record = wire::reading(
            &w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::ProgramEnvironment }).unwrap(),
        );
        *sign_in = 3;
    }
    let event = family::to_root(
        &jig_protocol_config::Directories::working(),
        input,
        &fixtures::forge_table(&[]),
        None,
        &family::Deployment { welcome: false, owned: false, program_login: None },
        None,
        &w::CEILINGS,
    )
    .unwrap();
    let root::Event::Party(root::PartyInput::ForgeRead { reply, sign_in, scope, query }) = event else {
        panic!("party gates the environment")
    };
    assert_eq!(
        (reply, sign_in, scope, query),
        (REPLY, 3, people::ApplicationScope::Deployment, root::ForgeRead::ProgramEnvironment)
    );
}
#[test]
fn forge_read_encodes_the_environment() {
    let words = b"TEMPER_CONFIG='/config' TEMPER_DATA='/data' TEMPER_CACHE='/cache'";
    let bytes = family::forge_read(root::ForgeReadAnswer::ProgramEnvironment, words, &w::CEILINGS).unwrap();
    assert_eq!(wire::decode_program_environment_answer(&bytes, &w::CEILINGS).unwrap().environment(), words);
}

#[test]
fn trying_an_account_preserves_its_fields_and_uses_own_scope() {
    for again in [false, true] {
        let record = w::Requesting::new(
            &w::CEILINGS,
            w::RequestingParts {
                request: w::Request::TryAccount(
                    w::TryAccount::new(&w::CEILINGS, w::TryAccountParts { account: u32::MAX, again }).unwrap(),
                ),
            },
        )
        .unwrap();
        let event = away_family(exchange(wire::FORGE, wire::CLIENT_VERSION, wire::requesting(&record)));
        let root::Event::Party(root::PartyInput::Application {
            reply_to,
            sign_in,
            key,
            digest,
            scope,
            carried,
            request,
        }) = event
        else {
            panic!("trial application")
        };
        assert_eq!(
            (reply_to, sign_in, key, digest, scope, carried),
            (REPLY, 3, REQUEST_KEY, [7; 32], people::ApplicationScope::Own, None)
        );
        assert_eq!(request, root::Application::Try { account: u32::MAX, again });
    }
}

#[test]
fn trials_encode_the_whole_document_in_its_order_and_with_its_bounds() {
    use skein_lib::{Reader, Wall};
    use temper_engine_domain_trials::{Class, Row, State, Step, Why};
    let states = [
        State::Running,
        State::Answered { seconds: 3 },
        State::Failed { why: Why::Refused },
        State::Failed { why: Why::Waited },
        State::Failed { why: Why::Ended { class: Class::Transient } },
        State::Failed { why: Why::Ended { class: Class::Permanent } },
        State::Failed { why: Why::Ended { class: Class::Run } },
        State::Failed { why: Why::Ended { class: Class::Agent } },
        State::Failed { why: Why::Ended { class: Class::Lost } },
        State::Failed { why: Why::Ended { class: Class::Invalid } },
        State::Failed { why: Why::Ended { class: Class::Refused } },
    ];
    let expected = [
        w::TrialState::Running,
        w::TrialState::Answered(w::TrialAnswered::new(&w::CEILINGS, w::TrialAnsweredParts { seconds: 3 }).unwrap()),
        w::TrialState::Failed(
            w::TrialFailed::new(&w::CEILINGS, w::TrialFailedParts { why: w::TrialWhy::Refused }).unwrap(),
        ),
        w::TrialState::Failed(
            w::TrialFailed::new(&w::CEILINGS, w::TrialFailedParts { why: w::TrialWhy::Waited }).unwrap(),
        ),
    ];
    for (index, state) in states.into_iter().enumerate() {
        let rows = Box::new([
            Row {
                account: 12,
                person: 7,
                task: Some(31),
                began: Wall::EPOCH,
                took: Some(14),
                step: Step::Answers,
                state,
            },
            Row {
                account: 11,
                person: 7,
                task: None,
                began: Wall::EPOCH,
                took: None,
                step: Step::Starts,
                state: State::Running,
            },
        ]);
        let bytes = family::trials(rows, &w::CEILINGS).expect("whole trial document");
        let mut reader = Reader::new(&bytes);
        let document = w::Trials::decode(&w::CEILINGS, &mut reader).unwrap();
        assert!(reader.is_empty());
        let rows = document.into_parts().rows.into_boxed();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].account(), 12);
        assert_eq!(*rows[0].task(), Some(31));
        assert_eq!(*rows[0].took(), Some(14));
        assert_eq!(*rows[0].step(), w::TrialStep::Answers);
        assert_eq!(rows[1].account(), 11);
        assert_eq!(*rows[1].task(), None);
        assert_eq!(*rows[1].took(), None);
        assert_eq!(*rows[1].step(), w::TrialStep::Starts);
        if index < expected.len() {
            assert_eq!(rows[0].state(), &expected[index]);
        } else {
            let w::TrialState::Failed(failed) = rows[0].state() else { panic!("failed row") };
            let w::TrialWhy::Ended(ended) = failed.why() else { panic!("run ending") };
            let classes = [
                w::TrialClass::Transient,
                w::TrialClass::Permanent,
                w::TrialClass::Run,
                w::TrialClass::Agent,
                w::TrialClass::Lost,
                w::TrialClass::Invalid,
                w::TrialClass::Refused,
            ];
            assert_eq!(*ended.class(), classes[index - expected.len()]);
        }
        let bounds = w::Limits { trials_rows: 0, ..w::CEILINGS };
        assert!(
            family::trials(
                Box::new([Row {
                    account: 1,
                    person: 7,
                    task: None,
                    began: Wall::EPOCH,
                    took: None,
                    step: Step::Starts,
                    state,
                }]),
                &bounds
            )
            .is_none()
        );
    }
}

#[test]
fn approvals_encode_each_row_with_its_kind_and_files_address() {
    let forgejo = fixtures::forges().first().unwrap().clone();
    let mut github = forgejo.clone();
    github.number = 2;
    github.kind = crate::settings::Kind::GitHub;
    github.git = b"https://api.github.example".as_slice().into();
    github.web = Some(b"https://github.example".as_slice().into());
    let mut fake = forgejo.clone();
    fake.number = 3;
    fake.mode = forge::Mode::Fake;
    let forges = fixtures::forge_table(&[forgejo, github, fake]);
    let cases = [
        (
            1,
            root::YourReview::Counts,
            w::Kind::Forgejo,
            w::ApprovalYours::Counts,
            b"https://forge.example/org/repo/pulls/5/files".as_slice(),
        ),
        (
            2,
            root::YourReview::NoAccount,
            w::Kind::Github,
            w::ApprovalYours::NoAccount,
            b"https://github.example/org/repo/pull/5/files".as_slice(),
        ),
        (3, root::YourReview::TooFew, w::Kind::Forgejo, w::ApprovalYours::TooFew, b"".as_slice()),
        (4, root::YourReview::OtherRole, w::Kind::Forgejo, w::ApprovalYours::OtherRole, b"".as_slice()),
    ];
    let mut rows = List::with_capacity(4);
    for &(forge, yours, ..) in &cases {
        rows.push(root::ApprovalRow {
            task: u64::from(forge),
            project: 7,
            goal: Some(8),
            forge,
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into(),
            pull: 5,
            title: b"Change".as_slice().into(),
            since: skein_lib::Wall::from_nanos(9),
            scope: 7,
            rule: 10,
            role: 1,
            required: 2,
            counted: 1,
            sets: Box::new([root::ApprovalSet { number: 1, name: b"vsr's protocol core".as_slice().into() }]),
            whole: false,
            yours,
        })
        .unwrap();
    }
    let rows = rows.into_boxed();
    let document = root::Approvals { rows, more: 6 };
    let bytes = family::approvals(document.clone(), &forges, &w::CEILINGS).unwrap();
    let decoded = wire::decode_approvals(&bytes, &w::CEILINGS).unwrap();
    assert_eq!(decoded.more(), 6);
    assert_eq!(decoded.rows().len(), 4);
    for ((actual, row), (_, _, kind, yours, address)) in decoded.rows().iter().zip(document.rows.iter()).zip(&cases) {
        assert_eq!(actual.task(), row.task);
        assert_eq!(actual.project(), row.project);
        assert_eq!(*actual.goal(), row.goal);
        assert_eq!(actual.forge(), row.forge);
        assert_eq!(actual.kind(), kind);
        assert_eq!(actual.owner(), row.owner.as_ref());
        assert_eq!(actual.name(), row.name.as_ref());
        assert_eq!(actual.pull(), row.pull);
        assert_eq!(actual.address(), *address);
        assert_eq!(actual.title(), row.title.as_ref());
        assert_eq!(actual.since(), row.since.as_nanos());
        assert_eq!((actual.scope(), actual.rule(), actual.role()), (row.scope, row.rule, row.role));
        assert_eq!((actual.required(), actual.counted()), (row.required, row.counted));
        assert_eq!(actual.sets().len(), 1);
        assert_eq!(actual.sets().as_slice()[0].number(), row.sets[0].number);
        assert_eq!(actual.sets().as_slice()[0].name(), row.sets[0].name.as_ref());
        assert_eq!(actual.whole(), row.whole);
        assert_eq!(actual.yours(), yours);
    }
    for limits in [w::Limits { approvals_rows: 3, ..w::CEILINGS }, w::Limits { approval_row_title: 0, ..w::CEILINGS }] {
        assert!(family::approvals(document.clone(), &forges, &limits).is_none());
    }
    let mut forges = forges;
    for (length, expected) in [(2048, 2048), (2049, 0)] {
        let length = length - b"/org/repo/pulls/5/files".len();
        let prefix = b"https://forge.example/";
        let mut base = skein_lib::Writer::new(length);
        base.put(prefix).unwrap();
        for _ in prefix.len()..length {
            base.put(b"x").unwrap();
        }
        forges.get_mut(1).unwrap().git = base.finish();
        let bytes = family::approvals(document.clone(), &forges, &w::CEILINGS).unwrap();
        let decoded = wire::decode_approvals(&bytes, &w::CEILINGS).unwrap();
        assert_eq!(decoded.rows().as_slice()[0].address().len(), expected);
    }
}
