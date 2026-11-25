use std::path::Path;

use temper_client_walk_lines::print::numbered_ids;

use super::{
    Ask, Host, Journey, Report, code, done, git, git_error, read,
    request::{Request, blob},
    text,
};

pub(super) fn compare(
    host: &mut Host,
    request: &Request,
    scratch: &Path,
    print: &Path,
    journey: &Journey,
    file: &str,
    report: &mut Report,
) -> Result<(), String> {
    let path = format!("{}/{file}", journey.path());
    let item = code(&format!("{}/{file}", journey.item()));
    let parent = blob(host, &request.parent, &path)?;
    let requested = blob(host, &request.commit, &path)?;
    let landed = read(host, &print.join(&path))?;
    let directory = scratch.join(format!("{}--{}--{file}", journey.side.name(), journey.name));
    done(host, Ask::Directory(directory.clone()))?;
    let page = Pages {
        parent: keep(host, &directory, "parent", parent.as_deref(), &path)?,
        requested: keep(host, &directory, "requested", requested.as_deref(), &path)?,
        landed: keep(host, &directory, "landed", landed.as_deref(), &path)?,
    };
    if landed.is_none() {
        if requested.is_some() {
            report.problem(&item, "missing");
        } else {
            report.row(&item, "landed");
        }
    } else if requested.is_none() && page.landed.is_empty() {
        report.detail(&format!("{item}, after line 0"), "differs", "the print still contains this empty page");
    } else if parent.is_none() {
        rows(report, &item, &diff(host, &directory, "requested", &page.landed, &page.requested)?, &[]);
    } else {
        let merge = merge(host, &directory, &page)?;
        let hunks = diff(host, &directory, merge.file, &page.landed, &merge.clean)?;
        let landed = page.landed.split_inclusive('\n').collect::<Vec<_>>();
        let conflicts = merge.conflicts.iter().map(|conflict| conflict.in_landing(&landed, &hunks)).collect::<Vec<_>>();
        rows(report, &item, &hunks, &conflicts);
    }
    Ok(())
}

struct Pages {
    parent: String,
    requested: String,
    landed: String,
}

fn keep(host: &mut Host, directory: &Path, name: &str, bytes: Option<&[u8]>, page: &str) -> Result<String, String> {
    let comparable = numbered_ids(text(bytes.unwrap_or_default(), &format!("{page} ({name})"))?);
    write(host, directory, name, &comparable)?;
    Ok(comparable)
}

fn write(host: &mut Host, directory: &Path, name: &str, content: &str) -> Result<(), String> {
    done(host, Ask::Write(directory.join(name), content.as_bytes().into()))
}

struct Merge {
    // Conflicts keep the landing's side in `clean`, so nearby clean edits can still get their own hunks.
    clean: String,
    file: &'static str,
    conflicts: Vec<Conflict>,
}

struct Conflict {
    after: usize,
    ours: Vec<String>,
    evidence: String,
}

fn merge(host: &mut Host, directory: &Path, pages: &Pages) -> Result<Merge, String> {
    let size = marker_size(pages);
    let args = [
        "-c",
        "merge.conflictStyle=merge",
        "merge-file",
        "-p",
        &format!("--marker-size={size}"),
        "-L",
        "landed",
        "-L",
        "parent",
        "-L",
        "requested",
        "landed",
        "parent",
        "requested",
    ];
    let result = git(host, Some(directory), &args)?;
    if !matches!(result.code, Some(0..=127)) {
        return Err(git_error("merge-file", &result));
    }
    let merged = text(&result.stdout, "git merge-file result")?;
    write(host, directory, "merged", merged)?;
    if result.code == Some(0) {
        return Ok(Merge { clean: merged.into(), file: "merged", conflicts: Vec::new() });
    }
    let conflicts = separate_conflicts(merged, size)?;
    write(host, directory, conflicts.file, &conflicts.clean)?;
    Ok(conflicts)
}

fn marker_size(pages: &Pages) -> usize {
    // A content line cannot be mistaken for a marker, even in a hand-edited page.
    [&pages.landed, &pages.parent, &pages.requested]
        .into_iter()
        .flat_map(|page| page.lines())
        .map(|line| {
            let first = line.as_bytes().first();
            if matches!(first, Some(b'<' | b'=' | b'>')) {
                line.bytes().take_while(|byte| Some(byte) == first).count() + 1
            } else {
                7
            }
        })
        .max()
        .unwrap_or(7)
        .max(7)
}

fn separate_conflicts(merged: &str, size: usize) -> Result<Merge, String> {
    let start = format!("{} landed\n", "<".repeat(size));
    let middle = format!("{}\n", "=".repeat(size));
    let end = format!("{} requested\n", ">".repeat(size));
    let mut clean = String::new();
    let mut conflicts = Vec::new();
    let mut after = 0;
    let mut lines = merged.split_inclusive('\n');
    while let Some(line) = lines.next() {
        if line != start {
            clean.push_str(line);
            after += 1;
            continue;
        }
        let mut evidence = line.to_owned();
        let mut ours = Vec::new();
        for line in lines.by_ref() {
            evidence.push_str(line);
            if line == middle {
                break;
            }
            ours.push(line.to_owned());
            clean.push_str(line);
        }
        for line in lines.by_ref() {
            evidence.push_str(line);
            if line == end {
                break;
            }
        }
        if !evidence.ends_with(&end) {
            return Err("git merge-file returned an incomplete conflict".into());
        }
        let count = ours.len();
        conflicts.push(Conflict { after, ours, evidence });
        after += count;
    }
    if conflicts.is_empty() {
        return Err("git merge-file reported conflicts without markers".into());
    }
    Ok(Merge { clean, file: "merged-with-landing-conflicts", conflicts })
}

