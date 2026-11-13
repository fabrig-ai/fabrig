//! Settings cross the clients' component as one value and one keyed digest.
use super::fixtures::{self as f, Exchange};
use crate::{Request, translate};
use jig_client_protocol_wire::v20 as w;
use jig_domain_core_vocabulary::{self as core, settings as s};
use jig_domain_people as p;
use skein_lib::List;

fn ask() -> w::Ask {
    w::Ask::ChangeSettings(
        jig_client_protocol_link::translate::settings::wire_change_settings(
            &jig_client_test_records::settings::settings_change(),
            &f::wire(),
        )
        .expect("settings fixture"),
    )
}
fn whole_settings_exchange() -> Exchange {
    let mut env = f::env();
    env.limits.message_bytes = 8192;
    env.limits.wire.keyed_request = 8192;
    let mut exchange = Exchange::with_env(env);
    exchange.component.settings.models[0].name = Box::from(b"model".as_slice());
    exchange.component.settings.models[1].name = Box::from(b"other".as_slice());
    exchange
}
#[test]
fn all_four_whole_values_translate_and_the_component_passes_the_exact_asks_digest() {
    let mut exchange = whole_settings_exchange();
    let wire = ask();
    let encoded = f::encode_ask(&wire);
    let expected_digest = crate::sign_in::digest(&encoded).expose().try_into().expect("settings fixture");
    let expected = p::Ask::ChangeSettings { against: 5, digest: expected_digest, projects: Box::default() };
    assert_eq!(translate::asks::ask(wire, &exchange.component.settings.models, &f::wire()), Ok(expected));
    let keyed = w::Keyed::new(
        &exchange.env.limits.wire,
        w::KeyedParts { key: [5; 16], owner: w::Owner::Jig, version: 20, request: encoded, value: None },
    )
    .expect("settings envelope");
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(keyed.measure()));
    keyed.encode(&mut writer).expect("settings envelope");
    exchange.send(crate::Message::Request(writer.finish()));
    exchange.authenticate();
    let Some(Request::Core(event)) = exchange.out.pop() else { panic!("core entrance") };
    let core::Event::ChangeSettings { sign_in, key, digest, change, .. } = *event else { panic!("settings entrance") };
    assert_eq!((sign_in, key, digest), (41, [5; 16], expected_digest));
    assert_eq!(change.against, 5);
    assert_eq!(change.running, s::Running::Moves);
    assert_eq!(change.items.len(), 4);
    let s::Edit::Set(s::Value::Charter(charter)) = &change.items[0].edit else { panic!("charter") };
    assert_eq!(change.items[0].setting, s::Setting::Charter(2));
    assert_eq!(charter.policy.instructions.as_ref(), "instructions λ".as_bytes());
    assert_eq!(charter.policy.model.effort.as_deref(), Some(b"high".as_slice()));
    assert_eq!(charter.policy.model.cache_write_price, Some(5));
    assert_eq!(charter.policy.alternatives[0].name.as_ref(), b"other");
    let s::Edit::Set(s::Value::Template(template)) = &change.items[1].edit else { panic!("template") };
    assert_eq!(template.grace, Some(skein_lib::Duration::from_secs(18)));
    assert_eq!(
        template.llm.as_ref().expect("settings fixture").main,
        jig_domain_tasks::ModelEffort { model: 0, effort: Some(1) }
    );
    assert_eq!(template.llm.as_ref().expect("settings fixture").account, Some(21));
    assert!(template.flags.conversation);
    let s::Edit::Set(s::Value::Policy(policy)) = &change.items[2].edit else { panic!("policy") };
    assert_eq!(policy.policy.period_spend, 101);
    assert_eq!(policy.policy.requirements[0].judge.parameters, 6);
    assert_eq!(policy.permissions[0].permission, 11);
    assert_eq!(policy.templates[0].priority, Some(5));
    let s::Edit::Set(s::Value::Rules(rules)) = &change.items[3].edit else { panic!("rules") };
    assert_eq!(rules.implies.entries()[0].implies, 3);
    assert_eq!(rules.maximum_run_spend, 90);
    assert!(exchange.out.is_empty());
}
#[test]
fn resets_and_removals_translate_and_bad_pieces_reach_no_core() {
    let limits = f::wire();
    let mut rows = List::with_capacity(2);
    for edit in [w::SettingEdit::Reset, w::SettingEdit::Removed] {
        let item = w::SettingItem::new(&limits, w::SettingItemParts { setting: w::Setting::Rules, edit })
            .expect("settings fixture");
        let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(item.measure()));
        item.encode(&mut writer).expect("settings fixture");
        rows.push(writer.finish()).expect("settings fixture");
    }
    let request = w::ChangeSettings::new(
        &limits,
        w::ChangeSettingsParts { against: 3, items: f::pieces(rows.as_slice()), running: w::Running::Stays },
    )
    .expect("settings fixture");
    let change = translate::settings::change(request, &f::models(), &limits).expect("settings fixture");
    assert_eq!(
        change.items.as_ref(),
        &[
            s::Item { setting: s::Setting::Rules, edit: s::Edit::Reset },
            s::Item { setting: s::Setting::Rules, edit: s::Edit::Removed }
        ]
    );
    for items in [Box::<[u8]>::from([0]), f::pieces(&[Box::new([255])])] {
        let wire = w::Ask::ChangeSettings(
            w::ChangeSettings::new(&limits, w::ChangeSettingsParts { against: 1, items, running: w::Running::Stays })
                .expect("settings fixture"),
        );
        let mut exchange = Exchange::new();
        let owner = exchange.send(crate::Message::Request(f::keyed(w::Owner::Jig, 20, f::encode_ask(&wire))));
        exchange.authenticate();
        assert!(exchange.out.is_empty());
        assert_eq!(exchange.response(owner).0, 400);
    }
}
#[test]
fn settings_answers_and_each_fault_keep_their_facts() {
    let limits = f::wire();
    let answer = f::translated(&p::Reply::Outcome(p::Outcome::SettingsChanged { revision: 8 }), &limits);
    let w::Final::Done(value) = answer else { panic!("done settings") };
    let w::Outcome::SettingsChanged(value) = value.outcome() else { panic!("settings changed") };
    assert_eq!(value.revision(), 8);
    for why in [
        p::Refusal::Stale { newest: 8 },
        p::Refusal::Full { revision: 8, tasks: Box::new([3, 5]), more: 2 },
        p::Refusal::NotWellFormed { at: 0, why: p::Malformed::SettingsUnchanged },
    ] {
        let value = translate::refusals::refusal(why.clone(), &limits).expect("settings fixture");
        match (why, value) {
            (p::Refusal::Stale { newest }, w::Refusal::Stale(value)) => assert_eq!(value.newest(), newest),
            (p::Refusal::Full { revision, tasks, more }, w::Refusal::Full(value)) => {
                assert_eq!(value.revision(), revision);
                assert_eq!(value.tasks().as_slice(), tasks.as_ref());
                assert_eq!(value.more(), more);
            }
            (p::Refusal::NotWellFormed { .. }, w::Refusal::Malformed(value)) => {
                assert_eq!(value.why(), &w::Malformed::SettingsUnchanged);
            }
            value => panic!("settings refusal {value:?}"),
        }
    }
    for setting in [p::Setting::Charter(1), p::Setting::Template(2), p::Setting::Policy(3), p::Setting::Rules] {
        for fault in [
            p::SettingFault::Shape,
            p::SettingFault::Count,
            p::SettingFault::Reference,
            p::SettingFault::Rules,
            p::SettingFault::Budget,
            p::SettingFault::Grace,
            p::SettingFault::Owner,
        ] {
            let value = translate::refusals::refusal(
                p::Refusal::NotWellFormed { at: 0, why: p::Malformed::Settings { setting, fault, detail: None } },
                &limits,
            )
            .expect("settings fixture");
            let w::Refusal::Malformed(value) = value else { panic!("malformed settings") };
            let w::Malformed::Settings(facts) = value.why() else { panic!("settings facts") };
            assert_eq!(
                facts.setting(),
                &match setting {
                    jig_domain_people::Setting::ProjectPolicy { project } => w::Setting::ProjectPolicy(
                        w::ProjectPolicySetting::new(&limits, w::ProjectPolicySettingParts { project })
                            .expect("project policy")
                    ),
                    p::Setting::Charter(number) => w::Setting::Charter(
                        w::SettingNumber::new(&limits, w::SettingNumberParts { number }).expect("number")
                    ),
                    p::Setting::Template(number) => w::Setting::Template(
                        w::SettingNumber::new(&limits, w::SettingNumberParts { number }).expect("number")
                    ),
                    p::Setting::Policy(number) => w::Setting::Policy(
                        w::SettingNumber::new(&limits, w::SettingNumberParts { number }).expect("number")
                    ),
                    p::Setting::Model(number) => w::Setting::Model(
                        w::SettingNumber::new(&limits, w::SettingNumberParts { number }).expect("number")
                    ),
                    p::Setting::Rules => w::Setting::Rules,
                    p::Setting::Application { kind, scope, number } => w::Setting::Application(
                        w::ApplicationSetting::new(&limits, w::ApplicationSettingParts { kind, scope, number })
                            .expect("application setting")
                    ),
                }
            );
        }
    }
}

