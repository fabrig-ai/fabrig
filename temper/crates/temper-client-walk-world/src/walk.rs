//! Walk a root-first chain in one world (temper/docs/design/client/journey-walks.md, 4).
use crate::{
    facts,
    tui::Terminal,
    web::{Failed, Page},
};
use skein_fake_person::{Face, Find, Found, TreeFace};
use skein_lib::Duration;
use skein_ui::{Act, Role, Tree};
use skein_ui_screen_terminal::Key;
use temper_client_shell_tui::journey;
use temper_client_tui::Event;
use temper_client_walk_lines::{
    line::{Record, Tui, Web},
    print::{self, Lead, Mark},
    walk::{Client, Numbered, Walk},
};

const WITHIN: Duration = Duration::from_secs(60);
const TUI_WITHIN: Duration = Duration::from_secs(30);

pub enum Marks {
    Web(Vec<Mark>),
    Tui(Vec<journey::Mark>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Stopped {
    pub journey: String,
    pub line: u32,
    pub why: String,
}
impl std::fmt::Display for Stopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} line {}: {}", self.journey, self.line, self.why)
    }
}

/// Execute all lines; a stop returns no marks.
pub fn walk(walk: &Walk) -> Result<Marks, Stopped> {
    let Some(last) = walk.chain.last() else {
        return Err(Stopped { journey: String::new(), line: 0, why: "empty walk".into() });
    };
    let stop = |why| Stopped { journey: last.name.clone(), line: 0, why };
    let mut joined = temper_fake::llm::ScriptJoin::empty();
    let mut any = false;
    for journey in &walk.chain {
        if let Some(scripts) = &journey.scripts {
            joined.push(temper_playground::stories::A.llm, scripts).map_err(|why| Stopped {
                journey: journey.name.clone(),
                line: 0,
                why: format!("scripts.json: {why:?}"),
            })?;
            any = true;
        }
    }
    let scripts = joined.into_bytes();
    let start =
        facts::start(any.then_some(scripts.as_ref())).map_err(|why| stop(format!("the world failed: {why}")))?;
    let mut page = Page::start(start, last.name.as_bytes())
        .map_err(|why| stop(failed(why, "the main region not drawn within 60 s")))?;
    let mut web_marks = Vec::new();
    let mut tui_marks = Vec::new();
    for (index, journey) in walk.chain.iter().enumerate() {
        for (at, numbered) in journey.lines.iter().enumerate() {
            let result = match numbered.line.record() {
                Record::Web(Web::Mark) => {
                    if index + 1 == walk.chain.len() {
                        let (acted, lead) = link(page.tree(), &journey.lines[at + 1..]);
                        web_marks.push(Mark { html: page.html(), heading: print::heading(page.tree()), acted, lead });
                    }
                    Ok(())
                }
                Record::Tui(Tui::Mark) => {
                    if index + 1 == walk.chain.len() {
                        tui_mark(&mut page, &journey.lines[at + 1..]).map(|mark| tui_marks.push(mark))
                    } else {
                        Ok(())
                    }
                }
                record => step(&mut page, record),
            };
            result.map_err(|why| Stopped { journey: journey.name.clone(), line: numbered.number, why })?;
        }
    }
    Ok(match last.client {
        Client::Web => Marks::Web(web_marks),
        Client::Tui => Marks::Tui(tui_marks),
    })
}

fn tui_mark(page: &mut Page, following: &[Numbered]) -> Result<journey::Mark, String> {
    page.tui_start().map_err(|why| failed(why, "the TUI not drawn within 30 s"))?;
    let terminal = page.tui().expect("started terminal");
    let frame = terminal.frame().clone();
    let key = tui_link(terminal, following);
    Ok(journey::Mark { frame, live: None, key })
}

// Resolve the first key on the frame as marked, not on a later frame.
fn tui_link(terminal: &Terminal, following: &[Numbered]) -> Option<Key> {
    let mut key = None;
    for numbered in following {
        match numbered.line.record() {
            Record::Tui(Tui::Mark) => return key,
            Record::Tui(Tui::Key(next)) if key.is_none() => match terminal.event_of(next) {
                Some(Event::Typed { .. }) | None => return None,
                Some(_) => key = Some(next.clone()),
            },
            Record::Tui(Tui::See(_) | Tui::Key(_)) | Record::Web(Web::See(_)) => {}
            _ if key.is_some() => {}
            _ => return None,
        }
    }
    None
}

