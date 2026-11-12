//! The standing role's lag and the reader's caught-up follow (client/web/gallery.md, team-role).
use super::{List, w};

const AFTER: [u8; 32] = [0x9c, 0x04, 0xd7, 0xe1, 0xa2, 0xb3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
fn follow(number: u32, owner: w::FollowOwner, source: &[u8], entries: List<w::FollowEntry>) -> w::FollowView {
    let repository = w::FollowRepository::new(&w::CEILINGS, w::FollowRepositoryParts { forge: 1, repository: 7 }).expect("gallery repository");
    let setting = w::Follow::new(&w::CEILINGS, w::FollowParts {
        source: w::FollowSource::new(&w::CEILINGS, w::FollowSourceParts { repository: repository.clone(), branch: b"main".as_slice().into(), paths: List::with_capacity(0) }).expect("gallery source"),
        target: w::FollowTarget::new(&w::CEILINGS, w::FollowTargetParts { repository, branch: b"main".as_slice().into() }).expect("gallery target"),
        owner, start: w::FollowStart::Owner, producer: None,
        verification: w::FollowCheck::new(&w::CEILINGS, w::FollowCheckParts {
            workflow: b".forgejo/workflows/check.yml".as_slice().into(), artifact: b"results".as_slice().into(), format: w::FollowFormat::Junit,
        }).expect("gallery check"),
    }).expect("gallery follow");
    let parity = if entries.is_empty() { None } else {
        Some(w::FollowParity::new(&w::CEILINGS, w::FollowParityParts { source: AFTER, head: [0xcd; 32], verdict: w::FollowVerdict::Passed, run: b"check".as_slice().into() }).expect("gallery parity"))
    };
    w::FollowView::new(&w::CEILINGS, w::FollowViewParts {
        number, setting, source: source.into(), source_branch: b"main".as_slice().into(), target: b"acme/shop".as_slice().into(), target_branch: b"main".as_slice().into(),
        task: None, stopped: None, lag: entries.len(), entries, skips: List::with_capacity(0), parity,
    }).expect("gallery follow view")
}
pub(super) fn panel(reader: u64) -> w::FollowsPanel {
    let entries = List::from_boxed(Box::new([
        w::FollowEntry::new(&w::CEILINGS, w::FollowEntryParts { after: AFTER, change: None,
            state: w::FollowState::HeldByOwner(w::FollowWords::new(&w::CEILINGS, w::FollowWordsParts { words: b"waits for the handoff fix".as_slice().into() }).expect("gallery held words")),
        }).expect("gallery held entry"),
        w::FollowEntry::new(&w::CEILINGS, w::FollowEntryParts { after: [0xab; 32], change: Some(812),
            state: w::FollowState::WaitsFor(w::FollowWait::new(&w::CEILINGS, w::FollowWaitParts { change: 812 }).expect("gallery wait")),
        }).expect("gallery waiting entry"),
    ]));
    w::FollowsPanel::new(&w::CEILINGS, w::FollowsPanelParts { project: super::PROJECT,
        follows: List::from_boxed(Box::new([
            follow(1, w::FollowOwner::Task(w::FollowOwnerTask::new(&w::CEILINGS, w::FollowOwnerTaskParts { task: 52 }).expect("gallery role owner")), b"acme/upstream", entries),
            follow(2, w::FollowOwner::Person(w::FollowPerson::new(&w::CEILINGS, w::FollowPersonParts { person: reader }).expect("gallery person owner")), b"acme/tools", List::with_capacity(0)),
        ])), dependencies: List::with_capacity(0),
    }).expect("gallery follows panel")
}
