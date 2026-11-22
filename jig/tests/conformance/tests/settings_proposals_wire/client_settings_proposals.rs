//! Settings proposal documents cross the client wire contract.
use jig_client_domain::contract as c;
use jig_client_protocol_link::{Limits, translate};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_scripted_engine::wire;
use jig_client_test_records::settings_proposals as values;
use skein_lib::{Reader, Writer};
macro_rules! encoded {
    ($value:expr) => {{
        let value = $value;
        let mut out = Writer::new(skein_lib::length::usize_of(
            value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
        ));
        value.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("measured");
        out.finish()
    }};
}
fn limits() -> Limits {
    let mut client = jig_client_kit::test::limits().client.domain;
    client.rows = 8;
    Limits::new(client, 4).expect("client limits")
}
fn submitted(
    ask: &w::Ask,
    _limits: &Limits,
) -> (jig_engine_clients_kit::Peer, skein_lib::Token, jig_domain_core::Event) {
    let mut peer = jig_engine_clients_kit::Peer::started(&jig_engine_clients_kit::fixtures::settings());
    let bearer = jig_engine_clients_kit::fixtures::signed_in(&mut peer);
    let keyed = w::Keyed {
        key: [7; 16],
        owner: w::Owner::Jig,
        version: jig_client_protocol_wire::VERSION,
        request: encoded!(ask),
        value: None,
    };
    let socket = peer.post(b"/jig/v1/request", encoded!(&keyed), Some(&bearer));
    let event = peer.take_core().expect("authenticated core request");
    (peer, socket, event)
}
#[test]
fn settings_proposals_and_running_decisions_keep_the_client_contract() {
    let limits = limits();
    let mut items = skein_lib::List::with_capacity(1);
    items
        .push(c::SettingItem {
            setting: c::Setting::Template(c::SettingNumber { number: 1 }),
            edit: c::SettingEdit::Removed,
        })
        .expect("one removal");
    let proposal = c::ProposeSettings { against: 6, items };
    let ask = c::Ask::Propose {
        project: 0,
        action: c::ProposeAction::Settings(proposal.clone()),
        reason: c::Text::new(Box::from(b"review".as_slice()), limits.client().text).expect("reason"),
    };
    let wire_ask = translate::asks::ask(&ask, limits.wire()).expect("ask");
    let (_, _, engine) = submitted(&wire_ask, &limits);
    assert!(matches!(
        engine,
        jig_domain_core::Event::ProposeSettings { change, .. } if change.against == 6 && change.items.len() == 1
    ));
    let w::Ask::Propose(wire) = wire_ask else { panic!("proposal") };
    assert_eq!(wire.project, 0);
    let w::ProposeAction::Settings(wire) = &wire.action else { panic!("settings") };
    assert_eq!(translate::settings_proposals::restore_propose_settings(wire, &limits), Ok(proposal));
    for running in [c::Running::Stays, c::Running::Moves] {
        let ask = c::Ask::Decide {
            project: 0,
            on: c::DecideOn::Proposal {
                proposal: c::ProposalKey { proposer: c::Proposer::Party(3), number: 7 },
                decision: c::Decision::Accepted,
                running: Some(running.clone()),
            },
        };
        let wire_ask = translate::asks::ask(&ask, limits.wire()).expect("decision");
        let (_, _, engine) = submitted(&wire_ask, &limits);
        let expected = match running {
            c::Running::Stays => jig_domain_people::Running::Stays,
            c::Running::Moves => jig_domain_people::Running::Moves,
        };
        assert!(
            matches!(engine, jig_domain_core::Event::People(jig_domain_people::Event::Ask { ask: jig_domain_people::Ask::DecideProposal { running: Some(value), .. }, .. }) if value == expected)
        );
        let w::Ask::Decide(wire) = wire_ask else { panic!("decide") };
        let w::DecideOn::Proposal(wire) = &wire.on else { panic!("proposal decision") };
        assert_eq!(translate::settings::restore_running(wire.running.as_ref().expect("running choice")), Ok(running));
    }
}
#[test]
fn pending_reviews_keep_origins_values_sizes_and_the_cut_fact() {
    let limits = limits();
    let pending = values::pending();
    let wire = wire_owner_proposal_action_settings(&pending, limits.wire()).expect("settings action");
    let mut parts = w::Proposal::decode(
        limits.wire(),
        &mut Reader::new(super::client_values::fixed_value("record_proposal_smallest.bin")),
    )
    .expect("proposal envelope");
    parts.action = w::ProposalAction::Settings(Box::new(wire));
    parts.waiting = w::ProposalWaiting {
        holder: w::HistoryHolder::Policy(w::HistoryHolderPolicy { project: 0, kind: w::MessageProposalKind::Settings }),
        since: 0,
    };

    let wire = w::ReadValue::Proposal(parts);
    let c::ReadValue::Proposal(client) = translate::reads::value(&encoded!(&wire), &limits).expect("client proposal")
    else {
        panic!("owner proposal")
    };
    assert_eq!(client.action, c::ProposalAction::Settings(Box::new(pending)));
    assert_eq!(client.waiting.holder, c::HistoryHolder::Policy { project: 0, kind: c::MessageProposalKind::Settings });
    let review = values::review();
    let c::ReviewValue::TooLarge(size) = &review.items.get(0).expect("first review item").would_be else {
        panic!("oversized")
    };
    assert_eq!(size.bytes, values::OVERSIZED_VALUE);
}
#[test]
fn retained_settings_actions_keep_the_revision_they_made() {
    let limits = limits();
    let retained = values::retained();
    let wire = wire::wire_retained_action_settings(&retained, limits.wire()).expect("retained settings");
    let mut proposal = w::ProposalHistory::decode(
        limits.wire(),
        &mut Reader::new(super::client_values::fixed_value("record_proposal_history_smallest.bin")),
    )
    .expect("history envelope");
    proposal.action = w::RetainedAction::Settings(wire);
    let key = w::ProposalKey {
        proposer: w::Proposer::Task(w::ProposerTask { value: proposal.proposer }),
        number: proposal.number,
    };
    proposal.state = w::ProposalHistoryState::Accepted(w::ProposalHistoryStateAccepted {
        by: w::Requester::Person(w::RequesterPerson { value: 3 }),
    });

    let value = w::EndedValue::Proposal(w::EndedValueProposal { value: proposal });
    let mut ended = w::Ended::decode(
        limits.wire(),
        &mut Reader::new(super::client_values::fixed_value("record_ended_smallest.bin")),
    )
    .expect("ended envelope");
    ended.key = w::ObjectKey::Proposal(w::ObjectKeyProposal { value: key });
    ended.value = value;
    let wire = w::ReadValue::Ended(Box::new(ended));
    let c::ReadValue::Ended(client) = translate::reads::value(&encoded!(&wire), &limits).expect("client history")
    else {
        panic!("ended")
    };
    let c::EndedValue::Proposal(proposal) = client.value else { panic!("retained proposal") };
    assert_eq!(proposal.action, c::RetainedAction::Settings(retained));
}

