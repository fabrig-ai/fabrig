//! A landed look request rendered beside the print (temper/docs/design/client/journey-writer.md, 6.2).

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    path::{Path, PathBuf},
};

use skein_browser::{Limits, boundary::Element};
use skein_lib::Token;

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
    copies::{Copy, Page, file_url},
    gallery::{Drawing, Gallery},
    measurement::{Matched, Properties},
    report::{PageReport, Report},
    rules::{Selected, Selectors},
};

mod browser;
mod copies;
mod diff;
mod gallery;
mod html;
mod measurement;
pub(crate) mod report;
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

enum Candidate {
    Uncompared(Why),
    Gallery(String),
    Journey { edited: bool },
}

#[derive(Clone, Copy)]
enum Kind {
    Gallery,
    Journey { edited: bool },
}

struct Plan {
    request: Request,
    requested: Copy,
    printed: Copy,
    gallery: Gallery,
    pages: BTreeMap<Page, Candidate>,
    rules: BTreeMap<Side, Selectors>,
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
        let states = pages.values().filter_map(|candidate| match candidate {
            Candidate::Gallery(state) => Some(state.as_str()),
            Candidate::Uncompared(_) | Candidate::Journey { .. } => None,
        });
        let gallery = Gallery::prepare(host, &root, &scratch, states.collect())?;
        let diff = scratch.join("diff");
        done(host, Ask::Directory(diff.clone()))?;
        Ok(Self { request, requested, printed, gallery, pages, rules, scratch, diff })
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
        for (page, candidate) in &self.pages {
            browser.item = page.item();
            let result = match candidate {
                Candidate::Uncompared(why) => ResultForPage::Uncompared(*why),
                Candidate::Gallery(state) => self.page(browser, page, self.gallery.page(state), Kind::Gallery)?,
                Candidate::Journey { edited } => {
                    let print = self.printed.pages.get(page).map(|source| (self.printed.path(page), source.as_str()));
                    self.page(browser, page, print, Kind::Journey { edited: *edited })?
                }
            };
            match result {
                ResultForPage::Uncompared(why) => report.uncompared(page, why),
                ResultForPage::Compared(lines) => report.compared(page, lines),
                ResultForPage::Untouched => (),
            }
        }
        Ok(report)
    }

    fn page(
        &self,
        browser: &mut Browser<'_>,
        page: &Page,
        print: Option<(PathBuf, &str)>,
        kind: Kind,
    ) -> Result<ResultForPage, String> {
        let edited = matches!(kind, Kind::Journey { edited: true });
        if print.is_none() && (edited || matches!(kind, Kind::Gallery)) {
            return Ok(ResultForPage::Uncompared(Why::Missing));
        }
        let request = browser.open(file_url(&self.requested.path(page)))?;
        let selected = self.select(browser, page.side, request)?;
        let result = if !edited && selected.is_empty() {
            ResultForPage::Untouched
        } else if let Some((path, print)) = print {
            let right = browser.open(file_url(&path))?;
            let result = self.compare(browser, page, [request, right], print, kind, &selected)?;
            browser.done(Command::Close { page: right })?;
            result
        } else {
            ResultForPage::Uncompared(Why::Missing)
        };
        browser.done(Command::Close { page: request })?;
        Ok(result)
    }

    fn select(&self, browser: &mut Browser<'_>, side: Side, page: Token) -> Result<Selected<'_>, String> {
        let mut selected = Selected::new();
        if let Some(rules) = self.rules.get(&side) {
            for (selector, names) in rules {
                let (elements, _) = browser.elements(page, Some(selector))?;
                for element in elements {
                    selected.entry(element.node).or_default().extend(names.iter().map(String::as_str));
                }
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
        kind: Kind,
        selected: &Selected<'_>,
    ) -> Result<ResultForPage, String> {
        let request = &self.requested.pages[page];
        let tags = html::tags(request);
        let print_tags = html::tags(print);
        let left = browser.elements(opened[0], None)?;
        let right = browser.elements(opened[1], None)?;
        if left.1 > 0 || right.1 > 0 {
            return Ok(ResultForPage::Uncompared(Why::TooLarge));
        }
        if !as_written(&tags, &left.0) || !as_written(&print_tags, &right.0) {
            return Ok(ResultForPage::Uncompared(Why::Markup));
        }
        let comparable = html::comparable(request);
        let comparable_tags = html::tags(&comparable);
        let touched: Vec<_> = match kind {
            Kind::Gallery => {
                left.0.iter().map(|element| selected.get(&element.node).map(Properties::Declared)).collect()
            }
            Kind::Journey { edited } => {
                let mut touched = left.0.iter().map(|element| selected.contains_key(&element.node)).collect::<Vec<_>>();
                if edited {
                    if let Some(parent) = blob(browser.host, &self.request.parent, &page.path())? {
                        let parent = text(&parent, &format!("{} in the request's parent", page.path()))?;
                        let edits = diff::between(
                            browser.host,
                            &self.diff,
                            ("parent", &html::comparable(parent)),
                            ("requested", &comparable),
                        )?;
                        diff::edited(&comparable_tags, &edits, &mut touched);
                    } else {
                        touched.fill(true);
                    }
                }
                descendants(&left.0, &mut touched);
                touched.into_iter().map(|marked| marked.then_some(Properties::Journey)).collect()
            }
        };
        if !touched.iter().any(Option::is_some) {
            return Ok(ResultForPage::Untouched);
        }
        let landed = html::comparable(print);
        let alignment = diff::between(browser.host, &self.diff, ("requested", &comparable), ("landed", &landed))?;
        let pairs = diff::pairs(&comparable_tags, &html::tags(&landed), &alignment)?;
        let matched = Matched { requested: &left.0, landed: &right.0, tags: &tags, pairs: &pairs, touched: &touched };
        Ok(matched.compare(browser, opened)?.map_or(ResultForPage::Uncompared(Why::Missing), ResultForPage::Compared))
    }
}

// Derive the test's coverage list from the same cases as the report's words.
macro_rules! reasons {
    ($($why:ident => $words:expr),+ $(,)?) => {
        #[derive(Clone, Copy)]
        pub(crate) enum Why { $($why),+ }

        impl Why {
            #[cfg(test)]
            pub(crate) const ALL: &[Self] = &[$(Self::$why),+];

            pub(crate) fn words(self) -> Cow<'static, str> {
                match self { $(Self::$why => $words.into()),+ }
            }
        }
    };
}

reasons! {
    HandMade => "hand-made",
    NotRequested => "not the requested look",
    Missing => "nothing printed beside it",
    Markup => "its markup doesn't parse as written",
    TooLarge => format!("too large to compare, over {} elements", Limits::default().elements),
}

enum ResultForPage {
    Untouched,
    Uncompared(Why),
    Compared(PageReport),
}

fn as_written(tags: &[html::Tag<'_>], elements: &[Element]) -> bool {
    tags.len() == elements.len()
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
    rules: &BTreeMap<Side, Selectors>,
) -> Result<BTreeMap<Page, Candidate>, String> {
    let mut pages = BTreeMap::new();
    let hand_made = hand_made_paths(host, root, request, changes)?;
    let mut drawings = BTreeMap::new();
    for page in requested.pages.keys() {
        let Some(journey) = page.journey() else { continue };
        let path = page.path();
        if hand_made.contains(&path) {
            let candidate = if page.side == Side::Web {
                let (directory, file) = path.rsplit_once('/').expect("a page in a directory");
                let drawing = match drawings.entry(directory.to_owned()) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert(Drawing::read(host, request, directory)?),
                };
                match drawing {
                    Some(drawing) if drawing.requests(file) => Candidate::Gallery(drawing.state.clone()),
                    Some(_) => Candidate::Uncompared(Why::NotRequested),
                    None => Candidate::Uncompared(Why::HandMade),
                }
            } else {
                Candidate::Uncompared(Why::HandMade)
            };
            pages.insert(page.clone(), candidate);
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
