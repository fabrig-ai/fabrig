//! Commit range fields without changed files (forge-github.md, 4.5).
use super::Malformed;
use crate::{
    api,
    github::{document, id},
};
use alloc::boxed::Box;
use skein_json::Token;
use skein_lib::{List, bytes};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Node {
    Comparison,
    Rows,
    Commit,
    Message,
    Parents,
    Parent,
}

#[derive(Debug)]
pub(super) struct State {
    total: u64,
    count: u32,
    inner: u32,
    id: api::Commit,
    parents: List<api::Commit>,
    message: Box<[u8]>,
}
impl State {
    pub(super) fn new(inner: u32) -> Self {
        Self { total: 0, count: 0, inner, id: [0; 32], parents: List::with_capacity(inner), message: Box::from([]) }
    }
    pub(super) fn document(&self) -> document::Compare {
        document::Compare { total_commits: self.total, ahead_by: 0, behind_by: 0, merge_base: None }
    }
    pub(super) fn end(&mut self, node: Node) -> Result<Option<document::Row>, Malformed> {
        if node != Node::Commit {
            return Ok(None);
        }
        self.count = self.count.checked_add(1).ok_or(Malformed::Bound { field: b"commits" })?;
        if self.count > self.inner {
            return Err(Malformed::Bound { field: b"commits" });
        }
        Ok(Some(document::Row::CommitMessage(api::CommitMessage {
            id: self.id,
            parents: core::mem::replace(&mut self.parents, List::with_capacity(self.inner)).into_boxed(),
            message: core::mem::replace(&mut self.message, Box::from([])),
        })))
    }
    pub(super) fn scalar(&mut self, node: Node, key: &'static [u8], token: Token) -> Result<(), Malformed> {
        match token {
            Token::Number(text) => {
                self.total = bytes::decimal(&text).ok_or(Malformed::Range { field: key })?;
            }
            Token::String(text) => {
                if node == Node::Message {
                    self.message = text;
                } else {
                    let commit = id::commit(&text).ok_or(Malformed::Id { field: key })?;
                    if node == Node::Parent {
                        self.parents.push(commit).or(Err(Malformed::Bound { field: b"parents" }))?;
                    } else {
                        self.id = commit;
                    }
                }
            }
            Token::ObjectStart
            | Token::ObjectEnd
            | Token::ArrayStart
            | Token::ArrayEnd
            | Token::Key(_)
            | Token::True
            | Token::False
            | Token::Null => return Err(Malformed::Shape { field: key }),
        }
        Ok(())
    }
}
