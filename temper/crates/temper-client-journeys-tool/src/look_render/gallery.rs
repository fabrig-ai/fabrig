//! The gallery print used by look-render (temper/docs/design/client/journey-writer.md, 6.2, steps 2–3).

use std::{
    collections::{BTreeMap, BTreeSet},
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
};

use crate::{
    host::{Answer, Ask, Host, Program, Stream},
    look_compare::{
        done, read,
        request::{Request, blob},
    },
};

use super::html;

pub(super) struct Drawing {
    pub(super) state: String,
    readme: String,
}

impl Drawing {
    pub(super) fn read(host: &mut Host, request: &Request, directory: &str) -> Result<Option<Self>, String> {
        let readme = blob(host, &request.commit, &format!("{directory}/README.md"))?;
        let Some(readme) = readme.and_then(|bytes| String::from_utf8(bytes.into_vec()).ok()) else {
            return Ok(None);
        };
        Ok(state(&readme).map(str::to_owned).map(|state| Self { state, readme }))
    }

    pub(super) fn requests(&self, file: &str) -> bool {
        let phrase = format!("`{file}` the request");
        self.readme.match_indices(&phrase).any(|(start, _)| {
            self.readme[start + phrase.len()..]
                .chars()
                .next()
                .is_none_or(|next| !next.is_ascii_alphanumeric() && !matches!(next, '-' | '_' | '\'' | '’'))
        })
    }
}

fn state(readme: &str) -> Option<&str> {
    let mut state = None;
    for phrase in readme.split("the gallery's state `").skip(1) {
        let Some((name, _)) = phrase.split_once('`') else { continue };
        if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-') {
            continue;
        }
        if state.is_some_and(|previous| previous != name) {
            return None;
        }
        state = Some(name);
    }
    state
}

pub(super) struct Gallery {
    directory: PathBuf,
    pages: BTreeMap<String, String>,
}

impl Gallery {
    pub(super) fn prepare(
        host: &mut Host,
        root: &Path,
        scratch: &Path,
        states: BTreeSet<&str>,
    ) -> Result<Self, String> {
        let mut gallery = Self { directory: scratch.join("gallery"), pages: BTreeMap::new() };
        if !states.is_empty() {
            gallery.print(host, root)?;
            for state in states {
                let path = gallery.path(state);
                if let Some(bytes) = read(host, &path)? {
                    let page =
                        String::from_utf8(bytes.into_vec()).map_err(|_| format!("{} is not UTF-8", path.display()))?;
                    let opened = html::opened(&page);
                    if opened != page {
                        done(host, Ask::Write(path, opened.into_bytes().into_boxed_slice()))?;
                    }
                    gallery.pages.insert(state.to_owned(), page);
                }
            }
        }
        Ok(gallery)
    }

    fn print(&self, host: &mut Host, root: &Path) -> Result<(), String> {
        let mut program = Program::new(
            b"cargo",
            &[
                b"run",
                b"--locked",
                b"--quiet",
                b"-p",
                b"temper-client-gallery",
                b"--",
                self.directory.as_os_str().as_bytes(),
            ],
        );
        program.directory = Some(root.to_owned());
        program.environment.push((b"TEMPER_INTERNAL_BUNDLE".as_slice().into(), b"none".as_slice().into()));
        program.stderr = Stream::Keep;
        match host.ask(Ask::Run(program)) {
            Answer::Ran(ran) if ran.code == Some(0) => Ok(()),
            Answer::Ran(ran) => {
                let status = ran.code.map_or_else(|| "no exit status".into(), |code| format!("exit {code}"));
                let stderr = String::from_utf8_lossy(&ran.stderr);
                let detail = stderr.lines().next().unwrap_or_default();
                let status = if detail.is_empty() { status } else { format!("{status}: {detail}") };
                Err(format!("gallery print: {status}"))
            }
            Answer::Failed(why) => Err(format!("gallery print: {why}")),
            answer => unreachable!("unexpected gallery print answer: {answer:?}"),
        }
    }

    fn path(&self, state: &str) -> PathBuf {
        self.directory.join(format!("{state}.html"))
    }

    pub(super) fn page(&self, state: &str) -> Option<(PathBuf, &str)> {
        self.pages.get(state).map(|source| (self.path(state), source.as_str()))
    }
}
