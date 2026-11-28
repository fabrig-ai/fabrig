//! Two copies of the journeys and their stylesheets (temper/docs/design/client/journey-writer-requests.md, 1.2, step 3).

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
};

use crate::{
    host::{Answer, Ask, Host},
    look_compare::{Journey, Side, done, git_ok, read, request::Request, text, version_two},
};

use super::html;

const SIDES: [Side; 2] = [Side::Web, Side::Tui];
const TREES: [&str; 3] = [
    temper_client_walk_lines::layout::WEB,
    temper_client_walk_lines::layout::TUI,
    "temper/crates/temper-client-shell-web",
];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Page {
    pub(super) side: Side,
    pub(super) relative: String,
}

impl Page {
    pub(super) fn from_path(path: &str) -> Option<Self> {
        let (side, relative) = in_root(path)?;
        (Path::new(relative).extension() == Some("html".as_ref())).then(|| Self { side, relative: relative.into() })
    }

    pub(super) fn journey(&self) -> Option<Journey> {
        let (name, _) = self.relative.split_once('/')?;
        Some(Journey { side: self.side, name: name.into() })
    }

    pub(super) fn numbered(&self) -> bool {
        self.relative.split_once('/').is_some_and(|(_, name)| self.side.numbered_page(name))
    }

    pub(super) fn path(&self) -> String {
        format!("{}/{}", self.side.root(), self.relative)
    }

    pub(super) fn item(&self) -> String {
        format!("{}/journeys/{}", self.side.name(), self.relative)
    }
}

pub(super) struct Copy {
    pub(super) directory: PathBuf,
    pub(super) pages: BTreeMap<Page, String>,
    pub(super) walks: BTreeSet<Journey>,
    directories: BTreeSet<PathBuf>,
}

impl Copy {
    fn new(host: &mut Host, scratch: &Path, name: &str) -> Result<Self, String> {
        let directory = scratch.join(name);
        done(host, Ask::Directory(directory.clone()))?;
        Ok(Self { directory, pages: BTreeMap::new(), walks: BTreeSet::new(), directories: BTreeSet::new() })
    }

    fn write(&mut self, host: &mut Host, path: &str, bytes: Box<[u8]>) -> Result<(), String> {
        let to = self.directory.join(path);
        let parent = to.parent().expect("copy paths have a parent");
        if self.directories.insert(parent.to_owned()) {
            done(host, Ask::Directories(parent.to_owned()))?;
        }
        let bytes = if let Some(page) = Page::from_path(path) {
            let original = String::from_utf8(bytes.into_vec()).map_err(|_| format!("{path} is not UTF-8"))?;
            let open = html::opened(&original);
            self.pages.insert(page, original);
            open.into_bytes().into_boxed_slice()
        } else {
            if let Some((side, relative)) = in_root(path)
                && let Some((name, "walk")) = relative.split_once('/')
                && version_two(&bytes)
            {
                self.walks.insert(Journey { side, name: name.into() });
            }
            bytes
        };
        done(host, Ask::Write(to, bytes))
    }

    pub(super) fn requested(host: &mut Host, root: &Path, scratch: &Path, request: &Request) -> Result<Self, String> {
        let mut copy = Self::new(host, scratch, "request")?;
        let args = ["ls-tree", "--full-tree", "-rz", &request.commit, "--", TREES[0], TREES[1], TREES[2]];
        let listing = git_ok(host, Some(root), &args)?;
        for entry in text(&listing, "request tree")?.split_terminator('\0') {
            let Some((info, path)) = entry.split_once('\t') else { return Err("invalid request tree entry".into()) };
            let mut fields = info.split_ascii_whitespace();
            let (Some(mode), Some(kind), Some(object), None) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                return Err(format!("invalid request tree entry for {path}"));
            };
            if !matches!(mode, "100644" | "100755") || kind != "blob" {
                continue; // No symlink or submodule can bring in a different tree.
            }
            check_path(path)?;
            if !matches!(object.len(), 40 | 64)
                || !object.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
            {
                return Err(format!("invalid request object for {path}"));
            }
            let bytes = git_ok(host, Some(root), &["cat-file", "blob", object])?;
            copy.write(host, path, bytes)?;
        }
        Ok(copy)
    }

    pub(super) fn printed(host: &mut Host, root: &Path, scratch: &Path) -> Result<Self, String> {
        let mut copy = Self::new(host, scratch, "print")?;
        for tree in TREES {
            let answer = host.ask(Ask::Files { root: root.to_owned(), tree });
            let files = match answer {
                Answer::Files(files) => files,
                Answer::Failed(why) => return Err(why),
                answer => unreachable!("unexpected file listing: {answer:?}"),
            };
            for path in files {
                let path = path.to_str().ok_or("print's file name is not UTF-8")?;
                check_path(path)?;
                if in_root(path).is_some_and(|(_, path)| path == "overlay.css") {
                    continue;
                }
                let disk = root.join(path);
                let bytes = read(host, &disk)?.ok_or_else(|| format!("{} disappeared", disk.display()))?;
                copy.write(host, path, bytes)?;
            }
        }
        for side in SIDES {
            copy.write(host, &format!("{}/overlay.css", side.root()), Box::default())?;
        }
        Ok(copy)
    }

    pub(super) fn path(&self, page: &Page) -> PathBuf {
        self.directory.join(page.path())
    }
}

pub(super) fn file_url(path: &Path) -> Box<[u8]> {
    let mut url = String::from("file://");
    for &byte in path.as_os_str().as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'.' | b'_' | b'~') {
            url.push(char::from(byte));
        } else {
            write!(url, "%{byte:02X}").expect("write string");
        }
    }
    url.into_bytes().into_boxed_slice()
}

fn in_root(path: &str) -> Option<(Side, &str)> {
    SIDES.into_iter().find_map(|side| Some((side, path.strip_prefix(side.root())?.strip_prefix('/')?)))
}

fn check_path(path: &str) -> Result<(), String> {
    if path.split('/').any(|component| matches!(component, "" | "." | ".."))
        || !TREES.iter().any(|tree| path.strip_prefix(tree).is_some_and(|tail| tail.starts_with('/')))
    {
        return Err(format!("path outside the render copy: {path:?}"));
    }
    Ok(())
}
