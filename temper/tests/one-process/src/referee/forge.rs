//! Outside branch moves and completed shell writes, independent of either root.
use crate::{
    fixtures,
    world::{Proc, Pushed, Story},
};
use skein_fake_forge_domain::{Observation, api};
use std::collections::BTreeMap;
use temper_playground::programs::{Harness, Process};

pub(crate) struct Forge {
    transport: temper_fake_git_hosted::Typed,
    tree: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) pushed: Vec<Pushed>,
    story: Story,
    edited: bool,
    prepared: bool,
    hooks: Option<std::net::SocketAddr>,
}
impl Forge {
    pub(crate) fn new(transport: temper_fake_git_hosted::Typed, story: Story) -> Self {
        let content: &[u8] = if matches!(story, Story::Merge) { b"topic\n" } else { b"base\n" };
        Self {
            transport,
            tree: BTreeMap::from([(b"conflict.txt".to_vec(), content.to_vec())]),
            pushed: Vec::new(),
            story,
            edited: false,
            prepared: false,
            hooks: None,
        }
    }
    pub(crate) fn hand_hooks(&mut self, bound: std::net::SocketAddr) {
        if matches!(self.story, Story::PlaygroundA) {
            return;
        }
        if self.hooks != Some(bound) {
            self.transport.forge.borrow_mut().hooks = Some(skein_fake_forge_protocol::delivery::Endpoint {
                forge: 1,
                host: bound.to_string().into_bytes().into(),
                secret: skein_lib::Secret::new(fixtures::HOOK_SECRET.into()),
            });
            self.hooks = Some(bound);
        }
    }
    pub(crate) fn adopted(&mut self) {
        if matches!(self.story, Story::Merge) && !self.prepared {
            crate::stories::prepare_merge_branch(&self.transport);
            self.prepared = true;
        }
    }
    pub(crate) fn observe<H: Harness>(&mut self, procs: &[Process<H>]) {
        for proc in procs {
            if let Process::Forge(peer) = proc {
                assert!(
                    peer.address().is_none() || peer.address() == Some(fixtures::FORGE),
                    "forge binds the configured address"
                );
            }
            if let Process::Shell(shell) = proc
                && !shell.written.is_empty()
            {
                self.tree.insert(shell.target.clone(), shell.written.clone());
                self.edited = true;
            }
        }
        let mut forge = self.transport.forge.borrow_mut();
        while let Some(observed) = forge.domain.pop_observation() {
            if let Observation::Merged { repository, head, .. } = &observed {
                green(&forge, repository, *head);
            }
            let Observation::Moved { repository, branch, from, to, by } = observed else { continue };
            if by != 1 && !matches!(self.story, Story::PlaygroundA) {
                continue;
            }
            let metadata = forge.domain.metadata(&repository).expect("observed repository");
            if branch == metadata.default_branch {
                green(&forge, &repository, to);
            }
            if matches!(self.story, Story::PlaygroundA) {
                if from != Some(to) && branch != metadata.default_branch {
                    let object = forge.domain.object(to).expect("pushed object");
                    self.pushed.push(Pushed {
                        branch,
                        commit: to,
                        parents: object.parent.into_iter().chain(object.merge_parent).collect(),
                    });
                }
                continue;
            }
            // Git may create the remote branch at the editor's first committed change.
            // Earlier preparation has no completed agent write.
            if !self.edited || from == Some(to) || branch == metadata.default_branch {
                continue;
            }
            let object = forge.domain.object(to).expect("observed pushed object");
            let tree: BTreeMap<_, _> =
                object.tree.iter().map(|(path, bytes)| (path.to_vec(), bytes.to_vec())).collect();
            assert_eq!(tree, self.tree, "pushed tree is the completed agent edits");
            let parents = object.parent.into_iter().chain(object.merge_parent).collect();
            self.pushed.push(Pushed { branch, commit: to, parents });
        }
    }
    pub(crate) fn hooks_complete(procs: &[Proc], hooks: usize) -> bool {
        procs.iter().any(|proc| {
            let Proc::Forge(peer) = proc else { return false };
            let deliveries = peer
                .observations()
                .iter()
                .filter(|record| matches!(record, skein_fake_peers::forge::Observation::Delivery { .. }))
                .count();
            deliveries == hooks
        })
    }
    pub(crate) fn require_hooks(&self, hooks: usize) {
        assert_eq!(
            usize::try_from(self.transport.forge.borrow().domain.tally().hooks).expect("hook tally fits"),
            hooks,
            "the story's own hooks have all been released"
        );
    }
    pub(crate) fn complete(&self) -> bool {
        if matches!(self.story, Story::PlaygroundA) {
            if crate::world::playground_answers() != 3 {
                return false;
            }
            let outcome = crate::stories::playground::outcome(&self.transport);
            return outcome.csv
                && outcome.green
                && outcome.answered == 3
                && self.pushed.iter().any(|push| push.commit == outcome.main);
        }
        self.pushed.iter().any(|push| {
            !matches!(self.story, Story::Merge) || push.parents == [fixtures::BRANCH_HEAD, fixtures::MOVED_BASE]
        })
    }
}
fn green(forge: &skein_fake_peers::forge::Forge, repository: &[u8], head: u64) {
    let api::Answer::Checks(checks) = forge
        .domain
        .inspect(&forge.config, repository, &api::Read::Checks { commit: head })
        .expect("landing CI statuses")
    else {
        panic!("CI status answer")
    };
    assert!(
        !checks.is_empty() && checks.iter().all(|check| check.status.state == api::Check::Passed),
        "nothing lands on a default branch without green CI"
    );
}
