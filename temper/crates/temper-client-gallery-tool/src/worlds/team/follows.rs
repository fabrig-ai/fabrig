//! The standing role's lag and the reader's caught-up follow (client/web/gallery.md, team-role).
use super::{List, w};

const AFTER: [u8; 32] =
    [0x9c, 0x04, 0xd7, 0xe1, 0xa2, 0xb3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
fn follow(number: u32, owner: w::FollowOwner, source: &[u8], entries: List<w::FollowEntry>) -> w::FollowView {
    let repository = w::FollowRepository { forge: 1, repository: 7 };
    let setting = w::Follow {
            source: w::FollowSource {
                    repository: repository.clone(),
                    branch: b"main".as_slice().into(),
                    paths: List::with_capacity(0),
                },
            target: w::FollowTarget { repository, branch: b"main".as_slice().into() },
            owner,
            start: w::FollowStart::Owner,
            producer: None,
            verification: w::FollowCheck {
                    workflow: b".forgejo/workflows/check.yml".as_slice().into(),
                    artifact: b"results".as_slice().into(),
                    format: w::FollowFormat::Junit,
                },
        };
    let parity = if entries.is_empty() {
        None
    } else {
        Some(
            w::FollowParity {
                    source: AFTER,
                    head: [0xcd; 32],
                    verdict: w::FollowVerdict::Passed,
                    run: b"check".as_slice().into(),
                },
        )
    };
    w::FollowView {
            number,
            setting,
            source: source.into(),
            source_branch: b"main".as_slice().into(),
            target: b"acme/shop".as_slice().into(),
            target_branch: b"main".as_slice().into(),
            task: None,
            stopped: None,
            lag: entries.len(),
            entries,
            skips: List::with_capacity(0),
            parity,
        }
}
pub(super) fn panel(reader: u64) -> w::FollowsPanel {
    let entries = List::from_boxed(Box::new([
        w::FollowEntry {
                after: AFTER,
                change: None,
                state: w::FollowState::HeldByOwner(
                    w::FollowWords { words: b"waits for the handoff fix".as_slice().into() },
                ),
            },
        w::FollowEntry {
                after: [0xab; 32],
                change: Some(812),
                state: w::FollowState::WaitsFor(
                    w::FollowWait { change: 812 },
                ),
            },
    ]));
    w::FollowsPanel {
            project: super::PROJECT,
            follows: List::from_boxed(Box::new([
                follow(
                    1,
                    w::FollowOwner::Task(
                        w::FollowOwnerTask { task: 52 },
                    ),
                    b"acme/upstream",
                    entries,
                ),
                follow(
                    2,
                    w::FollowOwner::Person(
                        w::FollowPerson { person: reader },
                    ),
                    b"acme/tools",
                    List::with_capacity(0),
                ),
            ])),
            dependencies: List::with_capacity(0),
        }
}
