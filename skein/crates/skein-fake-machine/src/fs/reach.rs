//! Landlock layers for the modeled probe (testing/README.md, 4).
use super::{Body, Machine, NodeId, Opened, READ, Refusal, SEARCH, WRITE};
use crate::probe::{self, Answer, Ids, Question, Report};
use crate::{Landlock, kernel_error};
use skein_io::kernel::{Error, Pipe, Rights, Unenforced};
use skein_sim::machine::Reach;

#[derive(Clone)]
pub(crate) struct Grant {
    node: NodeId,
    rights: Rights,
}

impl Machine {
    /// Sets the Landlock available to subsequent spawns; ABI 6 by default.
    pub fn set_landlock(&mut self, landlock: Landlock) {
        self.landlock = landlock;
    }

    pub(crate) fn reach(&self, reach: &Reach) -> Result<Vec<Grant>, Error> {
        self.reach_abi(reach.abi)?;
        let mut layer = Vec::new();
        for (index, rule) in reach.rules.iter().enumerate() {
            let root = self.handle(Opened::new(rule.root.raw())).node;
            let result = self.rule(root, &rule.path, rule.rights);
            match result {
                Ok(grant) => layer.push(grant),
                Err(refusal) => return Err(rule_error(index, refusal)),
            }
        }
        Ok(layer)
    }

    fn reach_abi(&self, abi: u32) -> Result<(), Error> {
        match self.landlock {
            Landlock::Absent => return Err(Error::Reach(Unenforced::Absent)),
            Landlock::Disabled => return Err(Error::Reach(Unenforced::Disabled)),
            Landlock::Abi(found) => {
                if found < abi {
                    return Err(Error::Reach(Unenforced::Abi { found, needs: abi }));
                }
            }
        }
        Ok(())
    }

    fn rule(&self, root: NodeId, path: &[u8], rights: Rights) -> Result<Grant, Refusal> {
        let node = if path.is_empty() {
            root
        } else {
            let found = self.resolve(root, path, false, false)?;
            let node = found.node.ok_or(Refusal::NotFound)?;
            if found.slash && !self.is_directory(node) {
                return Err(Refusal::NotADirectory);
            }
            node
        };
        let rights = if self.is_directory(node) { rights } else { Rights(rights.0 & Rights::FILE.0) };
        Ok(Grant { node, rights })
    }

    pub(crate) fn probe(
        &self,
        root: Opened,
        dir: &[u8],
        pipes: &[Pipe],
        layer: Option<Vec<Grant>>,
        questions: &[Question],
    ) -> Result<Report, Error> {
        let root = self.handle(root).node;
        let cwd = if dir.is_empty() {
            root
        } else {
            self.resolve(root, dir, false, false).map_err(kernel_error)?.node.ok_or(Error::NotFound)?
        };
        if !self.is_directory(cwd) {
            return Err(Error::NotADirectory);
        }
        let layers: Vec<_> = layer.into_iter().collect();
        Ok(self.probe_report(cwd, pipes, &layers, questions))
    }

    fn probe_report(&self, cwd: NodeId, pipes: &[Pipe], layers: &[Vec<Grant>], questions: &[Question]) -> Report {
        let mut answers = Vec::new();
        for question in questions {
            let answer = match question {
                Question::Read(path) => access(self.probe_access(cwd, path, layers, Rights::READ_FILE, READ)),
                Question::Write(path) => access(self.probe_access(cwd, path, layers, Rights::WRITE_FILE, WRITE)),
                Question::List(path) => access(self.probe_access(cwd, path, layers, Rights::READ_DIR, READ)),
                Question::Execute(path) => access(self.probe_access(cwd, path, layers, Rights::EXECUTE, SEARCH)),
                Question::SetId(path) => match self.probe_access(cwd, path, layers, Rights::EXECUTE, SEARCH) {
                    Ok(()) => Answer::Ids(Ids::Kept),
                    Err(error) => Answer::Refused(error),
                },
                Question::NoNewPrivs => Answer::NoNewPrivs(!layers.is_empty()),
                Question::SignalParent => {
                    if layers.is_empty() {
                        Answer::Granted
                    } else {
                        Answer::Refused(Error::Permission)
                    }
                }
                Question::Descriptors => {
                    let mut descriptors: Vec<_> = pipes.iter().map(|pipe| pipe.child).collect();
                    descriptors.sort_unstable();
                    Answer::Descriptors(descriptors.into_boxed_slice())
                }
                Question::Nest { rules, questions } => self.nested(cwd, pipes, layers, rules, questions),
            };
            answers.push(answer);
        }
        Report { answers: answers.into_boxed_slice() }
    }

