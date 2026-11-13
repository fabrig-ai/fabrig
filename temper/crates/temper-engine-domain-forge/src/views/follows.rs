//! A project's follows, from settings and retained facts (domain/forge/follows.md, 7).
use crate::{Domain, Limits};
use alloc::boxed::Box;
use skein_lib::{List, Writer};
pub use temper_engine_domain_forge_vocabulary::views::follows::*;

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
