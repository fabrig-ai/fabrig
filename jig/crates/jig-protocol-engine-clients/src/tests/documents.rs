use super::fixtures;
use crate::translate;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::List;
fn me_preserves_two_projects_in_its_wire_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::owner::frame(&fixtures::frame_two_projects(), fixtures::SIGN_IN_EXPIRES, &[], &limits),
        Ok(fixtures::frame_two_projects_record())
    );
}
#[test]
fn deployment_census_reaches_the_client_at_all_its_bounds_and_when_absent() {
    use jig_domain_core_vocabulary::documents::deployment::{
        CENSUS_HEADS, CENSUS_NAME, CENSUS_PREFIX, CENSUS_ROWS, CensusHead, CensusKeyspace, StoreCensus,
    };
    let mut keyspaces = List::with_capacity(u32::try_from(CENSUS_ROWS).unwrap());
    for n in 0..CENSUS_ROWS {
        let mut heads = List::with_capacity(u32::try_from(CENSUS_HEADS).unwrap());
        for h in 0..CENSUS_HEADS {
            heads
                .push(CensusHead {
                    head: if h == 0 { None } else { Some(u16::try_from(h).unwrap()) },
                    rows: u64::try_from(h).unwrap() + 10,
                })
                .unwrap();
        }
        keyspaces
            .push(CensusKeyspace {
                prefix: Box::new([u8::try_from(n).unwrap(); CENSUS_PREFIX]),
                name: "n".repeat(CENSUS_NAME).into_boxed_str(),
                rows: u64::try_from(n).unwrap(),
                keys: 11,
                values: 13,
                heads,
                puts: 17,
                erases: 19,
                written: u64::MAX,
            })
            .unwrap();
    }
    let census = Box::new(StoreCensus { commit: u64::MAX, since: 679_000, keyspaces });
    for census in [None, Some(census)] {
        let mut source = fixtures::deployment_document();
        source.census = census.clone();
        let wire = translate::deployment::document(&source, &w::CEILINGS).unwrap();
        let mut bytes = skein_lib::Writer::new(
            usize::try_from(wire.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
        );
        wire.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
        let bytes = bytes.finish();
        let limits = fixtures::client_limits();
        assert!(u64::try_from(bytes.len()).unwrap() <= limits.client().document_bytes);
        let wire = w::Deployment::decode(limits.receiving(), &mut skein_lib::Reader::new(&bytes)).unwrap();
        let client = jig_client_protocol_link::translate::documents::deployment(wire, &limits).unwrap();
        match (census, client.census) {
            (None, None) => (),
            (Some(expected), Some(actual)) => {
                assert_eq!((actual.commit, actual.since), (expected.commit, expected.since));
                assert_eq!(actual.keyspaces.len(), expected.keyspaces.len());
                for (a, e) in actual.keyspaces.iter().zip(expected.keyspaces.iter()) {
                    assert_eq!(
                        (&a.prefix, &a.name, a.rows, a.keys, a.values, a.puts, a.erases, a.written),
                        (&e.prefix, &e.name, e.rows, e.keys, e.values, e.puts, e.erases, e.written)
                    );
                    assert_eq!(a.heads.len(), e.heads.len());
                    for (a, e) in a.heads.iter().zip(e.heads.iter()) {
                        assert_eq!((a.head, a.rows), (e.head, e.rows));
                    }
                }
            }
            _ => panic!("census presence preserved"),
        }
    }
}

const PROJECT_ROOM: [u32; 2] = [2, 32];
#[test]
fn me_encoding_preserves_rows_despite_spare_list_capacity() {
    let limits = fixtures::wire_limits();
    let expected =
        translate::owner::frame(&frame_with_creating_bounds(), fixtures::SIGN_IN_EXPIRES, &[], &limits).unwrap();
    for project_room in PROJECT_ROOM {
        let mut frame = frame_with_creating_bounds();
        let mut projects = List::with_capacity(project_room);
        for mut project in frame.projects.into_boxed() {
            if let Some(creating) = &mut project.creating
                && let jig_domain_core_vocabulary::acts::Bounds::Create { authority, .. } = &mut creating.within
            {
                let mut kinds = List::with_capacity(64);
                for kind in &authority.executors {
                    kinds.push(*kind).expect("stock executor room");
                }
                authority.executors = kinds;
            }
            projects.push(project).expect("fixture project room");
        }
        frame.projects = projects;
        assert_eq!(
            translate::owner::frame(&frame, fixtures::SIGN_IN_EXPIRES, &[], &limits),
            Ok(expected.clone()),
            "projects room {project_room}"
        );
    }
}
fn a_task_with_a_hold_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::task::document(&fixtures::task_with_hold(), &[], &limits),
        Ok(jig_client_test_records::task_with_hold_record())
    );
}
fn a_chip_in_each_phase_becomes_its_record() {
    let limits = fixtures::wire_limits();
    let phases = fixtures::phases();
    let records = jig_client_test_records::owner_chip_records();
    let source = fixtures::chip();
    for ((phase, _compact), record) in phases.into_iter().zip(records) {
        let mut chip = source.clone();
        chip.phase = phase;
        assert_eq!(translate::chips::chip(&chip, &limits), Ok(record));
    }
}
#[test]
fn me_overflow_reports_the_wire_bound_it_exceeds() {
    let frame = fixtures::frame_two_projects();
    let limits = fixtures::wire_limits();
    let rows = [
        (w::Limits { me_projects: 1, ..limits }, w::paths::ME_PROJECTS),
        (w::Limits { document_text_bytes: 1, ..limits }, w::paths::DOCUMENT_TEXT_BYTES),
    ];
    for (limits, path) in rows {
        assert_eq!(
            translate::owner::frame(&frame, fixtures::SIGN_IN_EXPIRES, &[], &limits),
            Err(w::Problem { path, reason: skein_codec::Reason::Bound })
        );
    }
}

