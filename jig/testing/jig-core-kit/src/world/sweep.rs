//! A sweep holds its prefix once (`jig/docs/design/domain/testing.md`, 4).
use super::{Cut, World};

impl World {
    /// Arm one sweep. Its children continue the caller's story at the selected
    /// cut; each must run its checks, then call `cut_passed` before returning.
    /// Compare `forked` (in execution order) with the reference's cuts afterward.
    /// A child cannot arm another sweep.
    pub fn sweep(&mut self, cuts: Vec<Cut>) {
        assert!(self.fork_child.is_none() && !self.case_child, "a forked child cannot arm a sweep");
        let disk = cuts
            .iter()
            .filter_map(|cut| match cut {
                Cut::Disk { from, operation } => Some((*from, *operation)),
                Cut::Submitted(_) | Cut::Durable(_) | Cut::Failed(_) => None,
            })
            .collect();
        self.store.sweep_kills(disk);
        self.sweep = cuts;
        self.forked.clear();
    }

    /// Fork one named case from this held setup. Only the parent returns, after
    /// the child's checks and closing oracle pass. Count successful returns
    /// against the reference's cases; an error names the failed case.
    pub fn case(&mut self, name: &str, check: impl FnOnce(&mut Self)) -> Result<(), String> {
        assert!(self.fork_child.is_none() && !self.case_child, "a forked child cannot run another case");
        match skein_fork::split() {
            skein_fork::Split::Child => {
                self.sweep.clear();
                self.store.sweep_kills(Vec::new());
                self.forked.clear();
                self.case_child = true;
                check(self);
                self.close();
                skein_fork::passed();
                unreachable!("a passed child exits");
            }
            skein_fork::Split::Parent(skein_fork::Exit::Code(0)) => Ok(()),
            skein_fork::Split::Parent(exit) => Err(format!("forked case {name} failed: {exit:?}")),
        }
    }

    /// The cut this child took, or None in the uncut parent. A complete disk
    /// boundary may have no following operation at which to pause.
    #[must_use]
    pub fn forked_cut(&self) -> Option<Cut> {
        self.fork_child
    }

    /// End a child only after its closing oracle. Return in the parent.
    pub fn cut_passed(&mut self) {
        if self.fork_child.is_some() {
            self.close();
            skein_fork::passed();
        }
    }

    pub(super) fn fork_cut(&mut self, cut: Cut) -> Result<bool, String> {
        let Some(index) = self.sweep.iter().position(|selected| *selected == cut) else {
            return Ok(false);
        };
        self.sweep.remove(index);
        match skein_fork::split() {
            skein_fork::Split::Child => {
                self.sweep.clear();
                self.store.sweep_kills(Vec::new());
                self.forked.clear();
                self.fork_child = Some(cut);
                self.cut = Some(cut);
                Ok(true)
            }
            skein_fork::Split::Parent(skein_fork::Exit::Code(0)) => {
                self.forked.push(cut);
                Ok(false)
            }
            skein_fork::Split::Parent(exit) => Err(format!("forked cut {cut:?} failed: {exit:?}")),
        }
    }

    pub(super) fn collect_disk_forks(&mut self) {
        let child = self.store.take_forked_kill();
        let passed = self.store.take_passed_kills();
        if let Some((from, operation)) = child {
            let cut = Cut::Disk { from, operation };
            self.sweep.clear();
            self.forked.clear();
            self.fork_child = Some(cut);
            self.cut = Some(cut);
        } else {
            self.forked.extend(passed.into_iter().map(|(from, operation)| Cut::Disk { from, operation }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_setup_children_keep_the_parent_and_report_case_failures() {
        let mut world = World::new(super::super::Setup { seed: 190, ..super::super::Setup::default() });
        world.turn(1, 1, None, b"durable answer");
        world.end(jig_domain_tasks::End::Parked, 1);
        let rows = world.store.rows.clone();
        let commits = world.store.applied;
        for name in ["first restart", "second restart"] {
            world
                .case(name, |child| {
                    child.restart(190, false);
                    assert!(child.domain.ready());
                    child.say(71);
                    assert!(child.store.applied > commits);
                })
                .expect("case passed");
        }
        assert_eq!(world.store.rows, rows, "children leave the held setup unchanged");
        assert_eq!(world.store.applied, commits);
        assert_eq!(
            world.case("failed route", |_| panic!("case check")),
            Err("forked case failed route failed: Code(101)".into())
        );
        assert_eq!(
            world.case("unread transcript", |child| child.store.unread_transcript(1)),
            Err("forked case unread transcript failed: Code(101)".into()),
            "a child cannot pass before its closing oracle"
        );
        assert!(!world.oracle_closed, "the parent's closing oracle still waits");
    }

    #[test]
    fn a_child_failure_names_its_cut() {
        let mut world = World::new(super::super::Setup::default());
        let cut = Cut::Submitted(world.store.applied + 1);
        world.sweep(vec![cut]);
        let result = world.fork_cut(cut);
        if result == Ok(true) {
            std::process::exit(23);
        }
        assert_eq!(result, Err(format!("forked cut {cut:?} failed: Code(23)")));
    }
}
