//! A landed look request rendered beside the print (temper/docs/design/client/journey-writer.md, 6.2).

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use skein_browser::boundary::{Element, Node};
use skein_lib::Token;
use temper_client_walk_lines::print::numbered_ids;

use crate::{
    host::{Ask, Host, browser::Command},
    look_compare::{
        Change, Side, done, git_ok, git_path,
        request::{self, Changes, Request, blob},
        text,
    },
};

use self::{
    browser::{Browser, PERSON},
    copies::{Copy, Page},
    measurement::Matched,
    report::{PageReport, Report},
};

mod browser;
mod copies;
mod diff;
mod html;
mod measurement;
mod report;
mod rules;
#[cfg(test)]
pub(crate) mod tests;

pub(crate) fn run(arguments: &[Box<[u8]>], host: &mut Host) -> u8 {
    let request = match request::read(arguments, host) {
        Ok(request) => request,
        Err(why) => return diagnostic(host, 2, &why),
    };
    let result = Plan::prepare(host, request).and_then(|plan| plan.check(host));
    match result {
        Ok(report) => {
            host.out(&report.finish());
            0
        }
        Err(why) => diagnostic(host, 3, &why),
    }
}

fn diagnostic(host: &mut Host, code: u8, why: &str) -> u8 {
    host.err(format!("look-render: {why}\n").as_bytes());
    code
}

#[derive(Clone, Copy)]
enum Candidate {
    HandMade,
    Journey { edited: bool },
}

struct Plan {
    request: Request,
    requested: Copy,
    printed: Copy,
    pages: BTreeMap<Page, Candidate>,
    rules: BTreeMap<Side, BTreeSet<String>>,
    scratch: PathBuf,
    diff: PathBuf,
}

impl Plan {
    fn prepare(host: &mut Host, request: Request) -> Result<Self, String> {
        let root = git_path(host, &["rev-parse", "--show-toplevel"])?;
        let scratch = git_path(host, &["rev-parse", "--path-format=absolute", "--git-path", "look-render"])?;
        done(host, Ask::Remove(scratch.clone()))?;
        done(host, Ask::Directory(scratch.clone()))?;
        let changes = request.changes(host, &root)?;
        let requested = Copy::requested(host, &root, &scratch, &request)?;
        let printed = Copy::printed(host, &root, &scratch)?;
        let mut rules = BTreeMap::new();
        for &side in &changes.overlays {
            let selectors = rules::selectors(host, &request, side)?;
            rules.insert(side, selectors);
        }
        let pages = candidates(host, &root, &request, &changes, &requested, &rules)?;
        let diff = scratch.join("diff");
        done(host, Ask::Directory(diff.clone()))?;
        Ok(Self { request, requested, printed, pages, rules, scratch, diff })
    }

    fn check(self, host: &mut Host) -> Result<Report, String> {
        let profile = self.scratch.join("browser");
        done(host, Ask::Directory(profile.clone()))?;
        let mut browser = Browser::new(host);
        let checked = browser.done(Command::Start { profile }).and_then(|()| self.render(&mut browser));
        let closed = browser.done(Command::Stop);
        let report = checked?;
        closed?;
        Ok(report)
    }

    fn render(&self, browser: &mut Browser<'_>) -> Result<Report, String> {
        browser.done(Command::Person { person: PERSON })?;
        let mut report = Report::default();
        for (page, &candidate) in &self.pages {
            browser.item = page.item();
            match candidate {
                Candidate::HandMade => report.uncompared(page, "hand-made"),
                Candidate::Journey { edited } => match self.page(browser, page, edited)? {
                    ResultForPage::Uncompared(why) => report.uncompared(page, why),
                    ResultForPage::Compared(lines) => report.compared(page, lines),
                    ResultForPage::Untouched => (),
                },
            }
        }
        Ok(report)
    }

    fn page(&self, browser: &mut Browser<'_>, page: &Page, edited: bool) -> Result<ResultForPage, String> {
        let print = self.printed.pages.get(page);
        if print.is_none() && edited {
            return Ok(ResultForPage::Uncompared(MISSING));
        }
        let request = browser.open(self.requested.url(page))?;
        let selected = self.select(browser, page.side, request)?;
        let result = if !edited && selected.is_empty() {
            ResultForPage::Untouched
        } else if let Some(print) = print {
            let right = browser.open(self.printed.url(page))?;
            let result = self.compare(browser, page, [request, right], print, edited, &selected)?;
            browser.done(Command::Close { page: right })?;
            result
        } else {
            ResultForPage::Uncompared(MISSING)
        };
        browser.done(Command::Close { page: request })?;
        Ok(result)
    }

    fn select(&self, browser: &mut Browser<'_>, side: Side, page: Token) -> Result<BTreeSet<Node>, String> {
        let mut selected = BTreeSet::new();
        if let Some(rules) = self.rules.get(&side) {
            for selector in rules {
                let (elements, _) = browser.elements(page, Some(selector))?;
                selected.extend(elements.into_iter().map(|element| element.node));
            }
        }
        Ok(selected)
    }