#[test]
fn a_non_utf8_document_text_fails_translation() {
    let limits = fixtures::wire_limits();
    for bytes in [&[0xc3_u8][..], &[255_u8, b'/'][..]] {
        let value = jig_domain_core_vocabulary::documents::Text { bytes: bytes.into(), cut: true };
        assert_eq!(
            translate::owner::text(&value).encode(&limits),
            Err(w::Problem { path: w::paths::DOCUMENT_TEXT_BYTES, reason: skein_codec::Reason::Utf8 })
        );
    }
}

fn a_run_writing_with_a_call_becomes_its_run_state() {
    let source = fixtures::run_writing();
    assert_eq!(translate::live::run(&source), Ok(jig_client_test_records::run_writing_record()));
}
fn a_run_text_keeps_its_turn_retry_and_offsets() {
    assert_eq!(translate::live::writing(&fixtures::writing()), Ok(jig_client_test_records::writing_record()));
}
fn an_inbox_with_both_groups_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::inbox::document(&fixtures::inbox_document(), &limits),
        Ok(jig_client_test_records::inbox_both_groups_record())
    );
}
#[test]
fn an_inbox_entry_33_goals_deep_encodes_at_the_wire_ceiling() {
    use skein_lib::{Reader, Writer};
    // The standard task depth plus the task itself (client/domain/pages.md, 3).
    const STANDARD_GOALS: u32 = 33;
    let mut source = fixtures::inbox_document();
    let selection = source.selection.as_mut().unwrap().get_mut(0).unwrap();
    selection.goals = List::with_capacity(STANDARD_GOALS);
    for goal in 1..=u64::from(STANDARD_GOALS) {
        selection.goals.push(goal).unwrap();
    }
    let expected = selection.clone();
    let record = translate::inbox::document(&source, &w::CEILINGS).unwrap();
    let mut encoded = Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut encoded).unwrap();
    let bytes = encoded.finish();
    let record = w::Inbox::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    let mut selections = Reader::new(record.selection.as_ref().unwrap());
    let bytes = skein_codec::piece::split(&mut selections).unwrap().unwrap();
    let selection = w::InboxSelection::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert_eq!((&selection.cursor), &translate::inbox::cursor(expected.cursor));
    assert_eq!((&selection.goals), &expected.goals);
    assert!(selections.is_empty());
}
fn a_deployment_document_becomes_its_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::deployment::document(&fixtures::deployment_document(), &limits),
        Ok(jig_client_test_records::deployment_rows_record())
    );
}
fn a_lines_document_becomes_its_tasks_rows_record() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::queries::lines(&fixtures::tasks_document(), &List::with_capacity(0), &limits),
        Ok(jig_client_test_records::tasks_rows_record())
    );
}
fn a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor() {
    let limits = fixtures::wire_limits();
    assert_eq!(
        translate::live::tree(&fixtures::tree_document(), Some(jig_client_test_records::live_cursor_record()), &limits),
        Ok(jig_client_test_records::tree_rows_record())
    );
}
#[test]
fn live_nested_pieces_past_the_wire_bounds_are_refused_naming_them() {
    let source = fixtures::deployment_document();
    let limits = fixtures::wire_limits();
    let rows = [
        ("host runs", w::Limits { host_runs: 0, ..limits }, w::paths::HOST_RUNS),
        ("deployment hosts", w::Limits { deployment_hosts: 0, ..limits }, w::paths::DEPLOYMENT_HOSTS),
        ("deployment attempts", w::Limits { deployment_attempts: 0, ..limits }, w::paths::DEPLOYMENT_ATTEMPTS),
        ("account runs", w::Limits { account_runs: 0, ..limits }, w::paths::ACCOUNT_RUNS),
        ("account waiting", w::Limits { account_waiting: 0, ..limits }, w::paths::ACCOUNT_WAITING),
        ("deployment accounts", w::Limits { deployment_accounts: 0, ..limits }, w::paths::DEPLOYMENT_ACCOUNTS),
    ];
    for (label, limits, path) in rows {
        assert_eq!(
            translate::deployment::document(&source, &limits),
            Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
            "{label}"
        );
    }
}