fn proposal_ask() -> w::Ask {
    let w::Ask::ChangeSettings(value) = ask() else { panic!("settings fixture") };
    proposed_settings(value)
}
fn proposed_settings(value: w::ChangeSettings) -> w::Ask {
    let value = value.into_parts();
    let limits = f::wire();
    w::Ask::Propose(
        w::Propose::new(
            &limits,
            w::ProposeParts {
                project: 0,
                action: w::ProposeAction::Settings(
                    w::ProposeSettings::new(
                        &limits,
                        w::ProposeSettingsParts { against: value.against, items: value.items },
                    )
                    .expect("proposed set"),
                ),
                reason: b"review this set".as_slice().into(),
            },
        )
        .expect("proposal ask"),
    )
}
fn template_choice(model: &[u8], effort: &[u8]) -> jig_client_domain::contract::ModelEffort {
    use jig_client_domain::contract as c;
    c::ModelEffort {
        model: c::Text::new(model.into(), f::wire().model_effort_model).unwrap(),
        effort: Some(c::Text::new(effort.into(), f::wire().model_effort_effort_some).unwrap()),
    }
}
fn template_requests(
    main: jig_client_domain::contract::ModelEffort,
    subagents: List<jig_client_domain::contract::ModelEffort>,
) -> [w::Ask; 2] {
    use jig_client_domain::contract as c;
    let mut item = jig_client_test_records::settings::settings_change()
        .items
        .into_boxed()
        .into_iter()
        .nth(1)
        .expect("template item");
    item.setting = c::Setting::Template(c::SettingNumber { number: 4 });
    let c::SettingEdit::Set(set) = &mut item.edit else { panic!("set") };
    let c::SettingValue::Template(template) = &mut set.value else { panic!("template") };
    template.number = 4;
    template.llm = Some(c::LlmChoice { later: None, account: None, main, subagents });
    let change =
        c::ChangeSettings { against: 5, running: c::Running::Stays, items: List::from_boxed(Box::new([item])) };
    let change = jig_client_protocol_link::translate::settings::wire_change_settings(&change, &f::wire()).unwrap();
    [w::Ask::ChangeSettings(change.clone()), proposed_settings(change)]
}
fn send_template(ask: &w::Ask) -> (Exchange, skein_lib::Token) {
    let mut settings = f::settings();
    settings.models[0].efforts =
        Box::new([Box::from(b"high".as_slice()), Box::from(b"medium".as_slice()), Box::from(b"low".as_slice())]);
    let mut exchange = Exchange::configured(f::env(), settings);
    let owner = exchange.send(crate::Message::Request(f::keyed(w::Owner::Jig, 20, f::encode_ask(ask))));
    exchange.authenticate();
    (exchange, owner)
}
fn admitted_items(event: core::Event) -> Box<[s::Item]> {
    let core::Event::ChangeSettings { change, .. } = event else {
        let core::Event::ProposeSettings { change, .. } = event else { panic!("settings entrance: {event:?}") };
        return change.items;
    };
    change.items
}
#[test]
fn unsupported_template_efforts_receive_a_final_refusal() {
    for (main, subagents, model, why) in [
        (template_choice(b"careful", b"xhigh"), List::with_capacity(0), b"careful".as_slice(), w::LlmWhy::Effort),
        (
            template_choice(b"careful", b"high"),
            List::from_boxed(Box::new([template_choice(b"careful", b"xhigh")])),
            b"careful".as_slice(),
            w::LlmWhy::Effort,
        ),
        (template_choice(b"absent", b"high"), List::with_capacity(0), b"absent".as_slice(), w::LlmWhy::Unconfigured),
    ] {
        for ask in template_requests(main, subagents) {
            let (exchange, owner) = send_template(&ask);
            assert!(exchange.out.is_empty(), "invalid settings reached the core: {ask:?}");
            let w::Final::Refused(value) = exchange.final_answer(owner) else { panic!("final refusal: {ask:?}") };
            let w::Refusal::Llm(refusal) = value.refusal() else { panic!("named LLM refusal: {ask:?}") };
            assert_eq!(refusal.model(), model);
            assert_eq!(*refusal.why(), why);
            assert_eq!(*value.at(), None);
        }
    }
}
#[test]
fn supported_template_efforts_reach_the_core() {
    let expected = jig_domain_tasks::ModelEffort { model: 0, effort: Some(0) };
    for ask in template_requests(
        template_choice(b"careful", b"high"),
        List::from_boxed(Box::new([template_choice(b"careful", b"high")])),
    ) {
        let (mut exchange, _owner) = send_template(&ask);
        let Some(Request::Core(event)) = exchange.out.pop() else { panic!("core entrance: {ask:?}") };
        let items = admitted_items(*event);
        let [s::Item { setting: s::Setting::Template(4), edit: s::Edit::Set(s::Value::Template(template)) }] =
            items.as_ref()
        else {
            panic!("only template 4")
        };
        let llm = template.llm.as_ref().expect("template choice");
        assert_eq!(llm.main, expected);
        assert_eq!(llm.subagents.as_ref(), &[expected]);
        assert!(exchange.out.is_empty());
    }
}
#[test]
fn settings_proposals_reach_the_core_with_the_whole_set_and_exact_digest() {
    let mut exchange = whole_settings_exchange();
    let ask = proposal_ask();
    let bytes = f::encode_ask(&ask);
    let digest: [u8; 32] = crate::sign_in::digest(&bytes).expose().try_into().expect("digest");
    assert_eq!(
        translate::asks::ask(ask, &exchange.component.settings.models, &f::wire()),
        Ok(p::Ask::Propose {
            project: 0,
            action: p::Action::Settings { against: 5, digest },
            reason: b"review this set".as_slice().into()
        })
    );
    exchange.send(crate::Message::Request(proposal_envelope(bytes, &exchange.env.limits.wire)));
    exchange.authenticate();
    let Some(Request::Core(event)) = exchange.out.pop() else { panic!("core proposal") };
    let core::Event::ProposeSettings { sign_in, digest: held, change, .. } = *event else {
        panic!("settings proposal entrance")
    };
    assert_eq!((sign_in, held, change.against, change.items.len()), (41, digest, 5, 4));
    assert_eq!(change.reason.as_ref(), b"review this set");
}
#[test]
fn accepting_settings_translates_both_running_choices() {
    let limits = f::wire();
    for (wire, expected) in [(w::Running::Stays, p::Running::Stays), (w::Running::Moves, p::Running::Moves)] {
        let value = w::Ask::Decide(
            w::Decide::new(
                &limits,
                w::DecideParts {
                    project: 0,
                    on: w::DecideOn::Proposal(
                        w::DecideProposal::new(
                            &limits,
                            w::DecideProposalParts {
                                proposal: w::ProposalKey::new(
                                    &limits,
                                    w::ProposalKeyParts {
                                        proposer: w::Proposer::Party(
                                            w::ProposerParty::new(&limits, w::ProposerPartyParts { value: 9 })
                                                .expect("proposer"),
                                        ),
                                        number: 7,
                                    },
                                )
                                .expect("proposal key"),
                                decision: w::Decision::Accepted,
                                running: Some(wire),
                            },
                        )
                        .expect("decision"),
                    ),
                },
            )
            .expect("ask"),
        );
        assert_eq!(
            translate::asks::ask(value, &f::models(), &limits),
            Ok(p::Ask::DecideProposal {
                project: 0,
                proposer: p::Proposer::Party(9),
                proposal: 7,
                decision: p::ProposalDecision::Accept,
                running: Some(expected)
            })
        );
    }
}
#[test]
fn unknown_models_in_settings_proposals_are_named_before_decoding_the_set() {
    let mut env = f::env();
    env.limits.message_bytes = 8192;
    env.limits.wire.keyed_request = 8192;
    let mut exchange = Exchange::with_env(env);
    let bytes = proposal_envelope(f::encode_ask(&proposal_ask()), &exchange.env.limits.wire);
    let owner = exchange.send(crate::Message::Request(bytes));
    exchange.authenticate();
    assert!(exchange.out.is_empty());
    let w::Final::Refused(value) = exchange.final_answer(owner) else { panic!("named model refusal") };
    let w::Refusal::Llm(refusal) = value.refusal() else { panic!("LLM refusal") };
    assert_eq!(refusal.model(), b"model");
    assert_eq!(*refusal.why(), w::LlmWhy::Unconfigured);
}

