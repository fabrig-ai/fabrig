#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "step tests read files, format fixture errors and assemble text with ordinary Rust"
)]
#![expect(clippy::arithmetic_side_effects, reason = "test fixture offsets and counts are small")]
use super::*;
use alloc::{string::String, vec::Vec};

mod checks;
mod paging_search;
mod rules;

const KINDS: [Kind; 6] = [Kind::Index, Kind::Concept, Kind::Walk, Kind::Recipe, Kind::Reference, Kind::Guide];
const GUIDES: [&str; 6] = ["box", "budgets", "forgejo", "github", "server", "tasks"];

const fn fixture(kind: Kind, text: &'static str) -> Page {
    Page { name: "fixture", kind, title: "Fixture", summary: "Fixture.", text }
}

fn sections(entry: &Page) -> Vec<(&str, usize, &str)> {
    let mut starts = Vec::new();
    starts.push((entry.title, 0_usize));
    let mut fenced = false;
    let mut offset = 0;
    for line in entry.text.split_inclusive('\n') {
        if line.starts_with("```") {
            fenced = !fenced;
        } else if !fenced && let Some(heading) = line.strip_prefix("## ") {
            starts.push((heading.trim_end_matches('\n'), offset));
        }
        offset += line.len();
    }
    let mut out = Vec::new();
    for (i, &(heading, start)) in starts.iter().enumerate() {
        let end = starts.get(i + 1).map_or(entry.text.len(), |&(_, at)| at);
        out.push((heading, start, &entry.text[start..end]));
    }
    out
}
