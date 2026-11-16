//! Inputs at the merge and the brief's ordered sections (qa/triage.md, 3).
use crate::{
    Home, Kind, Landing, Ledger, Pages, Refusal,
    faces::{Changed, GitFace},
};
use std::{collections::BTreeMap, fmt::Write};

const GENERATED_SEGMENTS: &[&str] = &["generated", "golden"];
const REFERENCE_DIRECTORY: &str = "temper/docs/help/reference/";
const SETUPS_DIRECTORY: &str = "temper/docs/development/qa/setups";

pub(crate) fn setup_names(
    git: &mut impl GitFace,
    commit: &str,
    configured: &[String],
) -> Result<Vec<(String, Kind)>, Refusal> {
    let mut setups = vec![("fresh".into(), Kind::Box)];
    let directory = SETUPS_DIRECTORY;
    let entries = git.entries(commit, directory)?;
    let ctl_only = !configured.iter().any(|surface| surface == "web" || surface == "tui");
    let mut recipes = BTreeMap::new();
    for entry in &entries {
        if entry.directory
            && entry.name != "fresh"
            && let Some(bytes) = git.file(commit, &format!("{directory}/{}/recipe", entry.name))?
            && let Ok(recipe) = String::from_utf8(bytes)
        {
            let config = git
                .entries(commit, &format!("{directory}/{}", entry.name))?
                .iter()
                .any(|entry| entry.directory && entry.name == "config");
            recipes.insert(entry.name.clone(), (recipe, config));
        }
    }
    for entry in entries {
        if !entry.directory || entry.name == "fresh" {
            continue;
        }
        let kind = crate::kind::chain_kind(&entry.name, |name| {
            let Some((recipe, config)) = recipes.get(name) else { return Ok(None) };
            if recipe.lines().next() != Some("recipe 1")
                || (ctl_only
                    && recipe.lines().any(|line| matches!(line.split_whitespace().next(), Some("web" | "tui"))))
            {
                return Ok(None);
            }
            Ok(Some(crate::kind::Setup { parent: crate::kind::parent(recipe).map(str::to_owned), config: *config }))
        })?;
        if !ctl_only || kind.is_some() {
            setups.push((entry.name, kind.unwrap_or_default()));
        }
    }
    Ok(setups)
}

fn read_setups(
    git: &mut impl GitFace,
    commit: &str,
    configured: &[String],
) -> Result<(Vec<(String, Kind)>, String), Refusal> {
    let setups = setup_names(git, commit, configured)?;
    let mut catalogue = String::from("- fresh (box): a fresh box at its welcome with nothing set up\n");
    for (name, kind) in setups.iter().skip(1) {
        let path = format!("{SETUPS_DIRECTORY}/{name}/README.md");
        let bytes = git.file(commit, &path)?.unwrap_or_default();
        let text = String::from_utf8_lossy(&bytes);
        writeln!(catalogue, "- {name} ({}): {} ({path})", kind.name(), first_paragraph(&text)).expect("string");
    }
    Ok((setups, catalogue))
}

fn opening(pages: &Pages) -> &str {
    match (pages.contains_key("ctl"), pages.contains_key("web"), pages.contains_key("tui")) {
        (true, true, true) => {
            "ctl, the web client or the TUI, ctl first: write ctl cases before web and TUI cases, the person's acts and temper's account as `temper ctl` gives them"
        }
        (true, true, false) => {
            "ctl or the web client, ctl first: write ctl cases before web cases, the person's acts and temper's account as `temper ctl` gives them"
        }
        (true, false, true) => {
            "ctl or the TUI, ctl first: write ctl cases before TUI cases, the person's acts and temper's account as `temper ctl` gives them"
        }
        (true, false, false) => "ctl, the person's acts and temper's account as `temper ctl` gives them",
        (false, true, true) => "the web client or the TUI",
        (false, true, false) => "the web client",
        (false, false, true) => "the TUI",
        (false, false, false) => "the configured surfaces (none)",
    }
}

