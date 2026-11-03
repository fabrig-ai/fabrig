//! Process producers edit and deliver; Open chats inspect
//! (temper/docs/design/domain/agent.md, 4.3).
use jig_domain_tasks as tasks;
use temper_engine_domain as engine;

fn assignment(charter: &engine::Charter, ended_by: tasks::EndedBy) -> engine::Assignment {
    engine::Assignment {
        task: 1,
        attempt: 1,
        conversation: 1,
        live: jig_domain_host::LiveAsk::NONE,
        charter: charter.number,
        run: Box::new(engine::RunCharter {
            revision: 1,
            ended_by,
            sources: Box::new([Some(1)]),
            accounts: Box::new([]),
            placement: jig_domain_fleet::PlacementConstraint {
                channel: skein_lib::Token::new(1),
                kind: jig_domain_fleet::HostKind::Worker,
            },
            policy: charter.policy.clone(),
            contract: tasks::Contract::Change { connector: 1, kind: 1, words: 128 },
            authority: tasks::Authority {
                tools: tasks::Tools(1),
                models: tasks::Models::NONE,
                grants: Box::new([]),
                delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
                budget: tasks::Budget { spend: charter.spend, deadline: None },
                notes: tasks::Scopes(0),
                note_resources: Box::new([]),
            },
            budget: charter.spend,
        }),
        sections: Box::new([]),
        messages: Box::new([]),
        saved: Box::new([]),
        workstream: 1,
        workspace: engine::ForgeWorkspace { repositories: Box::new([]) },
        transcript: Box::new([]),
        answered: Box::new([]),
        settled: Box::new([]),
        grants: Box::new([]),
    }
}

#[test]
fn process_producers_can_write_and_deliver_while_open_chats_cannot() {
    let (_, bytes) = crate::support::configuration::documents(std::path::Path::new("/fixture"), "/fixture/temper");
    let deployment: serde_json::Value = serde_json::from_slice(&bytes).expect("process configuration");
    let configured = temper_real_loop_world::settings::configuration(&deployment);
    let defaults = temper_service::stock::defaults(&configured.llm);
    let number = deployment["application"]["charter"].as_u64().expect("producer charter");
    let producer = defaults.charters.iter().find(|charter| u64::from(charter.number) == number).expect("producer");
    let finite = temper_engine_smith::charter(&assignment(producer, tasks::EndedBy::Result));
    assert_eq!(finite.mode, smith_domain_run::Mode::Finite);
    assert!(finite.grants.deliver.is_some());
    assert!(finite.grants.tools.modify, "process producer needs workspace edits");
    let mut report = assignment(producer, tasks::EndedBy::Result);
    report.run.contract = tasks::Contract::Report { words: 128 };
    let report = temper_engine_smith::charter(&report);
    assert!(report.grants.deliver.is_none());
    assert!(!report.grants.tools.modify, "a report has no delivery path");
    let chat = defaults.charters.iter().find(|charter| charter.number == 1).expect("stock chat");
    assert!(!chat.policy.modify, "stock chat inspects");
    for charter in [chat, producer] {
        let open = temper_engine_smith::charter(&assignment(charter, tasks::EndedBy::Requester));
        assert_eq!(open.mode, smith_domain_run::Mode::Open);
        assert!(open.grants.tools.inspect);
        assert!(open.grants.deliver.is_none());
        assert!(!open.grants.tools.modify, "Open runs have no delivery path");
    }
}
