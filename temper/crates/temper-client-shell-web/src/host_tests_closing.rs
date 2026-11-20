//! A fresh load must wait for the saved playground start's answer (client/web/README.md, 2.1).
use super::*;
use jig_client_protocol_wire as w;
use temper_client_protocol_wire as wire;

#[test]
fn the_playground_start_warns_only_until_its_answer_is_carried() {
    let mut host = super::tab::linked();
    host.client(jig_client::Event::Send {
        key: jig_client::Key { asker: jig_client::Asker::JIG, number: 99 },
        name: None,
        request: root::forge::playground::ask(&host.env.limits.forge).expect("playground start"),
    });
    let saving = host.report(Report::Start);
    let [Asked::Save { key, value }] = saving.asked.as_slice() else { panic!("save before sending") };
    let mut stored = vec![skein_ui_web::store::Record { key: key.clone(), value: value.clone() }];
    assert!(records::closing_loses(&stored), "the unanswered start would be lost");

    let sent = host.report(Report::Carried { reports: Vec::new(), writes: saving.writes });
    let [Asked::Exchange { id, call, .. }] = sent.asked.as_slice() else { panic!("send after saving") };
    assert_eq!(call.path.as_ref(), b"/jig/v1/request");
    assert!(records::closing_loses(&stored), "sending is not an answer");

    let answer = wire::v1::Answer { outcome: wire::v1::Outcome::Running };
    let answer = w::KeyedAnswer::FinalAnswer(w::Owned {
        owner: w::Owner::Family(w::Family { number: wire::FORGE }),
        version: wire::CLIENT_VERSION,
        record: wire::answer(&answer),
    });
    let answered = host.report(Report::Heard(Heard::Answered {
        exchange: *id,
        status: 200,
        body: answer.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding"),
        headers: Vec::new(),
    }));
    let [Asked::Erase { key }] = answered.asked.as_slice() else { panic!("erase the answered start") };
    stored.retain(|record| record.key != *key);
    assert!(stored.is_empty(), "the saved start is erased by its own key");
    assert!(!records::closing_loses(&stored), "the answered start cannot be lost");
}
