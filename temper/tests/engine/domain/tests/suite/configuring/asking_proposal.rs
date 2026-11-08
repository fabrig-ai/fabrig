//! Ask temper's proposal read (temper/docs/design/domain/agent.md, 8.4.1).
use super::*;
use serde_json::json;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one stock-world table reads task and party proposals across their decisions"
)]
fn an_asking_chat_reads_a_proposal() {
    let mut world = World::stock();
    let owner = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &owner, 1, CHAT);
    let items = charter_item(value, b"Push each complete change.");
    let first = propose(&mut world.driver, &owner, 2, 1, &items);
    let asking = world.chat(5, world.owner, 21);
    let mut party = None;
    for (index, state) in
        ["waiting", "rejected", "accepted", "unknown", "party waiting", "party rejected"].into_iter().enumerate()
    {
        let number = match state {
            "waiting" => first,
            "rejected" => {
                world.decide(
                    &owner,
                    first,
                    22,
                    people::ProposalDecision::Reject { reason: b"Keep each change complete.".as_slice().into() },
                    people::Running::Stays,
                );
                first
            }
            "accepted" => {
                let second = propose(&mut world.driver, &owner, 3, 1, &items);
                world.decide(&owner, second, 23, people::ProposalDecision::Accept, people::Running::Stays);
                second
            }
            "unknown" => 999,
            "party waiting" => {
                world.driver.send(Event::Party(PartyInput::ProposeSettings {
                    reply_to: ReplyTo::new(Token::new(24)),
                    sign_in: world.owner,
                    key: [24; 16],
                    digest: [24; 32],
                    kinds: Box::new([]),
                    change: Box::new(s::Proposed {
                        against: 2,
                        items: Box::new([owner_charter(b"Keep proposals reviewable.")]),
                        reason: REASON.as_bytes().into(),
                    }),
                }));
                world.driver.settle();
                let people::Reply::Outcome(people::Outcome::Proposed { proposal, .. }) = world.reply(24) else {
                    panic!("party's proposal: {:?}", world.reply(24))
                };
                party = Some(*proposal);
                *proposal
            }
            "party rejected" => {
                let proposal = party.unwrap();
                world.ask(
                    world.owner,
                    25,
                    people::Ask::DecideProposal {
                        project: people::DEPLOYMENT,
                        proposer: people::Proposer::Party(OWNER_PERSON),
                        proposal,
                        decision: people::ProposalDecision::Reject {
                            reason: b"Keep each change complete.".as_slice().into(),
                        },
                        running: None,
                    },
                );
                assert!(matches!(world.reply(25), people::Reply::Outcome(people::Outcome::ProposalDecided { .. })));
                proposal
            }
            _ => unreachable!(),
        };
        let address = if state.starts_with("party") {
            format!("proposal p{OWNER_PERSON}/{number}")
        } else {
            format!("proposal t{}/{number}", owner.task)
        };
        let before = world.driver.store.rows.clone();
        let (body, error) = host_tool(
            &mut world.driver,
            &asking,
            u32::try_from(index + 1).unwrap(),
            b"see",
            json!({"address":address}).to_string().as_bytes(),
        );
        let text = std::str::from_utf8(&body).unwrap();
        assert_eq!(error, state == "unknown", "{state}: {text}");
        if state == "unknown" {
            assert_eq!(
                text,
                format!(
                    "see {address}: not read: no such proposal of settings. This says nothing about the settings: see settings lists those waiting."
                )
            );
        } else {
            let outcome = match state {
                "waiting" => "waiting, against revision 1",
                "rejected" | "party rejected" => "rejected: Keep each change complete.",
                "accepted" => "accepted as revision 2",
                "party waiting" => "waiting, against revision 2",
                _ => unreachable!(),
            };
            assert!(text.starts_with(&format!("{address} of settings, {outcome}\n")), "{text}");
            assert!(text.contains("reason:\n  Explain running work\n"), "{text}");
            assert!(text.contains("items:\n  charter 1: set\n"), "{text}");
            if state.ends_with("waiting") {
                let (revision, instructions) = if state == "waiting" {
                    (1, "Push each complete change.")
                } else {
                    (2, "Keep proposals reviewable.")
                };
                assert!(text.contains(&format!("review at revision {revision}:\n")), "{text}");
                let (_, proposed) = text.split_once("would be:\n").expect("proposed value");
                assert!(proposed.contains(instructions), "{text}");
                if state == "waiting" {
                    eprintln!("waiting answer:\n{text}");
                }
            } else {
                assert!(!text.contains("would be:"), "decided proposals keep heads alone: {text}");
            }
            assert_eq!(text.lines().last(), Some("see: revision N · revisions · help proposal"));
            let end = text.find("more: ").or_else(|| text.find("see: ")).unwrap();
            assert!(end <= usize::try_from(temper_engine_smith::SEE_PART).unwrap());
        }
        assert!(body.len() <= usize::try_from(world.driver.env.limits.call_answer_bytes - 3).unwrap());
        if let Some(bound) = world.driver.env.limits.host_reply_bytes {
            assert!(body.len() <= usize::try_from(bound).unwrap());
        }
        assert_eq!(world.driver.store.rows, before, "{address} commits nothing");
    }
}
