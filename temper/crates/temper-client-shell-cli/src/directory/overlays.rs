//! Bounded set reads before DIRECTORY opens (client/ctl/overlays.md, 2).
use super::{Directory, OWNER, Stage, file};
use skein_lib::{List, Token};
use settings_form::overlay::{self, DirectoryName, Markdown};
#[derive(Clone, Copy)]
enum Step { Open, List, Load, Close }
pub(super) struct Sets {
    names: List<DirectoryName>, index: u32, root: Option<Token>, parent: Token, step: Step,
    pending: Vec<Box<[u8]>>, file: Option<Box<[u8]>>, files: List<Markdown>, remaining: u32,
    pub(super) output: List<overlay::Directory>,
}
impl Directory {
    pub(crate) fn load_sets(&mut self, names: List<DirectoryName>, remaining: u32, stdin: bool) {
        let parent = if stdin { self.root } else { self.text_root }.expect("settings input parent");
        self.sets = Some(Sets { names, index: 0, root: None, parent, step: Step::Open, pending: Vec::new(), file: None,
            files: List::uncapped(), remaining, output: List::uncapped() });
        self.stage = Stage::Overlay;
    }
    pub(super) fn sets_request(&mut self) -> Option<file::Request> {
        let sets = self.sets.as_mut().expect("overlay input");
        let Some(name) = sets.names.get(sets.index) else {
            self.loaded_sets = Some(self.sets.take().expect("sets").output);
            self.stage = Stage::Ready;
            return None;
        };
        Some(match sets.step {
            Step::Open => file::Request::OpenDirectory { owner: OWNER, root: self.io.descriptor(sets.parent).expect("input parent"), name: name.path.clone(), no_follow: false },
            Step::List => file::Request::ListPage { owner: OWNER, directory: sets.root.expect("set directory"), max: 64 },
            Step::Load => {
                let name = sets.pending.pop().expect("Markdown name"); sets.file = Some(name.clone());
                file::Request::Load { owner: OWNER, root: sets.root.expect("set directory"), path: name, max: sets.remaining, no_follow: false }
            }
            Step::Close => file::Request::Close { owner: OWNER, file: sets.root.expect("set directory") },
        })
    }
    pub(super) fn sets_answer(&mut self, event: file::Event) {
        let sets = self.sets.as_mut().expect("overlay input");
        let name = sets.names.get(sets.index).expect("current directory");
        match (sets.step, event) {
            (Step::Open, file::Event::Opened { file, .. }) => { sets.root = Some(file); sets.step = Step::List; }
            (Step::List, file::Event::Listed { entries, .. }) => {
                if entries.is_empty() { sets.step = Step::Close; }
                for entry in entries {
                    if entry.name.ends_with(b".md") { sets.pending.push(entry.name); }
                }
                if !sets.pending.is_empty() { sets.step = Step::Load; }
            }
            (Step::Load, file::Event::Loaded { bytes, .. }) => {
                sets.remaining = sets.remaining.checked_sub(u32::try_from(bytes.len()).expect("read bound")).expect("read bounded");
                sets.files.push(Markdown { name: sets.file.take().expect("file name"), text: bytes }).expect("input bounded");
                sets.step = if sets.pending.is_empty() { Step::List } else { Step::Load };
            }
            (Step::Close, file::Event::Closed { .. }) => {
                sets.root = None;
                sets.output.push(overlay::Directory { path: name.path.clone(), files: core::mem::replace(&mut sets.files, List::uncapped()) }).expect("input bounded");
                sets.index += 1; sets.step = Step::Open;
            }
            (_, event) => {
                let file = sets.file.as_deref().unwrap_or(b"directory");
                let why = format!("{}: {}", String::from_utf8_lossy(file), super::answer_words(&event));
                let error = overlay::refusal(name, &why); self.fail(&error);
            }
        }
    }
    pub(super) fn close_set(&mut self) -> Option<Token> { self.sets.as_mut().and_then(|sets| sets.root.take()) }
}