fn proposal_envelope(request: Box<[u8]>, limits: &w::Limits) -> Box<[u8]> {
    let keyed =
        w::Keyed::new(limits, w::KeyedParts { key: [5; 16], owner: w::Owner::Jig, version: 20, request, value: None })
            .expect("whole proposal envelope");
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(keyed.measure()));
    keyed.encode(&mut writer).expect("measured envelope");
    writer.finish()
}

#[test]
fn move_to_newest_translates_the_revision_and_both_counts() {
    let limits = f::wire();
    let ask = w::Ask::MoveToNewest(w::MoveToNewest::new(&limits, w::MoveToNewestParts { revision: 7 }).expect("move"));
    assert_eq!(translate::asks::ask(ask, &[], &limits), Ok(p::Ask::MoveToNewest { revision: 7 }));
    let answer = translate::answers::answer(
        &p::Reply::Outcome(p::Outcome::MovedToNewest { newest: 9, moved: 3, stayed: 1 }),
        &[],
        &limits,
    )
    .expect("answer");
    let w::KeyedAnswer::FinalAnswer(answer) = answer else { panic!("final answer") };
    let answer = w::Final::decode(&limits, &mut skein_lib::Reader::new(answer.record())).expect("final");
    let w::Final::Done(answer) = answer else { panic!("done") };
    let w::Outcome::MovedToNewest(value) = answer.outcome() else { panic!("counts") };
    assert_eq!((value.newest(), value.moved(), value.stayed()), (9, 3, 1));
}