#[derive(Clone, Copy)]
struct Span {
    after: usize,
    count: usize,
}

impl Span {
    fn parse(word: &str, sign: char) -> Option<Self> {
        let word = word.strip_prefix(sign)?;
        let (start, count) = word.split_once(',').unwrap_or((word, "1"));
        let start = start.parse::<usize>().ok()?;
        let count = count.parse::<usize>().ok()?;
        let after = if count == 0 { start } else { start.checked_sub(1)? };
        after.checked_add(count)?;
        Some(Self { after, count })
    }

    fn end(self) -> usize {
        self.after + self.count
    }

    fn item(self, page: &str) -> String {
        match self.count {
            0 => format!("{page}, after line {}", self.after),
            1 => format!("{page}, line {}", self.end()),
            _ => format!("{page}, lines {}-{}", self.after + 1, self.end()),
        }
    }
}

struct Hunk {
    from: Span,
    to: Span,
    evidence: String,
}

fn diff(host: &mut Host, directory: &Path, expected: &str, landed: &str, clean: &str) -> Result<Vec<Hunk>, String> {
    if landed == clean {
        return Ok(Vec::new());
    }
    let result = git(
        host,
        Some(directory),
        &[
            "diff",
            "--no-index",
            "--unified=0",
            "--inter-hunk-context=0",
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--text",
            "--diff-algorithm=myers",
            "--no-indent-heuristic",
            "--",
            "landed",
            expected,
        ],
    )?;
    match result.code {
        Some(1) => hunks(text(&result.stdout, "git diff result")?),
        _ => Err(git_error("diff --no-index", &result)),
    }
}

fn hunks(patch: &str) -> Result<Vec<Hunk>, String> {
    let mut hunks: Vec<Hunk> = Vec::new();
    for line in patch.split_inclusive('\n') {
        if let Some(header) = line.strip_prefix("@@ ") {
            let mut words = header.split_ascii_whitespace();
            let from = words.next().and_then(|word| Span::parse(word, '-'));
            let to = words.next().and_then(|word| Span::parse(word, '+'));
            let (Some(from), Some(to), Some("@@")) = (from, to, words.next()) else {
                return Err(format!("invalid git diff hunk header: {}", line.trim_end()));
            };
            hunks.push(Hunk { from, to, evidence: line.into() });
        } else if let Some(hunk) = hunks.last_mut() {
            hunk.evidence.push_str(line);
        }
    }
    if hunks.is_empty() { Err("git diff reported a difference without a hunk".into()) } else { Ok(hunks) }
}

impl Conflict {
    fn in_landing<'a>(&'a self, landed: &[&str], hunks: &[Hunk]) -> (Span, &'a str) {
        let at = project(self.after, hunks).min(landed.len());
        let count = self.ours.len();
        let matches = |lines: &[&str]| lines.iter().zip(&self.ours).all(|(left, right)| *left == right);
        // Git may align identical lines across a nearby edit. Prefer the landing's side itself to that alignment.
        let after = if count == 0 || landed.get(at..at.saturating_add(count)).is_some_and(matches) {
            at
        } else {
            landed
                .windows(count)
                .enumerate()
                .filter(|(_, lines)| matches(lines))
                .min_by_key(|(start, _)| start.abs_diff(at))
                .map_or(at, |(start, _)| start)
        };
        (Span { after, count: count.min(landed.len() - after) }, &self.evidence)
    }
}

fn project(after: usize, hunks: &[Hunk]) -> usize {
    let mut old = 0;
    let mut new = 0;
    for hunk in hunks {
        if after < hunk.to.after {
            break;
        }
        if after < hunk.to.end() {
            return hunk.from.after + (after - hunk.to.after).min(hunk.from.count);
        }
        old = hunk.from.end();
        new = hunk.to.end();
    }
    old + after - new
}

fn rows(report: &mut Report, page: &str, hunks: &[Hunk], conflicts: &[(Span, &str)]) {
    if hunks.is_empty() && conflicts.is_empty() {
        report.row(page, "landed");
        return;
    }
    let mut rows = hunks
        .iter()
        .map(|hunk| (hunk.from, "differs", hunk.evidence.as_str()))
        .chain(conflicts.iter().map(|&(span, evidence)| (span, "conflict", evidence)))
        .collect::<Vec<_>>();
    rows.sort_by_key(|(span, _, _)| span.after);
    for (span, result, evidence) in rows {
        report.detail(&span.item(page), result, evidence);
    }
}
