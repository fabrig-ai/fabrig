//! A project's follows, from settings and retained facts (domain/forge/follows.md, 7).
use crate::{FollowParity, FollowSkip, FollowState, settings};
use alloc::boxed::Box;
use core::mem::size_of_val;

#[must_use]
pub fn follows_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 40) | u64::from(project)
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowsView {
    pub project: u32,
    pub follows: Box<[FollowView]>,
    pub dependencies: Box<[DependencyView]>,
}
/// A dependency beside the follows (temper/docs/design/domain/forge/dependencies.md, 2 and 4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DependencyView {
    pub number: u32,
    pub project: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowView {
    pub number: u32,
    pub setting: settings::Follow,
    pub source: Box<[u8]>,
    pub source_branch: Box<[u8]>,
    pub target: Box<[u8]>,
    pub target_branch: Box<[u8]>,
    pub task: Option<u64>,
    pub stopped: Option<Box<[u8]>>,
    pub lag: u32,
    pub entries: Box<[FollowLagEntry]>,
    pub skips: Box<[FollowSkip]>,
    pub parity: Option<FollowParity>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowLagEntry {
    pub after: [u8; 32],
    pub change: Option<u64>,
    pub state: FollowState,
}
pub(super) fn owned_bytes(view: &FollowsView) -> usize {
    let mut size = size_of_val(view.follows.as_ref()).saturating_add(size_of_val(view.dependencies.as_ref()));
    for follow in &view.follows {
        size = size
            .saturating_add(follow.source.len())
            .saturating_add(follow.source_branch.len())
            .saturating_add(follow.target.len())
            .saturating_add(follow.target_branch.len())
            .saturating_add(follow.setting.source.branch.len())
            .saturating_add(follow.setting.target.branch.len())
            .saturating_add(size_of_val(follow.setting.source.paths.as_ref()))
            .saturating_add(follow.setting.check.workflow.len())
            .saturating_add(follow.setting.check.artifact.len())
            .saturating_add(size_of_val(follow.entries.as_ref()))
            .saturating_add(size_of_val(follow.skips.as_ref()));
        for path in &follow.setting.source.paths {
            size = size.saturating_add(path.len());
        }
        if let Some(producer) = &follow.setting.producer {
            size = size.saturating_add(producer.len());
        }
        if let Some(stopped) = &follow.stopped {
            size = size.saturating_add(stopped.len());
        }
        for entry in &follow.entries {
            match &entry.state {
                FollowState::HeldByOwner(words) | FollowState::Held(words) | FollowState::Ended(words) => {
                    size = size.saturating_add(words.len());
                }
                FollowState::NotMade | FollowState::WaitsWord | FollowState::WaitsFor(_) | FollowState::Producing => {}
            }
        }
        for skip in &follow.skips {
            size = size.saturating_add(skip.reason.len());
        }
        if let Some(parity) = &follow.parity {
            size = size.saturating_add(parity.run.len());
        }
    }
    size
}