fn find(role: Role, name: &[u8], order: u32) -> Find {
    if role == Role::TextField { Find::text_box(name) } else { Find::exactly_named(role, name) }.nth(order)
}

// Resolve the future line on the tree at the mark, before any typing or act can
// change it. The immutable lines let us retain the result rather than copy a Tree.
fn link(tree: &Tree, following: &[Numbered]) -> (Vec<skein_ui::NodeId>, Lead) {
    let mut next_act = None;
    for numbered in following {
        match numbered.line.record() {
            Record::Web(Web::Mark) => {
                return next_act.map_or((Vec::new(), Lead::Press), |(role, name, order, lead)| {
                    let found = TreeFace::new(tree).find(&find(role, name, order));
                    match found {
                        Found::Node(node) => (vec![node], lead),
                        Found::Absent | Found::Pending => (Vec::new(), lead),
                    }
                });
            }
            Record::Web(Web::Act { role, name, order, act }) if next_act.is_none() => {
                let lead = match act {
                    Act::Press => Some(Lead::Press),
                    Act::Submit => Some(Lead::Submit),
                    Act::Toggle => Some(Lead::Toggle),
                    _ => None,
                };
                if let Some(lead) = lead {
                    next_act = Some((*role, name.as_ref(), *order, lead));
                }
            }
            _ => {}
        }
    }
    (Vec::new(), Lead::Press)
}

fn step(page: &mut Page, record: &Record) -> Result<(), String> {
    match record {
        Record::Web(Web::Act { role, name, order, act }) => {
            let role_name = match role {
                Role::TextField => "textbox",
                Role::Check => "checkbox",
                _ => skein_fake_person::role_name(*role),
            };
            let order_text = if *order == 0 { String::new() } else { format!(" {order}") };
            let bound = format!("{role_name} {:?}{order_text} not found within 60 s", String::from_utf8_lossy(name));
            let node = page.wait(&find(*role, name, *order), Some(act), WITHIN).map_err(|why| failed(why, &bound))?;
            let act = match act {
                Act::Press => Act::Press,
                Act::Submit => Act::Submit,
                Act::Toggle => Act::Toggle,
                Act::Input { value } => Act::Input { value: value.clone() },
                Act::Drawing { .. } | Act::Near { .. } => return Err("not a journey's walk act".into()),
            };
            page.act(node, act).map_err(|why| failed(why, &bound))
        }
        Record::Web(Web::See(text)) => {
            let bound = format!("{:?} not seen within 60 s", String::from_utf8_lossy(text));
            page.wait(&Find::text(text), None, WITHIN).map(|_| ()).map_err(|why| failed(why, &bound))
        }
        Record::Web(Web::Reload(path)) => {
            page.reload(path, WITHIN).map_err(|why| failed(why, "the main region not drawn within 60 s"))
        }
        Record::Tui(Tui::Key(key)) => page.tui_key(key.clone()).map_err(|why| failed(why, "the TUI key failed")),
        Record::Tui(Tui::See(text)) => {
            let bound = format!("{:?} not seen in the TUI within 30 s", String::from_utf8_lossy(text));
            page.tui_wait(text, TUI_WITHIN).map_err(|why| failed(why, &bound))
        }
        Record::Tui(Tui::Stop) => {
            page.tui_stop();
            Ok(())
        }
        Record::Tui(Tui::Approve) => {
            page.tui_approve(TUI_WITHIN).map_err(|why| failed(why, "the TUI approval not completed within 30 s"))
        }
        _ => Err("not a journey's walk line".into()),
    }
}

fn failed(why: Failed, bound: &str) -> String {
    match why {
        Failed::Bound => bound.into(),
        Failed::World(words) => format!("the world failed: {}", String::from_utf8_lossy(&words)),
        Failed::Stopped => "the world stopped".into(),
        Failed::Turns => "the turn bound at one instant passed".into(),
        Failed::NoLink => "the service printed no sign-in link".into(),
        Failed::PageLoad { status: Some(status) } => format!("the service returned HTTP {status} for the page"),
        Failed::PageLoad { status: None } => "the page request failed".into(),
        Failed::Closed => "the TUI closed".into(),
        Failed::NotBrowsed => "the TUI asked to browse no address on the service within 30 s".into(),
        Failed::Asked(duty) => format!("the TUI asked to {duty} its terminal, which a world does not play"),
    }
}

#[cfg(test)]
mod tests;