#[test]
fn public_settings_items_share_model_resolution_and_name_invalid_fields() {
    let limits = f::wire();
    let w::Ask::ChangeSettings(request) = ask() else { panic!("settings fixture") };
    let mut models = f::models();
    models[0].name = b"model".as_slice().into();
    models[1].name = b"other".as_slice().into();
    let (first, rest) = models.split_at_mut_checked(1).expect("two models");
    ::core::mem::swap(first.first_mut().expect("first model"), rest.first_mut().expect("second model"));
    let expected = translate::settings::change(request.clone(), &models, &limits).expect("existing translation");
    let decoded = translate::settings::items(request.items().into(), &models, &limits).expect("public entrance");
    assert_eq!(decoded, expected.items);
    assert_eq!(decoded.len(), 4);
    let s::Edit::Set(s::Value::Template(template)) = &decoded[1].edit else { panic!("template") };
    assert_eq!(template.llm.as_ref().expect("model selection").main.model, 1);
    let bad = translate::settings::items(Box::new([0]), &models, &limits).expect_err("truncated item length");
    assert_eq!(bad.field, w::Path::ChangeSettingsItems);
    let bad = translate::settings::items(request.items().into(), &[], &limits).expect_err("unconfigured model");
    assert_eq!(bad.field, w::Path::TemplateLlm);
}