    fn nested(
        &self,
        cwd: NodeId,
        pipes: &[Pipe],
        layers: &[Vec<Grant>],
        rules: &[probe::Rule],
        questions: &[Question],
    ) -> Answer {
        if let Err(error) = self.reach_abi(skein_io::kernel::Reach::ABI) {
            return Answer::Refused(error);
        }
        let mut layer = Vec::new();
        for (index, rule) in rules.iter().enumerate() {
            match self.rule(cwd, &rule.path, rule.rights) {
                Ok(grant) => layer.push(grant),
                Err(refusal) => return Answer::Refused(rule_error(index, refusal)),
            }
        }
        let mut nested = layers.to_vec();
        nested.push(layer);
        Answer::Nested(self.probe_report(cwd, pipes, &nested, questions))
    }

    fn probe_access(
        &self,
        cwd: NodeId,
        path: &[u8],
        layers: &[Vec<Grant>],
        right: Rights,
        bits: u32,
    ) -> Result<(), Error> {
        // A working directory is not a containment root: ../canary is a
        // normal lookup, then Landlock decides whether the target is granted.
        let mut root = cwd;
        while let Some(parent) = self.node(root).parent {
            if parent == self.top {
                break;
            }
            root = parent;
        }
        let found = self.resolve_at(root, cwd, path, false, false, false).map_err(kernel_error)?;
        let node = found.node.ok_or(Error::NotFound)?;
        if found.slash && !self.is_directory(node) {
            return Err(Error::NotADirectory);
        }
        if right == Rights::READ_DIR && !self.is_directory(node) {
            return Err(Error::NotADirectory);
        }
        if (right == Rights::READ_FILE || right == Rights::WRITE_FILE) && self.is_directory(node) {
            return Err(Error::IsADirectory);
        }
        if right == Rights::EXECUTE && self.is_directory(node) {
            return Err(Error::Permission);
        }
        match self.node(node).body {
            Body::Special(_) => return Err(Error::NotAFile),
            Body::Link(_) => unreachable!("probe lookup follows links"),
            Body::File(_) | Body::Directory { .. } => {}
        }
        if !self.may(node, bits) {
            return Err(Error::Permission);
        }
        for layer in layers {
            let mut granted = Rights(0);
            for rule in layer {
                if rule.node == node || (self.is_directory(rule.node) && self.within(found.parent, rule.node)) {
                    granted = granted.union(rule.rights);
                }
            }
            if !granted.contains(right) {
                return Err(Error::Permission);
            }
        }
        Ok(())
    }
}

fn access(result: Result<(), Error>) -> Answer {
    match result {
        Ok(()) => Answer::Granted,
        Err(error) => Answer::Refused(error),
    }
}

fn rule_error(index: usize, refusal: Refusal) -> Error {
    let errno: i32 = match refusal {
        Refusal::Held => 16,
        Refusal::NotFound => 2,
        Refusal::Exists => 17,
        Refusal::NotADirectory => 20,
        Refusal::IsADirectory => 21,
        Refusal::NotEmpty => 39,
        Refusal::Permission => 13,
        Refusal::NoSpace => 28,
        Refusal::Loop => 40,
        Refusal::NameTooLong => 36,
        Refusal::Escape => 18,
        Refusal::NotAFile => 6,
        Refusal::Beneath => 22,
    };
    Error::Reach(Unenforced::Rule { index: u32::try_from(index).expect("bounded reach rules"), errno })
}
