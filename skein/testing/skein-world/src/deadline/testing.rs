use super::{Deadline, Duration, Time};

/// Arms a deadline at the script's time with no scheduler sources.
#[must_use]
pub fn at(what: &'static str, patience: Duration, start: Time) -> Deadline {
    Deadline::at(what, patience, start)
}

#[cfg(target_os = "linux")]
pub(crate) mod linux {
    use super::super::{Deadline, Duration, Proc, TreeWatch};
    use crate::end_to_end::Binary;
    use crate::tree::Tree;
    use core::fmt;

    #[derive(Debug)]
    pub enum Watch {
        Cgroup { start: Option<u64> },
        Threads(usize),
    }

    pub struct Watches<'a>(&'a Deadline);

    impl fmt::Debug for Watches<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.trees.fmt(f)
        }
    }

    impl Watches<'_> {
        #[must_use]
        pub fn count(&self) -> usize {
            self.0.trees.len()
        }

        #[must_use]
        pub fn first(&self) -> Option<Watch> {
            match self.0.trees.first()? {
                TreeWatch::Cgroup { start, .. } => Some(Watch::Cgroup { start: *start }),
                TreeWatch::Threads(threads) => Some(Watch::Threads(threads.len())),
            }
        }

        #[must_use]
        pub fn waits(&self) -> Vec<Option<Duration>> {
            self.0.trees.iter().map(|tree| tree.waited(&Proc::Live, self.0.boot).0).collect()
        }
    }

    #[derive(Debug)]
    pub struct ThreadBaseline {
        pub cpu: u64,
        pub waited: Option<u64>,
    }

    #[must_use]
    pub fn watches(deadline: &Deadline) -> Watches<'_> {
        Watches(deadline)
    }

    #[must_use]
    pub fn thread_baseline(deadline: &Deadline) -> Option<ThreadBaseline> {
        deadline.thread.map(|sched| ThreadBaseline { cpu: sched.cpu, waited: sched.waited })
    }

    #[must_use]
    pub fn has_boot_origin(deadline: &Deadline) -> bool {
        deadline.boot.is_some()
    }

    #[must_use]
    pub fn has_cgroup(binary: &Binary) -> bool {
        binary.keeper().and_then(Tree::cgroup).is_some()
    }

    #[must_use]
    pub fn figure(value: Option<Duration>) -> String {
        super::super::figure(value)
    }
}