#[test]
fn a_running_choice_refusal_keeps_its_reason() {
    let limits = limits();
    let refusal = w::Refusal::Malformed(w::RefusalMalformed { at: 0, why: w::Malformed::Running });
    let answer = w::Final::Refused(w::Refused { refusal, at: None });
    let owned =
        w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: encoded!(&answer) };
    assert_eq!(
        translate::answers::answer(w::KeyedAnswer::FinalAnswer(owned), &limits),
        Ok(c::Answer::Refused { refusal: c::Refusal::Malformed { at: 0, why: c::Malformed::Running }, at: None })
    );
}

fn engine_proposal() -> jig_domain_core::documents::Proposal {
    use jig_domain_core::{documents as d, reads, settings as s};
    use jig_domain_tasks as t;
    let template = jig_domain_core::Template {
        resources: Box::new([]),
        number: 1,
        name: b"chat".as_slice().into(),
        executor: jig_domain_people::TemplateExecutor::Chosen,
        contract: t::Contract::Report { words: 1 },
        shallower: 0,
        budget: jig_domain_people::TemplateBudget::Given,
        deadline: jig_domain_people::TemplateDeadline::None,
        tracked: None,
        flags: t::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
    };
    let mut next = template.clone();
    next.grace = Some(skein_lib::Duration::from_secs(4));
    let mut items = skein_lib::List::with_capacity(1);
    items
        .push(d::ReviewItem {
            setting: s::Setting::Template(1),
            edit: reads::RevisionEdit::Set,
            now: Some(s::Value::Template(Box::new(template))),
            origin: Some((false, 1)),
            would_be: Some(s::Value::Template(Box::new(next))),
            application: None,
        })
        .expect("item");
    let mut heads = skein_lib::List::with_capacity(1);
    heads
        .push(reads::RevisionItemHead { setting: s::Setting::Template(1), edit: reads::RevisionEdit::Set })
        .expect("head");
    d::Proposal {
        projects: skein_lib::List::with_capacity(0),
        proposer: jig_domain_people::Proposer::Party(9),
        number: 7,
        project: 0,
        reason: d::Text { bytes: b"review".as_slice().into(), cut: false },
        action: d::ProposalAction::Settings {
            against: 1,
            items: heads,
            review: Some(Box::new(d::SettingsReview { key: None, newest: 1, items, value_bytes: 65_536 })),
            made: None,
        },
        as_holder: false,
        waiting: d::ProposalWaiting {
            holder: t::ProposalHolder::Policy { project: 0, kind: t::ProposalKind::Settings },
            since: skein_lib::Wall::EPOCH,
        },
        funding: skein_lib::List::with_capacity(0),
        acts: skein_lib::List::with_capacity(0),
    }
}
#[test]
fn settings_proposal_projects_cross_the_row_and_document_wire() {
    use jig_domain_core::{documents as d, reads};
    use jig_protocol_engine_clients::translate as engine;
    use skein_lib::List;
    let limits = limits();
    let mut source = engine_proposal();
    source.projects = List::with_capacity(2);
    source.projects.push(1).expect("first project");
    source.projects.push(2).expect("second project");
    let wire = engine::objects::proposal(&[], &source, limits.wire()).expect("engine proposal");
    let c::ReadValue::Proposal(proposal) =
        translate::reads::value(&encoded!(&w::ReadValue::Proposal(wire)), &limits).expect("proposal through link")
    else {
        panic!("proposal document")
    };
    assert_eq!(proposal.projects.as_slice(), &[1, 2]);
    assert_eq!(proposal.project, 0);
    assert_eq!(
        proposal.waiting.holder,
        c::HistoryHolder::Policy { project: 0, kind: c::MessageProposalKind::Settings }
    );
    let object = d::InboxObject::Proposal { proposer: source.proposer, number: source.number };
    let row = d::InboxEntry {
        projects: source.projects,
        reply: None,
        task: d::TaskLink::Stored { number: 0, status: None },
        project: Some(0),
        object,
        activity: None,
        at: None,
        unread: None,
        cursor: d::Cursor::Inbox { task: 0, object, at: None },
    };
    let mut rows = List::with_capacity(1);
    rows.push(reads::PageRow::Inbox(row)).expect("one row");
    let read = reads::Read::Inbox {
        group: reads::Group::Proposals,
        filter: reads::Filter { project: Some(1), goal: None },
        from: None,
    };
    let wire = engine::pages::page(
        &[],
        &read,
        &reads::Page { rows, before: None, next: None, cut: false, inbox_selection: None },
        &jig_protocol_engine_clients::Private::none(),
        limits.wire(),
    )
    .expect("inbox wire");
    let c::ReadValue::Inbox(inbox) = translate::reads::value(&encoded!(&wire), &limits).expect("inbox through link")
    else {
        panic!("inbox page")
    };
    let row = inbox.rows.get(0).expect("proposal row");
    assert_eq!(row.projects, proposal.projects);
    assert!(row.shows_in(0));
    assert!(row.shows_in(1));
    assert!(row.shows_in(2));
    assert!(!row.shows_in(3));
}

