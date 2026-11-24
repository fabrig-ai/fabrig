//! A scripted git DAG and its worktrees (temper/docs/design/testing/qa/README.md, 6).
use super::{Tree, id, path_io};
use crate::{
    Refusal,
    faces::{Changed, Commit, DirectoryMode, Entry, FilesFace, GitFace, RemoveMode, trailers},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::ErrorKind,
    path::Path,
};

#[derive(Default)]
pub struct Range {
    pub patch: Vec<u8>,
    pub stat: Vec<Changed>,
    pub paths: Vec<String>,
}
#[derive(Default)]
pub struct Node {
    pub parents: Vec<String>,
    pub message: String,
    pub files: BTreeMap<String, Vec<u8>>,
    pub paths: Vec<String>,
    pub stat: Vec<Changed>,
    pub ranges: BTreeMap<String, Range>,
}
#[derive(Default)]
pub struct Git<F = Tree> {
    pub(super) files: F,
    pub head: String,
    pub nodes: BTreeMap<String, Node>,
    order: Vec<String>,
    pub calls: Vec<String>,
    pub fail_head: bool,
    pub fail_add: bool,
    pub move_error: Option<String>,
    pub prunes: usize,
}

impl<F> Git<F> {
    pub fn new(files: F) -> Self {
        Self {
            files,
            head: String::new(),
            nodes: BTreeMap::new(),
            order: Vec::new(),
            calls: Vec::new(),
            fail_head: false,
            fail_add: false,
            move_error: None,
            prunes: 0,
        }
    }

