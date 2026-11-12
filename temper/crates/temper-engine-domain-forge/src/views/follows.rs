//! A project's follows, from settings and retained facts (domain/forge/follows.md, 7).
use crate::{Domain, FollowParity, FollowSkip, FollowState, Limits, settings};
use alloc::boxed::Box;
use core::mem::size_of_val;
use skein_lib::{List, Writer};

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
fn name(owner: &[u8], name: &[u8]) -> Box<[u8]> {
    let mut out = Writer::new(owner.len().saturating_add(name.len()).saturating_add(1));
    out.put(owner).expect("name room");
    out.put(b"/").expect("name room");
    out.put(name).expect("name room");
    out.finish()
}
impl Domain {
    pub(super) fn follows_view(&self, project: u32, limits: &Limits) -> Option<FollowsView> {
        let mut dependencies = List::with_capacity(limits.dependencies);
        for (number, value) in self.dependencies(project) {
            dependencies.push(DependencyView { number, project: value.project }).ok()?;
        }
        // Dependencies need no repository facts (temper/docs/design/domain/forge/dependencies.md, 4).
        let follows = self.follow_views(project, limits, !dependencies.is_empty())?;
        Some(FollowsView { project, follows, dependencies: dependencies.into_boxed() })
    }

    fn follow_views(&self, project: u32, limits: &Limits, with_dependencies: bool) -> Option<Box<[FollowView]>> {
        let mut follows = List::with_capacity(limits.follows);
        for (number, setting) in self.follows(project) {
            let (Some(source), Some(target)) =
                (self.repository(setting.source.repository), self.repository(setting.target.repository))
            else {
                if with_dependencies {
                    continue;
                }
                return None;
            };
            let row = self.follow_row(project, number);
            let mut entries = List::with_capacity(limits.follow_queue);
            if let Some(row) = row {
                for entry in &row.queue {
                    entries
                        .push(FollowLagEntry { after: entry.after, change: entry.change, state: entry.state.clone() })
                        .ok()?;
                }
            }
            follows
                .push(FollowView {
                    number,
                    setting: setting.clone(),
                    source: name(&source.owner, &source.name),
                    source_branch: if setting.source.branch.is_empty() {
                        source.settings.default_branch.clone()
                    } else {
                        setting.source.branch.clone()
                    },
                    target: name(&target.owner, &target.name),
                    target_branch: if setting.target.branch.is_empty() {
                        target.settings.default_branch.clone()
                    } else {
                        setting.target.branch.clone()
                    },
                    task: match row {
                        Some(row) => row.task,
                        None => None,
                    },
                    stopped: match row {
                        Some(row) => row.stopped.clone(),
                        None => None,
                    },
                    lag: entries.len(),
                    entries: entries.into_boxed(),
                    skips: match row {
                        Some(row) => row.skips.clone(),
                        None => Box::new([]),
                    },
                    parity: match row {
                        Some(row) => row.parity.clone(),
                        None => None,
                    },
                })
                .ok()?;
        }
        Some(follows.into_boxed())
    }
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
