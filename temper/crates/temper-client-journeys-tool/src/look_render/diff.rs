//! Source-line alignment using git's zero-context hunks (temper/docs/design/client/journey-writer.md, 6.2).

use std::path::Path;

use crate::{
    host::{Ask, Host},
    look_compare::{
        done,
        pages::{self, Hunk},
    },
};

use super::html::Tag;

pub(super) fn between(host: &mut Host, directory: &Path, from: &str, to: &str) -> Result<Vec<Hunk>, String> {
    if from == to {
        return Ok(Vec::new());
    }
    // The shared comparison diff keeps its baseline in `landed`.
    for (name, value) in [("landed", from), ("requested", to)] {
        done(host, Ask::Write(directory.join(name), value.as_bytes().into()))?;
    }
    pages::diff(host, directory, "requested", from, to)
}

pub(super) fn edited(tags: &[Tag<'_>], edits: &[Hunk], touched: &mut [bool]) {
    for hunk in edits {
        let range = tag_range(tags, hunk.to.after, hunk.to.end());
        touched[range].fill(true);
    }
}

pub(super) fn pairs(left: &[Tag<'_>], right: &[Tag<'_>], edits: &[Hunk]) -> Result<Vec<Option<usize>>, String> {
    let mut pairs = vec![None; left.len()];
    let (mut from, mut to) = (0, 0);
    for hunk in edits {
        let kept = hunk.from.after.checked_sub(from);
        if kept.is_none() || kept != hunk.to.after.checked_sub(to) {
            return Err("inconsistent source-line alignment from git diff".into());
        }
        let mut kept_pairs = Alignment::new(&mut pairs, left, right, (from, hunk.from.after), (to, hunk.to.after));
        kept_pairs.unchanged();
        let mut changed_pairs =
            Alignment::new(&mut pairs, left, right, (hunk.from.after, hunk.from.end()), (hunk.to.after, hunk.to.end()));
        changed_pairs.hunk();
        (from, to) = (hunk.from.end(), hunk.to.end());
    }
    Alignment::new(&mut pairs, left, right, (from, usize::MAX), (to, usize::MAX)).unchanged();
    Ok(pairs)
}

fn tag_range(tags: &[Tag<'_>], after: usize, end: usize) -> std::ops::Range<usize> {
    tags.partition_point(|tag| tag.line <= after)..tags.partition_point(|tag| tag.line <= end)
}

struct Alignment<'a, 'source> {
    pairs: &'a mut [Option<usize>],
    left: &'a [Tag<'source>],
    right: &'a [Tag<'source>],
    from: std::ops::Range<usize>,
    to: std::ops::Range<usize>,
    line_from: usize,
    line_to: usize,
}

impl<'a, 'source> Alignment<'a, 'source> {
    fn new(
        pairs: &'a mut [Option<usize>],
        left: &'a [Tag<'source>],
        right: &'a [Tag<'source>],
        (line_from, end_from): (usize, usize),
        (line_to, end_to): (usize, usize),
    ) -> Self {
        Self {
            pairs,
            left,
            right,
            from: tag_range(left, line_from, end_from),
            to: tag_range(right, line_to, end_to),
            line_from,
            line_to,
        }
    }

    fn unchanged(&mut self) {
        let mut to = self.to.start;
        for from in self.from.clone() {
            let tag = &self.left[from];
            let position = (tag.line - self.line_from, tag.column);
            let right_position = |at: usize| {
                let tag = &self.right[at];
                (tag.line - self.line_to, tag.column)
            };
            while to < self.to.end && right_position(to) < position {
                to += 1;
            }
            if to < self.to.end && right_position(to) == position {
                if self.left[from].name.eq_ignore_ascii_case(self.right[to].name) {
                    self.pairs[from] = Some(to);
                }
                to += 1;
            }
        }
    }

    fn hunk(&mut self) {
        for (from, to) in self.from.clone().zip(self.to.clone()) {
            if !self.left[from].name.eq_ignore_ascii_case(self.right[to].name) {
                break;
            }
            self.pairs[from] = Some(to);
        }
    }
}