#[test]
fn a_changed_template_keeps_its_resources() {
    use jig_client_domain::contract as client;
    let limits = f::wire();
    let resources = [p::Symbol { connector: 7, symbol: 3 }, p::Symbol { connector: 2, symbol: 11 }];
    let mut change = jig_client_test_records::settings::settings_change();
    let client::SettingEdit::Set(set) = &mut change.items.get_mut(1).unwrap().edit else { panic!("set") };
    let client::SettingValue::Template(template) = &mut set.value else { panic!("template") };
    template.resources = List::from_boxed(Box::new([
        client::Symbol { connector: 7, symbol: 3 },
        client::Symbol { connector: 2, symbol: 11 },
    ]));
    let wire = jig_client_protocol_link::translate::settings::wire_change_settings(&change, &limits).unwrap();
    let mut models = f::models();
    models[0].name = Box::from(b"model".as_slice());
    let translated = translate::settings::items(wire.into_parts().items, &models, &limits).unwrap();
    let s::Edit::Set(s::Value::Template(template)) = &translated[1].edit else { panic!("template") };
    assert_eq!(template.resources.as_ref(), &resources);
}

#[test]
fn application_edits_round_trip_from_client_bytes_into_changes_and_proposals() {
    use jig_client_domain::contract as c;
    use jig_client_protocol_link::translate::settings as link;
    let limits = f::wire();
    let setting = c::Setting::Application { kind: 17, scope: 4, number: 9 };
    for (edit, expected) in [
        (
            c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(Box::new([0, 255, 128, 7])) }),
            s::Edit::Set(s::Value::Application(Box::new([0, 255, 128, 7]))),
        ),
        (c::SettingEdit::Removed, s::Edit::Removed),
        (c::SettingEdit::Reset, s::Edit::Reset),
    ] {
        let mut items = List::with_capacity(1);
        items.push(c::SettingItem { setting, edit }).unwrap();
        let client = c::ChangeSettings { against: 5, running: c::Running::Stays, items };
        let wire = link::wire_change_settings(&client, &limits).unwrap();
        assert_eq!(link::restore_change_settings(&wire, &f::client_limits()), Ok(client));
        let bytes = f::encode_ask(&w::Ask::ChangeSettings(wire));
        let w::Ask::ChangeSettings(wire) = w::Ask::decode(&limits, &mut skein_lib::Reader::new(&bytes)).unwrap() else {
            panic!("change")
        };
        let change = translate::settings::change(wire.clone(), &f::models(), &limits).unwrap();
        let expected = s::Item { setting: s::Setting::Application { kind: 17, scope: 4, number: 9 }, edit: expected };
        assert_eq!(change.items.as_ref(), ::core::slice::from_ref(&expected));
        let parts = wire.into_parts();
        let proposed = w::Propose::new(
            &limits,
            w::ProposeParts {
                project: 0,
                action: w::ProposeAction::Settings(
                    w::ProposeSettings::new(&limits, w::ProposeSettingsParts { against: 5, items: parts.items })
                        .unwrap(),
                ),
                reason: Box::from(b"review".as_slice()),
            },
        )
        .unwrap();
        let proposed = translate::settings::proposed(proposed, &f::models(), &limits).unwrap();
        assert_eq!(proposed.items.as_ref(), &[expected]);
        assert_eq!(proposed.reason.as_ref(), b"review");
    }
}