#[test]
fn documents_become_their_literal_records() {
    table([
        ("me_preserves_two_projects_in_its_wire_record", me_preserves_two_projects_in_its_wire_record),
        ("a_task_with_a_hold_becomes_its_record", a_task_with_a_hold_becomes_its_record),
        ("a_chip_in_each_phase_becomes_its_record", a_chip_in_each_phase_becomes_its_record),
        ("a_run_writing_with_a_call_becomes_its_run_state", a_run_writing_with_a_call_becomes_its_run_state),
        ("a_run_text_keeps_its_turn_retry_and_offsets", a_run_text_keeps_its_turn_retry_and_offsets),
        ("an_inbox_with_both_groups_becomes_its_record", an_inbox_with_both_groups_becomes_its_record),
        ("a_deployment_document_becomes_its_record", a_deployment_document_becomes_its_record),
        ("a_lines_document_becomes_its_tasks_rows_record", a_lines_document_becomes_its_tasks_rows_record),
        (
            "a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor",
            a_tree_keeps_its_rows_folds_dependencies_and_opaque_cursor,
        ),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    skein_rows::check(cases, |case| case());
}

const TEMPLATE: u32 = 2;
const POOL_LEFT: u64 = 70;
const BUDGET: u64 = 50;
const TEMPLATE_CHARTER: u32 = 1;
const TEMPLATE_HOLDER_ROLE: u32 = 0;
#[test]
fn a_me_document_carries_each_project_s_template() {
    let mut frame = fixtures::frame_two_projects();
    for index in 0..frame.projects.len() {
        let project = frame.projects.get_mut(index).unwrap();
        let mut template = fixtures::template();
        template.number = TEMPLATE;
        template.budget = jig_domain_people::TemplateBudget::Amount(42);
        template.label = 5;
        project.templates = List::from_boxed(Box::new([template]));
    }
    let record = translate::owner::frame(&frame, fixtures::SIGN_IN_EXPIRES, &[], &w::CEILINGS).unwrap();
    let mut reader = skein_lib::Reader::new(record.projects.as_ref());
    for source in &frame.projects {
        let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
        let project = w::MeProject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!(project.project, source.project);
        let mut templates = skein_lib::Reader::new(project.templates.as_ref());
        let bytes = skein_codec::piece::split(&mut templates).unwrap().unwrap();
        let template = w::Template::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!(template.number, TEMPLATE);
        assert_eq!(template.label, 5);
        let w::TemplateBudget::Amount(budget) = &template.budget else { panic!("amount") };
        assert_eq!(budget.value, 42);
    }
    assert_eq!(skein_codec::piece::split(&mut reader), Ok(None));
}
#[test]
fn a_me_document_carries_each_project_s_creating_bounds() {
    let frame = frame_with_creating_bounds();
    let record = translate::owner::frame(&frame, fixtures::SIGN_IN_EXPIRES, &[], &w::CEILINGS).unwrap();
    let mut reader = skein_lib::Reader::new(record.projects.as_ref());
    for source in &frame.projects {
        let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
        let project = w::MeProject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!(project.project, source.project);
        let creating = project.creating.as_ref().unwrap();
        assert_eq!(creating.within.budget, BUDGET);
        assert_eq!(creating.within.pool_left, POOL_LEFT);
        let w::Beyond::Proposed(beyond) = &creating.beyond else { panic!("proposed") };
        let w::Recipient::Role(to) = &beyond.to else { panic!("role") };
        assert_eq!(to.project, source.project);
        assert_eq!(to.role, TEMPLATE_HOLDER_ROLE);
    }
    assert_eq!(skein_codec::piece::split(&mut reader), Ok(None));
}
#[test]
fn a_project_document_carries_its_policy_s_template_overrides() {
    let mut project = fixtures::project_document();
    project.policy.templates = List::from_boxed(Box::new([jig_domain_people::TemplateOverride {
        template: TEMPLATE,
        executor: None,
        budget: Some(jig_domain_people::TemplateBudget::Amount(42)),
        deadline: None,
        priority: None,
    }]));
    let record = translate::objects::project(&fixtures::models(), &project, &w::CEILINGS).unwrap();
    let entry = record.policy.templates.get(0).unwrap();
    assert_eq!(entry.template, TEMPLATE);
    assert_eq!((&entry.budget), &Some(w::TemplateBudget::Amount(w::TemplateAmount { value: 42 })));
}

fn frame_with_creating_bounds() -> jig_domain_core_vocabulary::documents::Frame {
    use jig_domain_core_vocabulary::acts;
    let mut frame = fixtures::frame_two_projects();
    for index in 0..frame.projects.len() {
        let project = frame.projects.get_mut(index).unwrap();
        project.creating = Some(acts::Creating {
            within: acts::Bounds::Create {
                authority: Box::new(fixtures::authority()),
                pool_left: POOL_LEFT,
                charters: List::from_boxed(Box::new([TEMPLATE_CHARTER])),
                procedures: List::with_capacity(0),
                latest: None,
                budget: BUDGET,
                maximum_budget: 100,
                maximum_deadline: None,
            },
            beyond: acts::Beyond::Proposed {
                to: acts::Recipient::Role { project: project.project, role: TEMPLATE_HOLDER_ROLE },
            },
        });
    }
    frame
}

const CREATION_CAPACITY: [(jig_domain_people::CapacityAction, w::PersonCapacityAction); 2] = [
    (jig_domain_people::CapacityAction::Create, w::PersonCapacityAction::Create),
    (jig_domain_people::CapacityAction::Compound, w::PersonCapacityAction::Compound),
];
#[test]
fn creating_and_compound_capacity_refusals_keep_the_operations_kind() {
    let deployment = fixtures::deployment_document();
    let mut refusal = *deployment.person_room_refusals.get(0).unwrap();
    for (action, expected) in CREATION_CAPACITY {
        refusal.action = action;
        let row = translate::deployment::person_refusal(&refusal);
        assert_eq!((&row.action), &expected);
        assert_eq!(row.key.person, refusal.key.person);
        assert_eq!((&row.key.key), &refusal.key.key);
    }
}

#[test]
fn added_dependencies_and_kept_waits_cross_both_client_translations() {
    use jig_client_domain::contract as c;
    use jig_domain_core_vocabulary::documents as d;
    use jig_domain_tasks as t;
    use skein_lib::{List, Reader, Wall, Writer};
    let limits = fixtures::wire_limits();
    let mut task = fixtures::task_with_hold();
    let mut facts = List::with_capacity(2);
    facts
        .push(d::Dependency {
            number: 7,
            pending: true,
            added: Some(t::Added { task: 7, at: Wall::from_nanos(11), by: t::Party::Person(13) }),
            keeps: true,
            status: None,
        })
        .unwrap();
    facts
        .push(d::Dependency {
            number: 9,
            pending: false,
            added: Some(t::Added { task: 9, at: Wall::from_nanos(17), by: t::Party::Task(19) }),
            keeps: true,
            status: Some(t::Status::Done),
        })
        .unwrap();
    task.dependency_facts = facts.clone();
    task.chip.details.dependencies = facts;
    let mut pending = List::with_capacity(1);
    pending.push(7).unwrap();
    task.waits_on = d::WaitsOn::Dependencies { pending, count: 1 };
    let record = translate::task::document(&task, &[], &limits).unwrap();
    let mut out = Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
    let bytes = out.finish();
    let record = w::Task::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    let actual = jig_client_protocol_link::translate::documents::task(record, &fixtures::client_limits()).unwrap();
    let first = actual.dependency_facts.get(0).unwrap();
    assert_eq!((first.number, first.pending, first.keeps), (7, true, true));
    assert_eq!(first.added, Some(c::DependencyAdded { at: Wall::from_nanos(11), by: c::Requester::Person(13) }));
    let second = actual.dependency_facts.get(1).unwrap();
    assert_eq!(second.status, Some(c::TaskStatus::Done));
    assert_eq!(second.added.unwrap().by, c::Requester::Task(19));
    let c::WaitsOn::Dependencies { pending, count } = actual.waits_on else { panic!("kept wait") };
    assert_eq!((pending.as_slice(), count), ([7].as_slice(), 1));
    let c::ChipPhaseFacts { details, .. } = actual.chip.details.as_ref();
    assert_eq!(details.dependencies, actual.dependency_facts);
}

#[test]
fn a_tasks_retained_subscriptions_fit_the_clients_page() {
    use jig_domain_tasks as t;
    const RETAINED: u32 = 7;
    const PAGE: u32 = 5;
    let mut task = fixtures::task_with_hold();
    task.subscriptions = List::with_capacity(RETAINED);
    for number in 1..=u64::from(RETAINED) {
        task.subscriptions
            .push(t::Subscription { number, kind: t::SubscriptionKind::Topic { connector: 1, topic: number } })
            .expect("fixture interest");
    }
    let limits = w::Limits { subscriptions_entries: PAGE, ..fixtures::wire_limits() };
    let record = translate::task::document(&task, &[], &limits).expect("task document");
    record.measure(&limits).expect("subscriptions fit the client wire");
    assert!(record.cut);
    assert_eq!(record.subscriptions.entries.len(), PAGE);
    for (number, subscription) in (1..=u64::from(PAGE)).zip(&record.subscriptions.entries) {
        assert_eq!(subscription.number, number);
    }
}

#[test]
fn a_circle_hold_keeps_its_order_and_cut_through_the_client() {
    use jig_client_domain::contract as c;
    use jig_domain_tasks as t;
    let mut task = fixtures::task_with_hold();
    let hold = task.hold.as_mut().unwrap();
    hold.reason = t::Hold::Circle;
    hold.circle = Some(t::Circle { tasks: Box::new([9, 7]), cut: true });
    let record = translate::task::document(&task, &[], &fixtures::wire_limits()).unwrap();
    let actual = jig_client_protocol_link::translate::documents::task(record, &fixtures::client_limits()).unwrap();
    let Some(hold) = actual.hold else { panic!("owner hold") };
    assert_eq!(hold.reason, c::HoldCategory::Circle);
    let circle = hold.circle.unwrap();
    assert_eq!(circle.tasks.as_slice(), &[9, 7]);
    assert!(circle.cut);
}

#[test]
fn unread_words_cross_into_the_wire_document_empty_and_full() {
    use jig_domain_core_vocabulary::documents::UnreadWords;
    let limits = fixtures::wire_limits();
    let mut source = fixtures::task_with_hold();
    for count in [0, limits.task_unread] {
        source.unread = List::with_capacity(count);
        for message in 0..count {
            source
                .unread
                .push(UnreadWords {
                    message: u64::from(message),
                    by: jig_domain_tasks::Party::Person(42),
                    at: skein_lib::Wall::from_nanos(77),
                    seen: message % 2 == 0,
                    head: jig_domain_core_vocabulary::documents::Text { bytes: Box::new([b'x'; 256]), cut: true },
                    bytes: 300,
                })
                .unwrap();
        }
        let record = translate::task::document(&source, &[], &limits).unwrap();
        assert_eq!(record.unread.len(), count);
        for (actual, expected) in record.unread.iter().zip(source.unread.iter()) {
            assert_eq!(actual.message, expected.message);
            assert_eq!(&actual.by, &w::Requester::Person(w::RequesterPerson { value: 42 }));
            assert_eq!(actual.at, expected.at.as_nanos());
            assert_eq!(actual.seen, expected.seen);
            assert_eq!(actual.head.as_ref(), expected.head.bytes.as_ref());
            assert_eq!(actual.bytes, expected.bytes);
        }
    }
}

#[test]
fn task_settings_revisions_cross_into_the_wire_document() {
    use jig_domain_core_vocabulary::documents::TaskSettings;
    let limits = fixtures::wire_limits();
    let mut source = fixtures::task_with_hold();
    for settings in [
        None,
        Some(TaskSettings { task: 7, run: None, newest: 9 }),
        Some(TaskSettings { task: 7, run: Some(6), newest: 9 }),
    ] {
        source.settings = settings;
        let record = translate::task::document(&source, &[], &limits).expect("task document");
        match ((&record.settings), settings) {
            (Some(actual), Some(expected)) => {
                assert_eq!(actual.task, expected.task);
                assert_eq!(actual.run, expected.run);
                assert_eq!(actual.newest, expected.newest);
            }
            (None, None) => (),
            (Some(_), None) | (None, Some(_)) => panic!("settings presence"),
        }
    }
}

#[test]
fn an_owner_is_offered_remove_project_and_a_member_is_not() {
    use jig_domain_core_vocabulary::acts::{Act, Admission, Offered};
    use skein_lib::{List, Reader};
    let limits = fixtures::wire_limits();
    // The protocol preserves the party-specific offers supplied by the core
    // (jig/docs/design/domain/people/acts.md, 2).
    for owner in [true, false] {
        let mut project = fixtures::project_document();
        project.acts = List::with_capacity(2);
        if owner {
            for act in [Act::Rename, Act::RemoveProject] {
                project.acts.push(Offered { act, admission: Admission::Allowed, effect: None }).expect("owner offers");
            }
        }
        let record = translate::objects::project(&[], &project, &limits).expect("project document");
        let mut input = Reader::new(record.acts.as_ref());
        let mut acts = List::with_capacity(2);
        while !input.is_empty() {
            let length = input.u32().expect("offer size");
            let bytes = input.bytes(length).expect("offer bytes");
            let offer = w::Offered::decode(&limits, &mut Reader::new(bytes)).expect("offer");
            assert_eq!((&offer.admission), &w::Admission::Allowed);
            acts.push(offer.act.clone()).expect("project offers");
        }
        assert_eq!(acts.len(), if owner { 2 } else { 0 });
        assert_eq!(acts.as_slice().contains(&w::ActKind::RemoveProject), owner);
        assert_eq!(acts.as_slice().contains(&w::ActKind::Rename), owner);
        assert!(input.is_empty());
    }
}

#[test]
fn a_served_template_carries_its_resources() {
    let resources =
        [jig_domain_people::Symbol { connector: 7, symbol: 3 }, jig_domain_people::Symbol { connector: 2, symbol: 11 }];
    let mut frame = fixtures::frame_two_projects();
    let mut template = fixtures::template();
    template.resources = Box::new(resources);
    frame.projects.get_mut(0).unwrap().templates = List::from_boxed(Box::new([template]));
    let record = translate::owner::frame(&frame, fixtures::SIGN_IN_EXPIRES, &[], &w::CEILINGS).unwrap();
    let bytes = skein_codec::piece::split(&mut skein_lib::Reader::new(record.projects.as_ref())).unwrap().unwrap();
    let project = w::MeProject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
    let bytes = skein_codec::piece::split(&mut skein_lib::Reader::new(project.templates.as_ref())).unwrap().unwrap();
    let template = w::Template::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
    assert_eq!(template.resources.len(), 2);
    for (wire, expected) in (&template.resources).into_iter().zip(resources) {
        assert_eq!((wire.connector, wire.symbol), (expected.connector, expected.symbol));
    }
    let client = jig_client_protocol_link::translate::documents::frame(record, &fixtures::client_limits()).unwrap();
    let template = client.projects.get(0).unwrap().templates.get(0).unwrap();
    assert_eq!(
        template.resources.as_slice(),
        &[
            jig_client_domain::contract::Symbol { connector: 7, symbol: 3 },
            jig_client_domain::contract::Symbol { connector: 2, symbol: 11 },
        ]
    );
}

#[test]
fn me_names_no_public_identifier_until_the_names_are_held() {
    let limits = fixtures::wire_limits();
    for public in [false, true] {
        let mut frame = fixtures::frame_two_projects();
        let mut projects = List::with_capacity(1);
        let mut project = frame.projects.into_boxed().into_iter().next().expect("fixture project");
        project.project = u32::MAX;
        project.public = public;
        projects.push(project).expect("one project");
        frame.projects = projects;
        let record = translate::owner::frame(&frame, fixtures::SIGN_IN_EXPIRES, &[], &limits).expect("me");
        let mut pieces = skein_lib::Reader::new(record.projects.as_ref());
        let bytes = skein_codec::piece::split(&mut pieces).expect("project piece").expect("one project");
        let project = w::MeProject::decode(&limits, &mut skein_lib::Reader::new(&bytes)).expect("project");
        assert_eq!(project.project, u32::MAX);
        assert_eq!(project.public, None);
        assert!(pieces.is_empty());
    }
}

#[test]
fn the_task_document_carries_its_talk_session_and_offered_acts() {
    use jig_client_domain::contract as c;
    use jig_domain_core_vocabulary::acts;
    use jig_domain_tasks as t;
    use skein_lib::{Reader, Wall, Writer};
    let limits = fixtures::wire_limits();
    let mut source = fixtures::task_with_hold();
    source.talk = Some(t::Talk { party: 41, opened: Wall::from_nanos(43), last: Wall::from_nanos(47) });
    source.session = Some(t::Session { first: 53, last: Wall::from_nanos(59), reasons: t::Reasons::WORDS });
    source.acts = List::with_capacity(2);
    for act in [acts::Act::Talk, acts::Act::Leave] {
        source.acts.push(acts::Offered { act, admission: acts::Admission::Allowed, effect: None }).unwrap();
    }
    let record = translate::task::document(&source, &[], &limits).unwrap();
    let mut bytes = Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
    let record = w::Task::decode(&limits, &mut Reader::new(&bytes.finish())).unwrap();
    let decoded = jig_client_protocol_link::translate::documents::task(record, &fixtures::client_limits()).unwrap();
    assert_eq!(decoded.talk, Some(c::Talk { party: 41, opened: Wall::from_nanos(43), last: Wall::from_nanos(47) }));
    assert_eq!(decoded.session, Some(c::Session { first: 53, last: Wall::from_nanos(59) }));
    assert_eq!(decoded.acts.len(), 2);
    assert_eq!(decoded.acts.get(0).unwrap().act, c::ActKind::Talk);
    assert_eq!(decoded.acts.get(1).unwrap().act, c::ActKind::Leave);
}

#[test]
fn an_own_question_wait_reaches_the_client_through_both_translations() {
    use jig_client_domain::contract as c;
    use jig_domain_core_vocabulary::documents::WaitsOn;
    use skein_lib::{Reader, Writer};
    let limits = fixtures::wire_limits();
    let mut source = fixtures::task_with_hold();
    source.waits_on = WaitsOn::Question { number: 7 };
    let record = translate::task::document(&source, &[], &limits).unwrap();
    let mut bytes = Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
    let bytes = bytes.finish();
    let record = w::Task::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    let decoded = jig_client_protocol_link::translate::documents::task(record, &fixtures::client_limits()).unwrap();
    assert_eq!(decoded.waits_on, c::WaitsOn::Question { number: 7 });
}

#[test]
fn structured_waits_reach_the_client_in_order_with_the_full_count() {
    use skein_lib::Reader;
    let (expected_wire, expected_client) = jig_client_test_records::structured_waits();
    let mut source = fixtures::task_with_hold();
    source.waits = core_waits();
    source.wait_count = 13;
    let limits = fixtures::wire_limits();
    let record = translate::task::document(&source, &[], &limits).unwrap();
    assert_eq!(record.waits, expected_wire);
    assert_eq!(record.wait_count, 13);
    let bytes = record.encode(&limits).unwrap();
    let record = w::Task::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    let decoded = jig_client_protocol_link::translate::documents::task(record, &fixtures::client_limits()).unwrap();
    assert_eq!(decoded.waits, expected_client);
    assert_eq!(decoded.wait_count, 13);
}

#[test]
fn task_documents_keep_the_first_wait_and_each_nearest_outcome_through_the_wire() {
    use jig_client_domain::contract as c;
    use jig_domain_core_vocabulary::{AtBound, Nearest, WaitKind};
    use skein_lib::{Reader, Wall};

    let limits = fixtures::wire_limits();
    let client_limits = fixtures::client_limits();
    let mut source = fixtures::task_with_hold();
    let (wire, client) = jig_client_test_records::structured_waits();
    let mut first = core_waits().get(0).unwrap().clone();
    let mut wire_first = wire.get(0).unwrap().clone();
    let mut client_first = client.get(0).unwrap().clone();
    first.task = 71;
    wire_first.task = 71;
    client_first.task = 71;
    let at = Wall::from_nanos(59);
    let cases = [
        None,
        Some((AtBound::Hold, w::AtBound::Hold, c::AtBound::Hold)),
        Some((AtBound::PassUp, w::AtBound::PassUp, c::AtBound::PassUp)),
        Some((AtBound::Overdue, w::AtBound::Overdue, c::AtBound::Overdue)),
        Some((AtBound::GiveUp, w::AtBound::GiveUp, c::AtBound::GiveUp)),
    ];
    for case in cases {
        let (wait, wire_wait, client_wait, nearest, wire_nearest, client_nearest) = match case {
            Some((then, wire_then, client_then)) => (
                Some(first.clone()),
                Some(&wire_first),
                Some(&client_first),
                Some(Nearest { at, then, kind: WaitKind::Host }),
                Some(w::Nearest { at: at.as_nanos(), then: wire_then, kind: w::WaitKind::Host }),
                Some(c::Nearest { at, then: client_then, kind: c::WaitKind::Host }),
            ),
            None => (None, None, None, None, None, None),
        };
        source.wait = wait;
        source.nearest = nearest;
        let record = translate::task::document(&source, &[], &limits).unwrap();
        assert_eq!(record.wait.as_ref(), wire_wait);
        assert_eq!(record.nearest, wire_nearest);
        let bytes = record.encode(&limits).unwrap();
        let record = w::Task::decode(&limits, &mut Reader::new(&bytes)).unwrap();
        let decoded = jig_client_protocol_link::translate::documents::task(record, &client_limits).unwrap();
        assert_eq!(decoded.wait.as_ref(), client_wait);
        assert_eq!(decoded.nearest, client_nearest);
    }
}

fn core_waits() -> List<jig_domain_core_vocabulary::Wait> {
    use jig_domain_core_vocabulary::documents::TaskRunWait;
    use jig_domain_core_vocabulary::{AtBound, Bound, Thing, Unbounded as U, Wait, WaitKind as K, WaitObject as O};
    use jig_domain_people::{EntryKind as E, Proposer};
    use jig_domain_tasks::{EscalationHolder as H, Name};
    use skein_lib::Wall;

    let task = 7;
    let at = H::Role { project: 7, role: 5 };
    let rows = [
        (
            K::Party,
            O::Entry { kind: E::Question { message: 401 }, at },
            Bound::At { at: Wall::from_nanos(201), then: AtBound::PassUp },
            None,
        ),
        (
            K::Party,
            O::Entry { kind: E::Proposal { proposer: Proposer::Task(11), number: 403 }, at },
            Bound::None(U::NoDeadline),
            Some(H::Task(51)),
        ),
        (
            K::Party,
            O::Entry { kind: E::Escalation { revision: 409 }, at },
            Bound::At { at: Wall::from_nanos(101), then: AtBound::Hold },
            Some(H::Person(53)),
        ),
        (
            K::Party,
            O::Entry { kind: E::PersonTask, at },
            Bound::None(U::Standing),
            Some(H::Role { project: 7, role: 59 }),
        ),
        (
            K::Question,
            O::Question { number: 23, answerer: 29 },
            Bound::At { at: Wall::from_nanos(301), then: AtBound::Overdue },
            None,
        ),
        (K::Dependency, O::Task(17), Bound::None(U::Behind(17)), None),
        (
            K::Resource,
            O::Resource {
                name: Name {
                    connector: 13,
                    path: Box::new([Box::<[u8]>::from(&b"pool/tiny"[..]), Box::new([0xff, b':'])]),
                },
                holder: Some(97),
            },
            Bound::None(U::OperatorHold),
            None,
        ),
        (
            K::Host,
            O::Run(TaskRunWait::Worker { worker: 43, until: Wall::from_nanos(173) }),
            Bound::At { at: Wall::from_nanos(701), then: AtBound::GiveUp },
            None,
        ),
        (
            K::News,
            O::News { subscriptions: List::from_boxed(Box::new([37, 31, 41, 47])), timer: Some(Wall::from_nanos(83)) },
            Bound::None(U::Standing),
            None,
        ),
        (
            K::Step,
            O::Procedure {
                step: 19,
                things: List::from_boxed(Box::new([
                    Thing { what: 1, number: 113, owner: Some(29) },
                    Thing { what: 2, number: 127, owner: None },
                    Thing { what: 3, number: 131, owner: Some(31) },
                    Thing { what: 4, number: 137, owner: None },
                ])),
            },
            Bound::None(U::NoDeadline),
            None,
        ),
    ];
    let mut waits = List::with_capacity(10);
    for (kind, object, bound, hears) in rows {
        let since = Wall::from_nanos(11 + u64::from(waits.len()));
        waits.push(Wait { task, kind, object, since, bound, hears }).unwrap();
    }
    waits
}

#[test]
fn wait_hold_chips_keep_the_category_and_words() {
    use jig_domain_core_vocabulary::documents::Text;
    use jig_domain_tasks as t;
    const WORDS: &[u8] = b"Waited 24 h for a reply";
    let mut hold = fixtures::task_with_hold().hold.expect("held fixture");
    for (reason, expected) in
        [(t::Hold::Waited, w::HoldCategory::Waited), (t::Hold::Deadline, w::HoldCategory::Deadline)]
    {
        hold.reason = reason;
        hold.words = Some(Text { bytes: WORDS.into(), cut: false });
        let record = translate::chips::held(&hold);
        assert_eq!(record.reason, expected);
        assert_eq!(record.words, Some(w::DocumentText { bytes: WORDS.into(), cut: false }));
    }
}
