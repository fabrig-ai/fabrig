//! The files temper init writes, and the default document (protocol/setup/init.md, 2 and 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::num::NonZeroU16;
use skein_lib::{Writer, bytes, sha256};

mod hex;
mod path;
mod person;
mod template;
#[cfg(test)]
mod tests;

/// Claude Code is fetched when its program is first needed (protocol/setup/owned.md, 6).
pub use temper_fetch::claude_code::FETCHED;

/// What the binary finds on the machine, each path absolute (protocol/setup/init.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Facts<'a> {
    /// The running temper's own path: the worker's agent's program.
    pub binary: &'a [u8],
    /// The git found on PATH, or none: the worker's git.
    pub git: Option<&'a [u8]>,
    /// The claude found on PATH, or none when its pinned build is needed.
    pub claude: Option<&'a [u8]>,
    /// The listeners' ports on 127.0.0.1.
    pub ports: Ports,
    /// The effective uid's name in /etc/passwd, or none.
    pub login: Option<&'a [u8]>,
    /// Whether the host made or found the provider key (protocol/openid-provider.md, 9).
    pub sign_in: bool,
}

/// What a bare document needs, without the box's parts (protocol/setup/init.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bare<'a> {
    pub binary: &'a [u8],
    pub git: Option<&'a [u8]>,
    pub claude: Option<&'a [u8]>,
    pub clients: NonZeroU16,
    pub webhooks: NonZeroU16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ports {
    pub clients: NonZeroU16,
    pub owned: NonZeroU16,
    pub forge: NonZeroU16,
    pub llm: NonZeroU16,
    pub webhooks: NonZeroU16,
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
    /// Relative to its directory: `worker.secret`, `agent.json`.
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

/// The files to write and whether the machine's login names forge 3's person.
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
    /// It is not UTF-8, which JSON's text cannot hold.
    Text,
    /// It, or a path joined from it (`<config>/agent.json`,
    /// `<data>/programs`), passes 4096 bytes.
    Bound,
}

/// The files of protocol/setup/init.md 2, in its table's order: `worker.secret`,
/// `worker.digest`, `demo/llm.key` (data, private), `demo/bin/git`
/// (data, a link), `agent.json`, `temper.json` (config), the
/// document last.
pub fn files(
    facts: &Facts<'_>,
    directories: &Directories<'_>,
    secret: &[u8; 32],
    demo_key: &[u8; 32],
) -> Result<Answer, Refusal> {
    check(facts.binary, facts.git, facts.claude, directories)?;

    let digits = hex::digits(secret);
    let digest = hex::digits(&sha256::digest(&digits));
    let (login, person) = person::login(facts.login);
    let (document, agent) = template::documents(facts, directories, login);
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
                name: b"worker.digest",
                content: Content::Bytes { bytes: line(&digest), private: true },
            },
            File {
                directory: Directory::Data,
                name: b"demo/llm.key",
                content: Content::Bytes { bytes: line(&hex::digits(demo_key)), private: true },
            },
            File {
                directory: Directory::Data,
                name: b"demo/bin/git",
                content: Content::Link { target: bytes::copy_of(facts.binary) },
            },
            File {
                directory: Directory::Config,
                name: b"agent.json",
                content: Content::Bytes { bytes: agent, private: false },
            },
            File {
                directory: Directory::Config,
                name: b"temper.json",
                content: Content::Bytes { bytes: document, private: false },
            },
        ]),
    })
}

/// `worker.secret`, `worker.digest`, `agent.json`, `temper.json`, in that order.
pub fn bare(facts: &Bare<'_>, directories: &Directories<'_>, secret: &[u8; 32]) -> Result<Box<[File]>, Refusal> {
    check(facts.binary, facts.git, facts.claude, directories)?;
    let digits = hex::digits(secret);
    let digest = hex::digits(&sha256::digest(&digits));
    let (document, agent) = template::bare_documents(facts, directories);
    Ok(Box::new([
        File {
            directory: Directory::Data,
            name: b"worker.secret",
            content: Content::Bytes { bytes: line(&digits), private: true },
        },
        File {
            directory: Directory::Data,
            name: b"worker.digest",
            content: Content::Bytes { bytes: line(&digest), private: true },
        },
        File {
            directory: Directory::Config,
            name: b"agent.json",
            content: Content::Bytes { bytes: agent, private: false },
        },
        File {
            directory: Directory::Config,
            name: b"temper.json",
            content: Content::Bytes { bytes: document, private: false },
        },
    ]))
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
    path::check(Fact::Config, directories.config, b"agent.json")?;
    path::check(Fact::Data, directories.data, b"programs")?;

    Ok(())
}

fn line(digits: &[u8]) -> Box<[u8]> {
    let mut writer = Writer::new(digits.len().checked_add(1).expect("64 digits and a newline fit"));
    writer.put(digits).expect("the digits were counted");
    writer.put(b"\n").expect("the newline was counted");
    writer.finish()
}