#[test]
fn a_judged_refusal_reaches_the_client_with_its_item_and_connector_words() {
    use jig_client_domain::contract as c;
    let limits = f::wire();
    let reply = p::Reply::Outcome(p::Outcome::Refused(p::Refusal::Judged {
        at: 2,
        setting: p::Setting::Application { kind: 7, scope: 4, number: 9 },
        why: Box::from("requires approval λ".as_bytes()),
    }));
    let wire = translate::answers::answer(&reply, &f::models(), &limits).unwrap();
    let answer = jig_client_protocol_link::translate::answers::answer(wire, &f::client_limits()).unwrap();
    let c::Answer::Refused { refusal: c::Refusal::Judged { at, setting, why }, .. } = answer else {
        panic!("judged refusal")
    };
    assert_eq!(at, 2);
    assert_eq!(setting, c::Setting::Application { kind: 7, scope: 4, number: 9 });
    assert_eq!(why.bytes(), "requires approval λ".as_bytes());
}

#[test]
fn document_refusal_details_cross_the_client_wire() {
    use jig_client_domain::contract as c;
    use jig_domain_people as p;
    let limits = jig_client_protocol_wire::CEILINGS;
    for (member, expected) in [
        (p::SettingMember::Documents, c::SettingMember::Documents),
        (p::SettingMember::NameSegments, c::SettingMember::NameSegments),
        (p::SettingMember::NameSegmentBytes, c::SettingMember::NameSegmentBytes),
        (p::SettingMember::Path, c::SettingMember::Path),
        (p::SettingMember::Heading, c::SettingMember::Heading),
        (p::SettingMember::Connector, c::SettingMember::Connector),
        (p::SettingMember::StoredPolicy, c::SettingMember::StoredPolicy),
    ] {
        for (rule, expected_rule) in [
            (p::SettingRule::Maximum, c::SettingRule::Maximum),
            (p::SettingRule::Minimum, c::SettingRule::Minimum),
            (p::SettingRule::Registered, c::SettingRule::Registered),
        ] {
            let refusal = p::Refusal::NotWellFormed {
                at: 2,
                why: p::Malformed::Settings {
                    setting: p::Setting::Charter(5),
                    fault: p::SettingFault::Shape,
                    detail: Some(p::SettingDetail { member, document: 3, segment: 4, rule, supplied: 7, bound: 6 }),
                },
            };
            let wire = translate::refusals::refusal(refusal, &limits).expect("document refusal");
            let mut bytes = skein_lib::Writer::new(usize::try_from(wire.measure()).unwrap());
            wire.encode(&mut bytes).unwrap();
            let wire = w::Refusal::decode(&limits, &mut skein_lib::Reader::new(&bytes.finish())).unwrap();
            let w::Refusal::Malformed(wire) = wire else { panic!("malformed") };
            assert_eq!(wire.at(), 2);
            let w::Malformed::Settings(facts) = wire.why() else { panic!("settings facts") };
            let client = jig_client_protocol_link::translate::settings::restore_malformed_settings(facts).unwrap();
            assert_eq!(
                client,
                c::MalformedSettings {
                    setting: c::Setting::Charter(c::SettingNumber { number: 5 }),
                    fault: c::SettingFault::Shape,
                    detail: Some(c::SettingDetail {
                        member: expected,
                        document: 3,
                        segment: 4,
                        rule: expected_rule,
                        supplied: 7,
                        bound: 6,
                    }),
                }
            );
        }
    }
}