#[test]
fn engine_pending_settings_review_reaches_the_client_and_encodes_back_without_loss() {
    let limits = limits();
    let source = engine_proposal();
    let wire = jig_protocol_engine_clients::translate::objects::proposal(&[], &source, limits.wire())
        .expect("engine document");
    let c::ReadValue::Proposal(client) =
        translate::reads::value(&encoded!(&w::ReadValue::Proposal(wire.clone())), &limits).expect("client review")
    else {
        panic!("proposal document")
    };
    let c::ProposalAction::Settings(action) = client.action else { panic!("settings action") };
    assert_eq!((action.against, action.made), (1, None));
    let review = action.review.as_ref().expect("pending review");
    assert_eq!(review.newest, 1);
    let item = review.items.get(0).expect("one item");
    assert_eq!(item.origin, c::SettingOrigin::Default(c::OriginRevision { revision: 1 }));
    let c::ReviewValue::Value(value) = &item.would_be else { panic!("proposed value") };
    let c::SettingValue::Template(template) = &value.value else { panic!("template") };
    assert_eq!(template.grace, Some(skein_lib::Duration::from_secs(4)));
    let w::ProposalAction::Settings(original) = &wire.action else { panic!("settings wire action") };
    assert_eq!(wire_owner_proposal_action_settings(&action, limits.wire()).expect("client encoder"), **original);
}
#[test]
fn engine_accepted_party_settings_history_keeps_its_party_key_and_revision() {
    use jig_domain_core::reads;
    use jig_domain_tasks as t;
    let limits = limits();
    let proposal = t::PersonProposal {
        number: 7,
        proposer: 9,
        project: 0,
        reason: b"review".as_slice().into(),
        action: t::PersonAction::Settings {
            against: 1,
            items: Box::new([t::SettingHead { setting: t::Setting::Template(1), edit: t::SettingEdit::Set }]),
        },
        state: t::PersonProposalState::SettingsAccepted { by: t::Party::Person(10), revision: 2 },
    };
    let ended = reads::Ended {
        value: reads::EndedValue::PersonProposal { proposal: Box::new(proposal), decision: None },
        at: Some(skein_lib::Wall::EPOCH),
        by: Some(t::Party::Person(10)),
    };
    let wire =
        jig_protocol_engine_clients::translate::objects::ended(&ended, &[], limits.wire()).expect("engine history");
    let c::ReadValue::Ended(client) =
        translate::reads::value(&encoded!(&w::ReadValue::Ended(Box::new(wire))), &limits).expect("client history")
    else {
        panic!("ended proposal")
    };
    let key = c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Party(9), number: 7 });
    assert!(client.matches(key));
    let c::EndedValue::Proposal(proposal) = client.value else { panic!("retained action history") };
    let c::RetainedAction::Settings(action) = proposal.action else { panic!("settings heads") };
    assert_eq!((action.against, action.made), (1, Some(2)));
    assert_eq!(action.items.get(0).expect("head").setting, c::Setting::Template(c::SettingNumber { number: 1 }));
}
#[test]
fn engine_running_and_pending_settings_limit_refusals_keep_their_client_names() {
    use jig_domain_people as p;
    let limits = limits();
    for (source, expected) in [
        (
            p::Refusal::NotWellFormed { at: 0, why: p::Malformed::Running },
            c::Refusal::Malformed { at: 0, why: c::Malformed::Running },
        ),
        (
            p::Refusal::Limit { which: p::LimitKind::SettingsProposals },
            c::Refusal::Limit { which: c::Limit::Admission { which: c::AdmissionLimit::SettingsProposals } },
        ),
    ] {
        let wire = w::Ask::Message(w::Message { project: 1, task: 1, words: Box::new([]) });
        let (mut peer, socket, event) = submitted(&wire, &limits);
        let jig_domain_core::Event::People(p::Event::Ask { reply_to, .. }) = event else { panic!("people ask") };
        peer.answer_request(reply_to, p::Reply::Outcome(p::Outcome::Refused(source)));
        let answer = w::KeyedAnswer::decode(limits.wire(), &mut Reader::new(peer.answered(socket).expect("answer")))
            .expect("keyed answer");
        assert_eq!(translate::answers::answer(answer, &limits), Ok(c::Answer::Refused { refusal: expected, at: None }));
    }
}

// Preserve the review key carried by owner documents (client/domain/contract/documents.md, 1).
fn wire_owner_proposal_action_settings(
    value: &c::ProposalActionSettings,
    limits: &w::Limits,
) -> Result<w::ProposalActionSettings, w::Problem> {
    let mut parts = wire::wire_proposal_action_settings(value, limits)?;
    if let (Some(source), Some(review)) = (&value.review, parts.review.take()) {
        let mut review = review;
        review.key = source.key;
        parts.review = Some(review);
    }
    Ok(parts)
}
