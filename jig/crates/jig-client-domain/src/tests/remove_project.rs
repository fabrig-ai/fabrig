use super::fixtures::{Client, KEY, PARTY, PROJECT, me};
use crate::*;
use skein_lib::{Duration, RequestEnvelope, Time};

// Leave time for retries beyond the fixture table's one-second margin.
const RETENTION: Duration = Duration::from_secs(10);

#[test]
fn a_remove_project_request_is_kept_by_its_key_and_answered_once() {
    let mut client = Client::live();
    client.step(Event::Up { up: me(PARTY, RETENTION) });
    client.told();
    let request = Ask::Jig(Box::new(contract::Ask::RemoveProject { project: PROJECT }));
    client.step(Event::Send { name: None, key: KEY, request: request.clone() });
    let downs = client.downs();
    let [Down::Save { record }, Down::Send { attempt, key, request: sent, .. }] = downs.as_ref() else {
        panic!("save before sending deletion");
    };
    assert_eq!(record.request, Held { key: KEY, request: request.clone() });
    assert_eq!(record.scope, PARTY);
    assert_eq!(*key, record.key);
    assert_eq!(*sent, request);
    client.told();
    client.step(Event::Up {
        up: Up::Answer { attempt: *attempt, envelope: RequestEnvelope::Lost, answer: None, why: Some(Retry::Lost) },
    });
    client.told();
    client.at(Time::ZERO.saturating_add(Duration::from_millis(80)));
    let retry = client.downs();
    let [Down::Send { attempt: retried, key: retry_key, request: retry_request, .. }] = retry.as_ref() else {
        panic!("deletion retried under its saved key");
    };
    assert_eq!(retry_key, key);
    assert_ne!(retried, attempt);
    assert_eq!(*retry_request, request);
    client.told();
    let answer =
        Answered::Jig(contract::Answer::Done(contract::Outcome::Owner(contract::OwnerOutcome::ProjectRemoved {
            project: PROJECT,
        })));
    client.step(Event::Up {
        up: Up::Answer { attempt: *retried, envelope: RequestEnvelope::Final, answer: Some(answer.clone()), why: None },
    });
    assert_eq!(client.told().as_ref(), &[Told::Answered { key: KEY, outcome: Outcome::Answered(answer.clone()) }]);
    assert_eq!(client.downs().as_ref(), &[Down::Erase { key: *key }]);
    client.step(Event::Up {
        up: Up::Answer { attempt: *retried, envelope: RequestEnvelope::Final, answer: Some(answer), why: None },
    });
    assert!(client.told().is_empty());
    assert!(client.downs().is_empty());
    assert_eq!(client.domain.pending(KEY), None);
}
