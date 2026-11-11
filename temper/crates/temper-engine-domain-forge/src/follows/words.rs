//! Code-made production briefs (temper/docs/design/domain/forge/follows.md, 4).
use crate::{FollowEntry, commit_text};
use alloc::boxed::Box;
use skein_lib::{Decimal, List};
#[derive(Debug)]
pub struct Words {
    pub follow: u32,
    pub source: Box<[u8]>,
    pub branch: Box<[u8]>,
    pub target: Box<[u8]>,
    pub target_branch: Box<[u8]>,
    pub pages: Box<[u8]>,
    pub paths: Box<[Box<[u8]>]>,
}
/// `bound` is the specification bound after subtracting byte-valued parameters.
#[must_use]
pub fn build(entries: &[FollowEntry], names: &Words, bound: u32) -> Option<Box<[u8]>> {
    let mut rest = usize::MAX;
    let mut files = usize::MAX;
    let attempts = rest_size(entries).saturating_add(files_size(entries)).saturating_add(2);
    for _ in 0..=attempts {
        let words = render(entries, names, rest, files)?;
        let excess = words.len().saturating_sub(skein_lib::usize_of(bound));
        if excess == 0 {
            return Some(words);
        }
        if rest == usize::MAX {
            rest = rest_size(entries);
        }
        if rest > 0 {
            rest = rest.saturating_sub(excess);
            continue;
        }
        if files == usize::MAX {
            files = files_size(entries);
        }
        if files > 0 {
            files = files.saturating_sub(excess);
            continue;
        }
        return None;
    }
    None
}
fn add(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("measured follow words");
    }
}
fn number(out: &mut List<u8>, n: u64) {
    add(out, Decimal::of(n).as_bytes());
}
fn split(message: &[u8]) -> (&[u8], &[u8]) {
    match skein_lib::bytes::find(message, b"\n") {
        Some(at) => {
            (message.get(..at).expect("first line"), message.get(at.saturating_add(1)..).expect("message rest"))
        }
        None => (message, b""),
    }
}
fn rest_size(entries: &[FollowEntry]) -> usize {
    let mut size = 0_usize;
    for entry in entries {
        for commit in &entry.commits {
            size = size.saturating_add(split(&commit.message).1.len());
        }
    }
    size
}
fn files_size(entries: &[FollowEntry]) -> usize {
    let mut size = 0_usize;
    for entry in entries {
        if let Some(files) = &entry.files {
            for path in files {
                size = size.saturating_add(path.len().saturating_add(3));
            }
        }
    }
    size
}
#[expect(
    clippy::too_many_lines,
    reason = "the binding brief's ordered blocks and their two cut budgets are rendered together"
)]
fn render(entries: &[FollowEntry], n: &Words, mut rest: usize, mut files_room: usize) -> Option<Box<[u8]>> {
    let newest = entries.last()?;
    let after = commit_text(&newest.after);
    let subject = match newest.commits.last() {
        Some(commit) => split(&commit.message).0,
        None => b"Source landing",
    };
    let mut title = List::uncapped();
    add(&mut title, b"Follow ");
    add(&mut title, &n.source);
    add(&mut title, b" ");
    add(&mut title, after.get(..12)?);
    if entries.len() > 1 {
        add(&mut title, b" and ");
        number(&mut title, skein_lib::len_u64(entries.len().saturating_sub(1)));
        add(&mut title, b" more");
    }
    add(&mut title, b": ");
    add(&mut title, subject);
    let mut out = List::uncapped();
    add(&mut out, &crate::text::fit(title.as_slice(), 120, "…".as_bytes()));
    for entry in entries {
        let before = commit_text(&entry.before);
        let after = commit_text(&entry.after);
        add(&mut out, b"\n\nPort landing ");
        add(&mut out, &after);
        add(&mut out, b" on ");
        add(&mut out, &n.source);
        add(&mut out, b" ");
        add(&mut out, &n.branch);
        add(&mut out, b" (from ");
        add(&mut out, &before);
        add(&mut out, b") into ");
        add(&mut out, &n.target);
        add(&mut out, b" ");
        add(&mut out, &n.target_branch);
        add(&mut out, b", for follow ");
        number(&mut out, u64::from(n.follow));
        add(&mut out, b" of this project.\n");
        if let Some(pull) = entry.pull {
            add(&mut out, b"Pull request: ");
            page(&mut out, n);
            add(&mut out, b"/pulls/");
            number(&mut out, pull);
            add(&mut out, b"\n");
        }
        add(&mut out, b"Compare: ");
        page(&mut out, n);
        add(&mut out, b"/compare/");
        add(&mut out, &before);
        add(&mut out, b"...");
        add(&mut out, &after);
        add(&mut out, b"\nDiff: in the read-only checkout of ");
        add(&mut out, &n.source);
        add(&mut out, b", `git diff ");
        add(&mut out, &before);
        add(&mut out, b" ");
        add(&mut out, &after);
        if !n.paths.is_empty() {
            add(&mut out, b" --");
            for path in &n.paths {
                add(&mut out, b" ");
                add(&mut out, path);
            }
        }
        add(&mut out, b"`\n\nCommits:\n");
        for commit in &entry.commits {
            let (head, body) = split(&commit.message);
            let id = commit_text(&commit.id);
            add(&mut out, b"- ");
            add(&mut out, id.get(..12)?);
            add(&mut out, b" ");
            add(&mut out, head);
            add(&mut out, b"\n");
            if !body.is_empty() {
                let room = rest.min(body.len());
                rest = rest.saturating_sub(room);
                let fitted = if room < body.len() && room < "…".len() {
                    Box::from("…".as_bytes())
                } else {
                    crate::text::fit(body, room, "…".as_bytes())
                };
                message_lines(&mut out, &fitted);
            }
        }
        if entry.more_commits > 0 {
            add(&mut out, b"and ");
            number(&mut out, u64::from(entry.more_commits));
            add(&mut out, b" more commits\n");
        }
        add(&mut out, b"\n");
        match &entry.files {
            None => add(&mut out, b"Files: unknown; the forge's comparison was incomplete."),
            Some(files) => {
                add(&mut out, b"Files in the follow's paths:\n");
                let mut cut = false;
                for path in files {
                    let size = path.len().saturating_add(3);
                    if !cut && size <= files_room {
                        add(&mut out, b"- ");
                        add(&mut out, path);
                        add(&mut out, b"\n");
                        files_room = files_room.saturating_sub(size);
                    } else {
                        cut = true;
                    }
                }
                if cut {
                    add(&mut out, "…\n".as_bytes());
                }
                if entry.more_files > 0 {
                    add(&mut out, b"and ");
                    number(&mut out, u64::from(entry.more_files));
                    add(&mut out, b" more files\n");
                }
            }
        }
    }
    Some(out.into_boxed())
}
fn message_lines(out: &mut List<u8>, bytes: &[u8]) {
    let mut from = 0;
    for (at, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            add(out, b"  ");
            add(out, bytes.get(from..at).expect("message line"));
            add(out, b"\n");
            from = at.saturating_add(1);
        }
    }
    if from < bytes.len() {
        add(out, b"  ");
        add(out, bytes.get(from..).expect("last message line"));
        add(out, b"\n");
    }
}
fn page(out: &mut List<u8>, n: &Words) {
    add(out, n.pages.strip_suffix(b"/").unwrap_or(&n.pages));
    add(out, b"/");
    add(out, &n.source);
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FollowCommit, FollowState};
    use alloc::{format, vec};
    fn entry(after: u8) -> FollowEntry {
        FollowEntry {
            before: [1; 32],
            after: [after; 32],
            pull: Some(17),
            commits: Box::new([FollowCommit {
                id: [after; 32],
                message: format!("Whole head {after}\n{}", "body rest ".repeat(40)).into_bytes().into(),
            }]),
            more_commits: 2,
            files: Some(Box::new([vec![b'a'; 180].into(), vec![b'b'; 180].into()])),
            more_files: 1,
            change: None,
            state: FollowState::NotMade,
            word: false,
            remake: false,
        }
    }
    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "the word-builder test inspects UTF-8 cuts in its ASCII fixtures and generated ellipses"
    )]
    fn briefs_cut_message_rest_before_files_keep_heads_and_merge_blocks() {
        let names = Words {
            follow: 1,
            source: b"org/vsr".as_slice().into(),
            branch: b"main".as_slice().into(),
            target: b"org/vsr-rs".as_slice().into(),
            target_branch: b"main".as_slice().into(),
            pages: b"https://forge.example".as_slice().into(),
            paths: Box::new([b"src".as_slice().into(), b"include".as_slice().into()]),
        };
        let entries = [entry(2), entry(3)];
        let full = build(&entries, &names, 4096).unwrap();
        let spec_bytes = u32::try_from(full.len()).unwrap();
        let parameters_bytes = 100;
        let fitted = build(&entries, &names, spec_bytes - parameters_bytes).unwrap();
        assert!(fitted.len() <= usize::try_from(spec_bytes - parameters_bytes).unwrap());
        let text = core::str::from_utf8(&fitted).unwrap();
        assert_eq!(text.matches("Port landing ").count(), 2);
        assert!(text.starts_with("Follow org/vsr 030303030303 and 1 more: Whole head 3"));
        assert!(text.contains("- 020202020202 Whole head 2\n") && text.contains("- 030303030303 Whole head 3\n"));
        assert!(text.contains('…'));
        assert_eq!(text.matches(&format!("- {}\n", "a".repeat(180))).count(), 2, "files survive the first cut");
        let files_cut = build(&entries, &names, spec_bytes - 1000).unwrap();
        let text = core::str::from_utf8(&files_cut).unwrap();
        assert!(!text.contains("body rest"), "all message rest is cut before files");
        assert!(text.contains("Whole head 2\n  …\n") && text.contains("Whole head 3\n  …\n"));
        assert!(text.contains("Files in the follow's paths:\n…\n") || text.contains("\n…\nand 1 more files"));
        assert!(files_cut.len() <= usize::try_from(spec_bytes - 1000).unwrap());
        assert!(build(&entries, &names, 120).is_none(), "mandatory head lines are never cut");
        let mut long = entry(4);
        long.commits[0].message = vec![b'x'; 200].into();
        let title = build(&[long], &names, 4096).unwrap();
        let title = core::str::from_utf8(&title).unwrap().lines().next().unwrap();
        assert!(title.len() <= 120 && title.ends_with('…'));
    }
}