    pub fn commit(&mut self, number: u64, parents: &[u64], message: &str) {
        let name = id(number);
        self.order.push(name.clone());
        self.nodes.insert(
            name,
            Node { parents: parents.iter().map(|n| id(*n)).collect(), message: message.into(), ..Node::default() },
        );
        self.head = id(number);
    }
    pub fn merge(&mut self, number: u64, parents: &[u64], goal: u64) {
        self.commit(number, parents, &format!("Goal {goal}'s title\n\nGoal: {goal}\nPull-request: 12\n"));
        for path in [
            "temper/docs/design/client/pages/work.md",
            "temper/docs/design/client/pages/README.md",
            "temper/docs/design/client/pages/work.status.md",
            "temper/docs/design/client/pages/chat/README.md",
            "temper/docs/design/client/tui/pages/chat.md",
        ] {
            self.file_at(number, path, b"# Page\n");
        }
    }
    pub fn file_at(&mut self, commit: u64, path: &str, bytes: &[u8]) {
        self.nodes.get_mut(&id(commit)).expect("commit").files.insert(path.into(), bytes.into());
    }
    fn held(&self, commit: &str) -> BTreeSet<String> {
        let mut held = BTreeSet::new();
        let mut todo = vec![commit.to_owned()];
        while let Some(name) = todo.pop() {
            if held.insert(name.clone()) {
                todo.extend(self.nodes.get(&name).expect("known commit").parents.clone());
            }
        }
        held
    }
    fn node(&self, name: &str) -> &Node {
        self.nodes.get(name).expect("known commit")
    }
}
impl<F: FilesFace> GitFace for Git<F> {
    fn fetch(&mut self, remote: &str, branch: &str) -> Result<(), Refusal> {
        self.calls.push(format!("fetch {remote} {branch}"));
        Ok(())
    }
    fn head(&mut self, reference: &str) -> Result<String, Refusal> {
        self.calls.push(format!("head {reference}"));
        if self.fail_head {
            return Err(Refusal("head refused".into()));
        }
        Ok(self.head.clone())
    }
    fn commits(&mut self, old: &str, new: &str) -> Result<Vec<Commit>, Refusal> {
        self.calls.push(format!("move {old} {new}"));
        let old = self.held(old);
        let new = self.held(new);
        Ok(self
            .order
            .iter()
            .filter(|name| new.contains(*name) && !old.contains(*name))
            .map(|name| Commit { name: name.clone(), parents: self.node(name).parents.clone() })
            .collect())
    }
    fn message(&mut self, commit: &str) -> Result<String, Refusal> {
        self.calls.push(format!("message {commit}"));
        Ok(self.node(commit).message.clone())
    }
    fn parents(&mut self, commit: &str) -> Result<Vec<String>, Refusal> {
        self.calls.push(format!("parents {commit}"));
        Ok(self.node(commit).parents.clone())
    }
    fn contains(&mut self, ancestor: &str, descendant: &str) -> Result<bool, Refusal> {
        self.calls.push(format!("contains {ancestor} {descendant}"));
        Ok(self.held(descendant).contains(ancestor))
    }
    fn count(&mut self, commit: &str) -> Result<u64, Refusal> {
        Ok(u64::try_from(self.held(commit).len()).expect("count"))
    }
    fn entries(&mut self, commit: &str, directory: &str) -> Result<Vec<Entry>, Refusal> {
        self.calls.push(format!("entries {commit} {directory}"));
        let prefix = format!("{directory}/");
        let mut entries = BTreeMap::new();
        for path in self.node(commit).files.keys() {
            if let Some(rest) = path.strip_prefix(&prefix) {
                let (name, directory) = rest.split_once('/').map_or((rest, false), |(first, _)| (first, true));
                entries.insert(name.to_owned(), directory);
            }
        }
        Ok(entries.into_iter().map(|(name, directory)| Entry { name, directory }).collect())
    }
    fn file(&mut self, commit: &str, path: &str) -> Result<Option<Vec<u8>>, Refusal> {
        self.calls.push(format!("file {commit} {path}"));
        Ok(self.node(commit).files.get(path).cloned())
    }
    fn diff(&mut self, parent: &str, commit: &str) -> Result<Vec<u8>, Refusal> {
        self.calls.push(format!("diff {parent} {commit}"));
        assert!(self.node(commit).parents.iter().any(|name| name == parent));
        Ok(self
            .node(commit)
            .ranges
            .get(parent)
            .map_or_else(|| b"diff --git a/page b/page\n+new page\n".into(), |range| range.patch.clone()))
    }
    fn stat(&mut self, parent: &str, commit: &str) -> Result<Vec<Changed>, Refusal> {
        self.calls.push(format!("stat {parent} {commit}"));
        assert!(self.node(commit).parents.iter().any(|name| name == parent));
        Ok(self
            .node(commit)
            .ranges
            .get(parent)
            .map_or_else(|| self.node(commit).stat.clone(), |range| range.stat.clone()))
    }
    fn changed_paths(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal> {
        self.calls.push(format!("paths {parent} {commit}"));
        assert!(self.node(commit).parents.iter().any(|name| name == parent));
        Ok(self
            .node(commit)
            .ranges
            .get(parent)
            .map_or_else(|| self.node(commit).paths.clone(), |range| range.paths.clone()))
    }
    fn qa_trailers(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal> {
        self.calls.push(format!("qa {parent} {commit}"));
        let mut values = Vec::new();
        for record in self.commits(parent, commit)? {
            for (key, value) in trailers(&self.node(&record.name).message) {
                if key == "QA" {
                    values.push(value);
                }
            }
        }
        Ok(values)
    }
    fn gap_trailers(&mut self, parent: &str, commit: &str) -> Result<Vec<String>, Refusal> {
        self.calls.push(format!("gaps {parent} {commit}"));
        let mut values = Vec::new();
        for record in self.commits(parent, commit)? {
            for (key, value) in trailers(&self.node(&record.name).message) {
                if key == "QA-gap" && !values.contains(&value) {
                    values.push(value);
                }
            }
        }
        Ok(values)
    }
    fn add_tree(&mut self, path: &Path, commit: &str) -> Result<(), Refusal> {
        self.calls.push(format!("add {commit}"));
        path_io(path, self.files.directory(path, DirectoryMode::Parents))?;
        if self.fail_add {
            return Err(Refusal("worktree refused".into()));
        }
        Ok(())
    }
    fn move_tree(&mut self, path: &Path, commit: &str) -> Result<(), Refusal> {
        self.calls.push(format!("checkout {commit}"));
        if let Some(why) = &self.move_error {
            return Err(Refusal(why.clone()));
        }
        self.calls.push("clean -ffdxq".into());
        path_io(path, self.files.remove(path, RemoveMode::Tree))?;
        path_io(path, self.files.directory(path, DirectoryMode::Parents))
    }
    fn remove_tree(&mut self, path: &Path) -> Result<(), Refusal> {
        self.calls.push(format!("remove {}", path.display()));
        match self.files.remove(path, RemoveMode::Tree) {
            Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
            result => path_io(path, result),
        }
    }
    fn prune(&mut self) -> Result<(), Refusal> {
        self.prunes += 1;
        Ok(())
    }
}

pub fn baseline() -> Git {
    baseline_with(Tree::default())
}

pub fn baseline_with<F>(files: F) -> Git<F> {
    let mut git = Git::new(files);
    git.commit(0, &[], "baseline");
    git
}