fn generated_directory(path: &str) -> Option<&str> {
    if path.starts_with(REFERENCE_DIRECTORY) {
        return Some(REFERENCE_DIRECTORY);
    }
    let mut remaining = path;
    while let Some((segment, rest)) = remaining.split_once('/') {
        if GENERATED_SEGMENTS.contains(&segment) {
            return path.strip_suffix(rest);
        }
        remaining = rest;
    }
    None
}

#[derive(Default)]
struct Counts {
    files: usize,
    added: u64,
    deleted: u64,
}
impl Counts {
    fn include(&mut self, path: &Changed) {
        self.files += 1;
        self.added += path.added.unwrap_or(0);
        self.deleted += path.deleted.unwrap_or(0);
    }
}

/// Fold generated directories at their first path, then total every path (qa/triage.md, 3).
#[must_use]
pub fn stat_text(paths: &[Changed]) -> String {
    let mut paths: Vec<_> = paths.iter().collect();
    paths.sort_by(|left, right| left.path.cmp(&right.path));
    let mut directories: BTreeMap<&str, Counts> = BTreeMap::new();
    let mut total = Counts::default();
    for path in &paths {
        total.include(path);
        if let Some(directory) = generated_directory(&path.path) {
            directories.entry(directory).or_default().include(path);
        }
    }
    let mut text = String::new();
    for path in paths {
        if let Some(directory) = generated_directory(&path.path) {
            if let Some(counts) = directories.remove(directory) {
                writeln!(text, "{directory} | {} files, +{} -{}", counts.files, counts.added, counts.deleted)
                    .expect("string");
            }
        } else if let (Some(added), Some(deleted)) = (path.added, path.deleted) {
            writeln!(text, "{} | +{added} -{deleted}", path.path).expect("string");
        } else {
            writeln!(text, "{} | binary", path.path).expect("string");
        }
    }
    writeln!(text, "{} files changed, +{} -{}", total.files, total.added, total.deleted).expect("string");
    text
}

