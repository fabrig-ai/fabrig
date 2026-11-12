use super::fixtures as f;
use crate::forge::drafts;
use jig_client::{Ask, Family, Watchable, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[test]
fn watch_names_the_projects_drafts_with_no_window() {
    let Watchable::Family { scope, window, record } = drafts::watch(f::PROJECT, &f::limits().forge).unwrap() else {
        panic!("family watch")
    };
    assert_eq!(scope, c::Scope::Project(f::PROJECT));
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(
        wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch(),
        &w::Watch::Drafts(w::DraftsSubject::new(&w::CEILINGS, w::DraftsSubjectParts { project: f::PROJECT }).unwrap())
    );
}

#[test]
fn panel_accepts_only_a_complete_drafts_family_record() {
    let panel =
        w::DraftsPanel::new(&w::CEILINGS, w::DraftsPanelParts { project: f::PROJECT, drafts: List::with_capacity(0) })
            .unwrap();
    let record = Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::drafts_panel(&panel) };
    assert_eq!(drafts::panel(&record, f::limits().forge), Some(panel));
    let other = w::Watching::new(&w::CEILINGS, w::WatchingParts { watch: w::Watch::Away }).unwrap();
    let mut trailing = skein_lib::Writer::new(record.bytes.len().checked_add(1).unwrap());
    trailing.put(&record.bytes).unwrap();
    trailing.put(&[0]).unwrap();
    for bad in [
        Family { family: wire::number(wire::Owner::Pages), ..record.clone() },
        Family { version: wire::CLIENT_VERSION + 1, ..record.clone() },
        Family { bytes: wire::watching(&other), ..record.clone() },
        Family { bytes: trailing.finish(), ..record },
    ] {
        assert_eq!(drafts::panel(&bad, f::limits().forge), None);
    }
}

#[test]
fn ask_carries_set_draft_in_the_projects_scope() {
    let repository = w::DraftRepository::new(
        &w::CEILINGS,
        w::DraftRepositoryParts { owner: b"acme".as_slice().into(), name: b"shop".as_slice().into() },
    )
    .unwrap();
    let parts = w::SetDraftParts {
        project: f::PROJECT,
        repository: Some(repository),
        path: b"docs/design/draft/catch-up.md".as_slice().into(),
        change: w::DraftChange::Ready(w::ReadyDraft::new(&w::CEILINGS, w::ReadyDraftParts { chat: Some(7) }).unwrap()),
    };
    let Ask::Family { scope, record } = drafts::ask(
        parts.project,
        parts.repository.clone(),
        parts.path.clone(),
        parts.change.clone(),
        &f::limits().forge,
    )
    .unwrap() else {
        panic!("family ask")
    };
    assert_eq!(scope, c::Scope::Project(f::PROJECT));
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(
        wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request,
        w::Request::SetDraft(w::SetDraft::new(&w::CEILINGS, parts).unwrap()),
    );
}
