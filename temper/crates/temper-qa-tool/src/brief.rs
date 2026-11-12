//! Inputs at the merge and the brief's ordered sections (qa/triage.md, 3).
use crate::{Home, Landing, Ledger, Pages, Refusal, faces::GitFace};
use std::fmt::Write;

pub(crate) struct Inputs {
    pub pages: Pages,
    pub setups: Vec<String>,
    pub patch: Vec<u8>,
    drafts: Vec<String>,
    stat: String,
    qa: Vec<String>,
    surfaces: String,
    catalogue: String,
}
impl Inputs {
    pub fn read(git: &mut impl GitFace, landing: &Landing) -> Result<Self, Refusal> {
        let commit = &landing.commit;
        let parent = format!("{commit}^1");
        let drafts = landing_drafts(git, landing)?;
        let mut pages = Pages::new();
        let mut surfaces = String::new();
        for (surface, directory, journeys) in [
            ("web", "temper/docs/design/client/pages", "temper/docs/design/client/web/journeys.md"),
            ("tui", "temper/docs/design/client/tui/pages", "temper/docs/design/client/tui/journeys.md"),
        ] {
            let mut names = Vec::new();
            for entry in git.entries(commit, directory)? {
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
        let mut setups = vec!["fresh".into()];
        let mut catalogue = String::from("- fresh: a fresh box at its welcome with nothing set up\n");
        let directory = "temper/docs/development/qa/setups";
        for entry in git.entries(commit, directory)? {
            if entry.directory && entry.name != "fresh" {
                let path = format!("{directory}/{}/README.md", entry.name);
                let bytes = git.file(commit, &path)?.unwrap_or_default();
                let text = String::from_utf8_lossy(&bytes);
                writeln!(catalogue, "- {}: {} ({path})", entry.name, first_paragraph(&text)).expect("string");
                setups.push(entry.name);
            }
        }
        let mut qa = Vec::new();
        for value in git.qa_trailers(&parent, commit)? {
            if !qa.contains(&value) {
                qa.push(value);
            }
        }
        Ok(Self {
            pages,
            setups,
            drafts,
            qa,
            surfaces,
            catalogue,
            patch: git.diff(&parent, commit)?,
            stat: git.stat(&parent, commit)?,
        })
    }
    pub fn render(&self, home: &Home, number: u64, landing: &Landing, ledger: &Ledger) -> String {
        let mut text = String::from(
            "# Triage\n\nJudge what a person now sees or does in the web client or TUI, the green path only: the draft's intended use, weighed by v0.1.0's stories and each part's bar (`temper/docs/design/draft/v0.1.0.md`, 4–5). Edge cases, failures and recovery wait; ctl and help are outside these surfaces. Read the draft's promises and write expectations before anything is tried. If nothing a person meets changes, answer no QA with a line saying why. Read only: start no box. A security hole or lost data is a finding wherever it is.\n",
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
        for case in &ledger.cases {
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
        text.push_str("\n## Commands\n\nRun `temper-qa triage none WHY` (WHY one nonempty argument) or `temper-qa triage case FILE` (FILE a JSON file, or - for stdin). Each answer is recorded at once; an early end loses nothing recorded. A case on a queued case's surface and page joins it.\n\nCase form (adds and see optional; all other fields required):\n```json\n{\"surface\":\"web\",\"page\":\"PAGE\",\"setup\":\"fresh\",\"adds\":\"what it needs\",\"steps\":[{\"do\":\"an act\",\"expect\":\"the draft's promise\",\"see\":\"words to see\"}]}\n```\nUse only the pages and setups above; give at least one step, each with nonempty do and expect.\n");
        text
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