pub(crate) struct Inputs {
    pub pages: Pages,
    pub setups: Vec<String>,
    pub deployments: Vec<String>,
    pub patch: Vec<u8>,
    drafts: Vec<String>,
    stat: String,
    qa: Vec<String>,
    surfaces: String,
    catalogue: String,
}
impl Inputs {
    pub fn read(
        git: &mut impl GitFace,
        landing: &Landing,
        configured: &[String],
        build: &str,
    ) -> Result<Self, Refusal> {
        let commit = &landing.commit;
        let parent = format!("{commit}^1");
        let drafts = landing_drafts(git, landing)?;
        let mut pages = Pages::new();
        let mut surfaces = String::new();
        if configured.iter().any(|surface| surface == "ctl") {
            let mut groups = Vec::new();
            for entry in git.entries(build, REFERENCE_DIRECTORY.trim_end_matches('/'))? {
                if !entry.directory
                    && let Some(group) = entry.name.strip_prefix("ctl-").and_then(|name| name.strip_suffix(".md"))
                {
                    groups.push(group.to_owned());
                }
            }
            groups.sort();
            groups.dedup();
            for group in &groups {
                writeln!(surfaces, "- ctl: {group}: {REFERENCE_DIRECTORY}ctl-{group}.md").expect("string");
            }
            pages.insert("ctl".into(), groups);
            surfaces.push_str(
                "- ctl design: temper/docs/design/client/ctl/README.md; temper/docs/design/client/ctl/tools.md\n",
            );
        }
        for (surface, directory, journeys) in [
            ("web", "temper/docs/design/client/pages", "temper/docs/design/client/web/journeys.md"),
            ("tui", "temper/docs/design/client/tui/pages", "temper/docs/design/client/tui/journeys.md"),
        ] {
            if !configured.iter().any(|name| name == surface) {
                continue;
            }
            let mut names = Vec::new();
            for entry in git.entries(build, directory)? {
                let name = if entry.directory { Some(entry.name.as_str()) } else { page_name(&entry.name) };
                if let Some(name) = name {
                    names.push(name.to_owned());
                    writeln!(surfaces, "- {surface}: {name}: {directory}/{}", entry.name).expect("string");
                }
            }
            names.sort();
            names.dedup();
            pages.insert(surface.into(), names);
            writeln!(surfaces, "- {surface} journeys: {journeys}").expect("string");
        }
        let (catalogue_setups, catalogue) = read_setups(git, build, configured)?;
        let deployments = catalogue_setups
            .iter()
            .filter(|(_, kind)| *kind == Kind::Deployment)
            .map(|(name, _)| name.clone())
            .collect();
        let setups = catalogue_setups.into_iter().map(|(name, _)| name).collect();
        let mut qa = Vec::new();
        for value in git.qa_trailers(&parent, commit)? {
            if !qa.contains(&value) {
                qa.push(value);
            }
        }
        Ok(Self {
            pages,
            setups,
            deployments,
            drafts,
            qa,
            surfaces,
            catalogue,
            patch: git.diff(&parent, commit)?,
            stat: stat_text(&git.stat(&parent, commit)?),
        })
    }
    pub fn render(&self, home: &Home, number: u64, landing: &Landing, ledger: &Ledger) -> String {
        let mut text = format!(
            "# Triage\n\nJudge what a person now sees or does through {}. The green path only: the draft's intended use, weighed by v0.1.0's stories and each part's bar (`temper/docs/design/draft/v0.1.0.md`, 4–5). Edge cases, failures and recovery wait. Read the draft's promises and write expectations before anything is tried. If nothing a person meets changes, answer no QA with a line saying why. The fakes alone: the person's forge is the box's fake Forgejo, what a person meets that these cannot show is a gap. Read only: start no box. A security hole or lost data is a finding wherever it is.\n",
            opening(&self.pages),
        );
        writeln!(
            text,
            "Work within {} seconds.\n\n## Landing\n\nLanding {}; goal {}; pull {}; merge {}\n{}\n",
            home.configuration.bounds.triage,
            landing.number,
            landing.goal,
            landing.pull.map_or_else(|| "none".into(), |pull| pull.to_string()),
            landing.commit,
            landing.title
        )
        .expect("string");
        writeln!(text, "Tree: {}\n", home.triage_tree().display()).expect("string");
        text.push_str("## Drafts\n\n");
        if self.drafts.is_empty() {
            text.push_str("No draft is named; the request was the title's words.\n");
        }
        for path in &self.drafts {
            writeln!(text, "- {path}").expect("string");
        }
        writeln!(
            text,
            "\n## Diff\n\n{}\nWhole diff: {}\n\n## QA trailers\n",
            self.stat,
            home.session_directory(number).join("diff.patch").display()
        )
        .expect("string");
        if self.qa.is_empty() {
            text.push_str("None.\n");
        }
        for value in &self.qa {
            writeln!(text, "- QA: {value}").expect("string");
        }
        writeln!(text, "\n## Surfaces\n\n{}\n## Setups\n\n{}\n## Queued cases\n", self.surfaces, self.catalogue)
            .expect("string");
        if ledger.cases.is_empty() {
            text.push_str("None.\n");
        }
        let mut cases: Vec<_> = ledger.cases.iter().collect();
        cases.sort_by_key(|case| (case.surface != "ctl", case.surface != "web"));
        for case in cases {
            writeln!(
                text,
                "Case {}: {} / {}; landings {}",
                case.number,
                case.surface,
                case.page,
                crate::ledger::numbers(&case.landings())
            )
            .expect("string");
            for ask in &case.asks {
                writeln!(text, "- landing {}; setup {}; adds: {}", ask.landing, ask.setup, ask.adds).expect("string");
                for step in &ask.steps {
                    writeln!(text, "  - do: {}\n    expect: {}", step.act, step.expect).expect("string");
                    if let Some(see) = &step.see {
                        writeln!(text, "    see: {see}").expect("string");
                    }
                }
            }
        }
        self.commands(&mut text);
        text.push_str("\nA landing a person would meet that no case can judge yet, because ctl offers no way to do or see it, the fakes show no such behaviour, or QA's lines and setups can't reach it, is a gap: `temper-qa triage gap KIND WHY`, never none. KIND is ctl, fake or qa; WHY one nonempty argument. Gaps may accompany cases.\n");
        for gap in ledger.gaps.iter().filter(|gap| gap.landing == landing.number && gap.case.is_none()) {
            writeln!(text, "\nEarlier gap {} ({}; {}): {}", gap.number, gap.kind, gap.state.name(), gap.why)
                .expect("string");
        }
        text
    }
    fn commands(&self, text: &mut String) {
        if self.pages.is_empty() {
            text.push_str("\n## Commands\n\nNo surfaces are configured. Run `temper-qa triage none WHY` (WHY one nonempty argument). The answer is recorded at once.\n");
            return;
        }
        let example = if self.pages.contains_key("web") {
            "web"
        } else if self.pages.contains_key("ctl") {
            "ctl"
        } else {
            "tui"
        };
        let names: Vec<_> = ["ctl", "web", "tui"].into_iter().filter(|name| self.pages.contains_key(*name)).collect();
        let names = match names.as_slice() {
            [] => "none".into(),
            [name] => (*name).to_owned(),
            [first, second] => format!("{first} or {second}"),
            _ => "ctl, web or tui".into(),
        };
        write!(text, "\n## Commands\n\nRun `temper-qa triage none WHY` (WHY one nonempty argument) or `temper-qa triage case FILE` (FILE a JSON file, or - for stdin). Each answer is recorded at once; an early end loses nothing recorded. A case on a queued case's surface and page joins it.\n\nCase form (adds and see optional; all other fields required):\n```json\n{{\"surface\":\"{example}\",\"page\":\"PAGE\",\"setup\":\"fresh\",\"adds\":\"what it needs\",\"steps\":[{{\"do\":\"an act\",\"expect\":\"the draft's promise\",\"see\":\"words to see\"}}]}}\n```\nA case's surface is {names};").expect("string");
        if self.pages.contains_key("ctl") {
            text.push_str(r#" for ctl, page is one of its groups. For ctl steps, write do as ctl or forge drive lines. Quoted bytes take printable ASCII alone; write other bytes as `\xHH`, with `\"` and `\\` for quote and backslash."#);
        }
        text.push_str(
            " Use only the pages and setups above; give at least one step, each with nonempty do and expect.\n",
        );
    }
}

pub(crate) fn landing_drafts(git: &mut impl GitFace, landing: &Landing) -> Result<Vec<String>, Refusal> {
    let commit = &landing.commit;
    let message = git.message(commit)?;
    let mut drafts = present_drafts(
        git,
        commit,
        message.split(|c: char| !(c.is_alphanumeric() || matches!(c, '/' | '.' | '-' | '_'))).filter_map(draft_path),
    )?;
    if drafts.is_empty() {
        let paths = git.changed_paths(&format!("{commit}^1"), commit)?;
        drafts = present_drafts(git, commit, paths.into_iter().filter_map(|path| draft_path(&path)))?;
    }
    Ok(drafts)
}

fn present_drafts(
    git: &mut impl GitFace,
    commit: &str,
    paths: impl Iterator<Item = String>,
) -> Result<Vec<String>, Refusal> {
    let mut drafts = Vec::new();
    for path in paths {
        if !drafts.contains(&path) && git.file(commit, &path)?.is_some() {
            drafts.push(path);
        }
    }
    Ok(drafts)
}
fn draft_path(word: &str) -> Option<String> {
    let word = word.trim_end_matches('.');
    let (tree, leaf) = word.split_once("/docs/design/draft/")?;
    if tree.is_empty() || tree.contains('/') || leaf.contains('/') {
        return None;
    }
    let stem = leaf.strip_suffix(".md")?;
    let stem = stem.strip_suffix(".execution").or_else(|| stem.strip_suffix(".status")).unwrap_or(stem);
    if stem.is_empty() || stem == "notes" || stem.strip_suffix(".notes").is_some() {
        return None;
    }
    Some(format!("{tree}/docs/design/draft/{stem}.md"))
}
fn page_name(file: &str) -> Option<&str> {
    let name = file.strip_suffix(".md")?;
    if name == "README" || name.ends_with(".status") { None } else { Some(name) }
}
fn first_paragraph(text: &str) -> String {
    text.split("\n\n")
        .find(|part| !part.trim().is_empty() && !part.trim_start().starts_with('#'))
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
