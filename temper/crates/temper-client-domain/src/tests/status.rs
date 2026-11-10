//! Status's project-scoped watch, request and delivered record (client/forge/records.md).
use super::fixtures as f;
use crate::forge::status::{self, Answer};
use jig_client::{Ask, Family, Outcome, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) fn document(landed: u64, seen: u64, brief: Option<u64>) -> w::ProjectStatus {
    w::ProjectStatus::new(
        &w::CEILINGS,
        w::ProjectStatusParts {
            project: f::PROJECT,
            landed,
            last_landed: Some(100),
            brief: match brief {
                Some(landed) => Some(
                    w::StatusBrief::new(
                        &w::CEILINGS,
                        w::StatusBriefParts {
                            sentence: b"A change landed.".as_slice().into(),
                            lede: b"The change is ready.".as_slice().into(),
                            at: 100,
                            landed,
                        },
                    )
                    .unwrap(),
                ),
                None => None,
            },
            previous: Some(50),
            seen,
            queued: 2,
            red: true,
            branch: b"main".as_slice().into(),
        },
    )
    .unwrap()
}
pub(super) fn record(document: &w::ProjectStatus) -> Family {
    Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::project_status(document) }
}
fn answer(outcome: w::Outcome) -> Outcome {
    Outcome::Answered(jig_client::Answered::Family(Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
    }))
}
fn trailing(bytes: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(bytes.len().checked_add(1).unwrap());
    writer.put(bytes).unwrap();
    writer.put(&[0]).unwrap();
    writer.finish()
}
#[test]
fn watch_and_look_name_the_projects_scope() {
    for project in [0, f::PROJECT, u32::MAX] {
        let Watchable::Family { scope, window, record } = status::watch(project, &f::limits().forge).unwrap() else {
            panic!("family watch")
        };
        assert_eq!(scope, c::Scope::Project(project));
        assert_eq!(window, c::Window { rows: 0, after: None });
        assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
        let w::Watch::Status(subject) = wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().into_parts().watch
        else {
            panic!("status watch")
        };
        assert_eq!(subject.project(), project);
        let Ask::Family { scope, record } = status::ask(project, &f::limits().forge).unwrap() else {
            panic!("family request")
        };
        assert_eq!(scope, c::Scope::Project(project));
        assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
        let w::Request::Looked(looked) =
            wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        else {
            panic!("look request")
        };
        assert_eq!(looked.project(), project);
    }
    let mut limits = f::limits().forge;
    limits.field_bytes = u32::MAX;
    assert!(status::watch(f::PROJECT, &limits).is_none());
    assert!(status::ask(f::PROJECT, &limits).is_none());
}
#[test]
fn documents_require_the_family_version_and_valid_bytes() {
    let document = document(4, 2, Some(3));
    let valid = record(&document);
    assert_eq!(status::document(&valid, f::limits().forge), Some(document));
    for record in [
        Family { family: wire::FORGE + 1, ..valid.clone() },
        Family { version: wire::CLIENT_VERSION + 1, ..valid.clone() },
        Family { bytes: Box::default(), ..valid.clone() },
        Family { bytes: trailing(&valid.bytes), ..valid.clone() },
    ] {
        assert_eq!(status::document(&record, f::limits().forge), None);
    }
}
#[test]
fn only_looked_and_unknown_are_look_answers() {
    assert!(match status::answer(&answer(w::Outcome::Looked), f::limits().forge) {
        Answer::Looked => true,
        Answer::Unknown | Answer::Refused => false,
    });
    for outcome in [Outcome::Unknown, answer(w::Outcome::Unknown)] {
        assert!(match status::answer(&outcome, f::limits().forge) {
            Answer::Unknown => true,
            Answer::Looked | Answer::Refused => false,
        });
    }
    let Outcome::Answered(jig_client::Answered::Family(valid)) = answer(w::Outcome::Looked) else {
        panic!("family answer")
    };
    for outcome in [
        answer(w::Outcome::Dismissed),
        answer(w::Outcome::Removed),
        Outcome::Answered(jig_client::Answered::Family(Family { family: wire::FORGE + 1, ..valid.clone() })),
        Outcome::Answered(jig_client::Answered::Family(Family { version: wire::CLIENT_VERSION + 1, ..valid.clone() })),
        Outcome::Answered(jig_client::Answered::Family(Family { bytes: trailing(&valid.bytes), ..valid.clone() })),
        Outcome::Answered(jig_client::Answered::Family(Family { bytes: Box::default(), ..valid })),
        Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None })),
        Outcome::NotKept,
        Outcome::Limit,
        Outcome::Unconfirmed,
        Outcome::Held,
        Outcome::Unsent,
        Outcome::Full { held: 1, room_at: skein_lib::Wall::EPOCH },
        Outcome::Lost,
    ] {
        assert!(
            match status::answer(&outcome, f::limits().forge) {
                Answer::Refused => true,
                Answer::Looked | Answer::Unknown => false,
            },
            "{outcome:?}"
        );
    }
}
