//! The data files temper init writes (protocol/setup/init.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use skein_lib::{Writer, bytes};

mod hex;
mod path;
mod person;
#[cfg(test)]
mod tests;

/// Claude Code is fetched when its program is first needed (protocol/setup/owned.md, 6).
pub use temper_fetch::claude_code::FETCHED;

/// What the binary finds on the machine, each path absolute (protocol/setup/init.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Facts<'a> {
    /// The running temper's own path: the playground's git link target.
    pub binary: &'a [u8],
    /// The git found on PATH, or none: the worker's git.
    pub git: Option<&'a [u8]>,
    /// The claude found on PATH, or none when its pinned build is needed.
    pub claude: Option<&'a [u8]>,
    /// The effective uid's name in /etc/passwd, or none.
    pub login: Option<&'a [u8]>,
}

/// What the bare deployment's data files need (protocol/setup/init.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bare<'a> {
    pub binary: &'a [u8],
    pub git: Option<&'a [u8]>,
    pub claude: Option<&'a [u8]>,
}

/// The directories init writes into, each absolute (protocol/configuration/directories.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Directories<'a> {
    pub config: &'a [u8],
    pub data: &'a [u8],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Directory {
    Config,
    Data,
}

/// One file to create new, in its directory.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct File {
    pub directory: Directory,
    /// Relative to its directory: `worker.secret`, `demo/bin/git`.
    pub name: &'static [u8],
    pub content: Content,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Content {
    /// 0o600 when private, else 0o644.
    Bytes {
        bytes: Box<[u8]>,
        private: bool,
    },
    Link {
        target: Box<[u8]>,
    },
}

/// The files to write and whether the machine's login can name the owned Forgejo's person.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Answer {
    pub files: Box<[File]>,
    pub person: Person,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Person {
    Found,
    FellBack,
}

/// The fact or directory refused, and why; nothing is answered.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Refusal {
    pub fact: Fact,
    pub reason: Reason,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fact {
    Binary,
    Git,
    Claude,
    Config,
    Data,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reason {
    /// It does not begin with `/`.
    Relative,
    /// It is not UTF-8, which init's paths cannot hold.
    Text,
    /// It, or the joined `<data>/worker.secret`, passes 4096 bytes.
    Bound,
}

/// The data files of protocol/setup/init.md 2, in its table's order:
/// `worker.secret`, then `demo/bin/git` (a link). The shell renders the configuration last.
pub fn files(facts: &Facts<'_>, directories: &Directories<'_>, secret: &[u8; 32]) -> Result<Answer, Refusal> {
    check(facts.binary, facts.git, facts.claude, directories)?;

    let digits = hex::digits(secret);
    let (_, person) = person::login(facts.login);
    Ok(Answer {
        person,
        files: Box::new([
            File {
                directory: Directory::Data,
                name: b"worker.secret",
                content: Content::Bytes { bytes: line(&digits), private: true },
            },
            File {
                directory: Directory::Data,
                name: b"demo/bin/git",
                content: Content::Link { target: bytes::copy_of(facts.binary) },
            },
        ]),
    })
}

/// The bare deployment's data file, `worker.secret`.
pub fn bare(facts: &Bare<'_>, directories: &Directories<'_>, secret: &[u8; 32]) -> Result<Box<[File]>, Refusal> {
    check(facts.binary, facts.git, facts.claude, directories)?;
    let digits = hex::digits(secret);
    Ok(Box::new([File {
        directory: Directory::Data,
        name: b"worker.secret",
        content: Content::Bytes { bytes: line(&digits), private: true },
    }]))
}

fn check(
    binary: &[u8],
    git: Option<&[u8]>,
    claude: Option<&[u8]>,
    directories: &Directories<'_>,
) -> Result<(), Refusal> {
    path::check(Fact::Binary, binary, b"")?;
    if let Some(git) = git {
        path::check(Fact::Git, git, b"")?;
    }
    if let Some(claude) = claude {
        path::check(Fact::Claude, claude, b"")?;
    }
    path::check(Fact::Config, directories.config, b"")?;
    path::check(Fact::Data, directories.data, b"worker.secret")?;

    Ok(())
}

fn line(digits: &[u8]) -> Box<[u8]> {
    let mut writer = Writer::new(digits.len().checked_add(1).expect("64 digits and a newline fit"));
    writer.put(digits).expect("the digits were counted");
    writer.put(b"\n").expect("the newline was counted");
    writer.finish()
}