    fn compare(
        &self,
        browser: &mut Browser<'_>,
        page: &Page,
        opened: [Token; 2],
        print: &str,
        edited: bool,
        selected: &BTreeSet<Node>,
    ) -> Result<ResultForPage, String> {
        let request = &self.requested.pages[page];
        let tags = html::tags(request);
        let print_tags = html::tags(print);
        let left = browser.elements(opened[0], None)?;
        let right = browser.elements(opened[1], None)?;
        if !as_written(&tags, &left.0, left.1) || !as_written(&print_tags, &right.0, right.1) {
            return Ok(ResultForPage::Uncompared("its markup doesn't parse as written"));
        }
        let mut touched = left.0.iter().map(|element| selected.contains(&element.node)).collect::<Vec<_>>();
        let comparable = numbered_ids(request);
        if edited {
            if let Some(parent) = blob(browser.host, &self.request.parent, &page.path())? {
                let parent = text(&parent, &format!("{} in the request's parent", page.path()))?;
                let edits = diff::between(browser.host, &self.diff, &numbered_ids(parent), &comparable)?;
                diff::edited(&tags, &edits, &mut touched);
            } else {
                touched.fill(true);
            }
        }
        if !touched.contains(&true) {
            return Ok(ResultForPage::Untouched);
        }
        descendants(&left.0, &mut touched);
        let landed = numbered_ids(print);
        let alignment = diff::between(browser.host, &self.diff, &comparable, &landed)?;
        let pairs = diff::pairs(&html::tags(&comparable), &html::tags(&landed), &alignment)?;
        let matched = Matched { requested: &left.0, landed: &right.0, tags: &tags, pairs: &pairs, touched: &touched };
        Ok(ResultForPage::Compared(matched.compare(browser, opened)?))
    }
}

const MISSING: &str = "nothing printed beside it";

enum ResultForPage {
    Untouched,
    Uncompared(&'static str),
    Compared(PageReport),
}

fn as_written(tags: &[html::Tag<'_>], elements: &[Element], more: u32) -> bool {
    more == 0
        && tags.len() == elements.len()
        && tags.iter().zip(elements).all(|(tag, element)| tag.name.as_bytes().eq_ignore_ascii_case(&element.name))
}

fn descendants(elements: &[Element], touched: &mut [bool]) {
    let mut ancestor = None;
    for (element, marked) in elements.iter().zip(touched) {
        if ancestor.is_some_and(|depth| element.depth <= depth) {
            ancestor = None;
        }
        if ancestor.is_some() {
            *marked = true;
        } else if *marked {
            ancestor = Some(element.depth);
        }
    }
}

fn candidates(
    host: &mut Host,
    root: &Path,
    request: &Request,
    changes: &Changes,
    requested: &Copy,
    rules: &BTreeMap<Side, BTreeSet<String>>,
) -> Result<BTreeMap<Page, Candidate>, String> {
    let mut pages = BTreeMap::new();
    let hand_made = hand_made_paths(host, root, request, changes)?;
    for page in requested.pages.keys() {
        let Some(journey) = page.journey() else { continue };
        if hand_made.contains(&page.path()) {
            pages.insert(page.clone(), Candidate::HandMade);
        } else if page.numbered() && requested.walks.contains(&journey) {
            let edited = match changes.journeys.get(&journey) {
                Some(Change::Pages(files)) => {
                    page.relative.split_once('/').is_some_and(|(_, file)| files.contains(file))
                }
                Some(Change::HandMade) | None => false,
            };
            if edited || rules.get(&page.side).is_some_and(|selectors| !selectors.is_empty()) {
                pages.insert(page.clone(), Candidate::Journey { edited });
            }
        }
    }
    Ok(pages)
}

// The comparison collapses a hand-made directory to one row. The render check
// needs the changed HTML paths in those directories, not all their pages.
fn hand_made_paths(
    host: &mut Host,
    root: &Path,
    request: &Request,
    changes: &Changes,
) -> Result<BTreeSet<String>, String> {
    let directories = changes
        .journeys
        .iter()
        .filter(|(_, change)| matches!(change, Change::HandMade))
        .map(|(journey, _)| journey.path())
        .collect::<Vec<_>>();
    if directories.is_empty() {
        return Ok(BTreeSet::new());
    }
    let mut args = vec![
        "--literal-pathspecs",
        "diff-tree",
        "--no-commit-id",
        "--name-only",
        "--no-renames",
        "-r",
        "-z",
        &request.parent,
        &request.commit,
        "--",
    ];
    args.extend(directories.iter().map(String::as_str));
    let paths = git_ok(host, Some(root), &args)?;
    Ok(text(&paths, "changed hand-made path")?.split_terminator('\0').map(str::to_owned).collect())
}
