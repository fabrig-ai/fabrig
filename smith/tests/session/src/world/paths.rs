//! Classify names against the workspace when an answer arrives, before its
//! calls run. Record those classes beside the eventual results, as for jobs
//! (smith/docs/design/drawn-agent.md, 5; smith/docs/design/contracts.md, 1).

use skein_lib::Token;
use smith_domain_session::llm::{Block, Decoded, Problem, ToolCall};
use smith_domain_tools::{Authority, Call, Part, Path};

use super::World;
use crate::provider::SLOT_NAMES;
use crate::shapes::{Answer, Class};

/// A call's workspace references paired with its answer in the recorded turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathCall {
    pub sequence: u32,
    pub position: usize,
    pub path: Option<Class>,
    pub offset: Option<Class>,
    pub answer: Answer,
}

impl World {
    pub(super) fn drawn_paths(&mut self, opener: u64, content: &[Block]) -> Vec<PathCall> {
        let mut paths = Vec::new();
        for (position, call) in content.iter().enumerate().filter_map(|(pos, block)| Some((pos, block.tool_call()?))) {
            let (path, offset) = self.path_classes(opener, call);
            if path.is_some() || offset.is_some() {
                paths.push(PathCall { sequence: 0, position, path, offset, answer: Answer::Unanswered });
            }
            if self.settings.provider.capture_answers != 0 {
                // Draws can occur in fields the chosen tool ignores. Such a
                // slot gets Unknown but is not a reference in the call shape.
                for (slot, class) in SLOT_NAMES[1..].iter().zip([offset, path, path, path]) {
                    self.provider.classify(&call.id, slot, class.unwrap_or(Class::Unknown).word().as_bytes());
                }
                if call.name.as_ref() != b"job" {
                    self.provider.classify(&call.id, b"job", Class::Unknown.word().as_bytes());
                }
            }
        }
        paths
    }

    fn path_classes(&self, opener: u64, call: &ToolCall) -> (Option<Class>, Option<Class>) {
        let (path, offset) = match &call.call {
            Decoded::Owned { call: owned } => match owned {
                Call::Read { path, offset, .. } => (path, (*offset != 1 || fields(&call.input).1).then_some(*offset)),
                Call::List { path, .. }
                | Call::Write { path, .. }
                | Call::Edit { path, .. }
                | Call::Remove { path } => (path, None),
                Call::Search { path, .. }
                    if path.absolute || !matches!(&*path.parts, [Part::Current]) || fields(&call.input).0 =>
                {
                    (path, None)
                }
                Call::Search { .. } | Call::Shell { .. } | Call::Job { .. } => return (None, None),
            },
            Decoded::Invalid { descriptor: None, problem }
                if matches!(
                    problem,
                    Problem::Missing { .. } | Problem::WrongType { .. } | Problem::BadValue { .. }
                ) && matches!(&*call.name, b"read" | b"list" | b"search" | b"write" | b"edit" | b"remove") =>
            {
                let (path, offset) = fields(&call.input);
                return (
                    path.then_some(Class::Unknown),
                    (call.name.as_ref() == b"read" && offset).then_some(Class::Unknown),
                );
            }
            _ => return (None, None),
        };
        let session = &self.sessions[&opener];
        let Some((root, path)) = place(&session.authority, path) else {
            return (Some(Class::Foreign), offset.map(|_| Class::Unknown));
        };
        let global = name(self.checkout.root_path(root.raw()), &path);
        let path_class = if self.checkout.exists(&global) {
            Class::Live
        } else if session.removed_paths.contains(&global) {
            Class::Stale
        } else {
            Class::Unissued
        };
        let offset_class = offset.map(|offset| match self.checkout.content(&global) {
            Some(content) => {
                let lines = skein_lib::bytes::count(content, b"\n", u32::MAX)
                    .saturating_add(u32::from(!content.is_empty() && !content.ends_with(b"\n")));
                if offset <= lines { Class::Live } else { Class::Unissued }
            }
            None => Class::Unknown,
        });
        (Some(path_class), offset_class)
    }
}

// Optional typed fields have already lost their presence bit. Read it from
// the original object, not from a search that could find a key inside a value.
fn fields(input: &[u8]) -> (bool, bool) {
    let client = skein_llm_kit::limits();
    let bytes = u32::try_from(input.len()).unwrap_or(u32::MAX).min(client.input);
    let bounds = skein_llm::DocumentLimits { bytes, strings: bytes, tokens: bytes, depth: client.depth };
    skein_llm::Json::from_bytes(input, &bounds).map_or((false, false), |json| {
        let doc = json.document();
        (doc.member(0, b"path").is_some(), doc.member(0, b"offset").is_some())
    })
}

// Like the tools, resolve names lexically in the namespace, with an absolute
// host alias taking precedence over a mount and the longest alias winning.
fn place(authority: &Authority, path: &Path) -> Option<(Token, Vec<u8>)> {
    let mut parts = Vec::new();
    if !path.absolute {
        parts.extend(authority.cwd.iter().map(smith_domain_tools::Name::as_bytes));
    }
    for part in &path.parts {
        match part {
            Part::Current => {}
            Part::Parent => {
                parts.pop();
            }
            Part::Name { name } => parts.push(name.as_bytes()),
        }
    }
    if path.absolute {
        let alias = authority
            .repos
            .iter()
            .filter(|repo| {
                repo.host.len() <= parts.len()
                    && parts.iter().zip(&repo.host).all(|(part, host)| *part == host.as_bytes())
            })
            .max_by_key(|repo| repo.host.len());
        if let Some(alias) = alias {
            return Some((alias.root, parts[alias.host.len()..].join(&b'/')));
        }
    }
    let (mount, tail) = parts.split_first()?;
    let repo = authority.repos.iter().find(|repo| repo.mount.as_bytes() == *mount)?;
    Some((repo.root, tail.join(&b'/')))
}

pub(super) fn name(root: &[u8], path: &[u8]) -> Vec<u8> {
    if path.is_empty() { root.to_vec() } else { [root, b"/", path].concat() }
}
