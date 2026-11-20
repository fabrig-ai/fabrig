//! A milestone's own reads (temper/docs/design/client/pages/milestone.md, 3).
use super::{Request, goal, milestones::MilestoneRead};
use crate::{Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Readable, Readout, Told, contract as c};
use skein_lib::{Queue, Wall};
use temper_client_domain_forge::parts;

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads the milestone and its line; paging belongs to the page")]
pub struct Page {
    pub milestone: MilestoneRead,
    pub line: Option<c::TaskLine>,
    line_handle: Handle,
    query: c::TaskQuery,
    number: u64,
    key: (Wall, Wall),
    ended: bool,
    line_at: Option<(Wall, Wall)>,
    searching: Option<(Wall, Wall)>,
    pages: u32,
    ended_group: bool,
}
impl Page {
    pub(super) fn new(task: &c::Task, next: &mut u64, out: &mut Queue<Request>) -> Self {
        let milestone = MilestoneRead::new(task.project, task.chip.task, next);
        let line_handle = allocate(next);
        let mut page = Self {
            milestone,
            line: None,
            line_handle,
            query: own_line_query(task),
            number: task.chip.task,
            key: (task.changed, task.last_activity),
            ended: task.chip.phase == c::Phase::Ended,
            line_at: None,
            searching: None,
            pages: 0,
            ended_group: false,
        };
        page.search(out);
        page
    }
    pub(super) fn changed(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        self.key = (task.changed, task.last_activity);
        self.ended = task.chip.phase == c::Phase::Ended;
        self.query = own_line_query(task);
        self.search(out);
    }
    fn search(&mut self, out: &mut Queue<Request>) {
        if self.searching.is_none() && self.line_at != Some(self.key) {
            self.searching = Some(self.key);
            self.pages = 0;
            self.ended_group = false;
            self.read_line(None, None, out);
        }
    }
    fn read_line(&mut self, live: Option<c::Cursor>, ended: Option<c::Cursor>, out: &mut Queue<Request>) {
        self.pages = self.pages.checked_add(1).expect("at most eight own-line pages");
        read(self.line_handle, c::Read::Tasks { query: self.query.clone(), live, ended }, out);
    }
    pub(super) fn told(&mut self, client: &Client, limits: &Limits, told: &Told, out: &mut Queue<Request>) -> bool {
        let handled = match told {
            Told::Page { handle, page } if *handle == self.line_handle => {
                self.line_page(page, out);
                true
            }
            Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Page { .. }
            | Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. } => self.milestone.told(told, client, limits, out),
        };
        if handled {
            self.configure(client, limits, out);
        }
        handled
    }
    pub(super) fn configure(&mut self, client: &Client, limits: &Limits, out: &mut Queue<Request>) {
        self.milestone.configure_at(self.key.0, self.ended, client, limits, out);
    }
    fn line_page(&mut self, page: &Readout, out: &mut Queue<Request>) {
        let Some(at) = self.searching else { return };
        if at == self.key
            && let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page
        {
            let ended = match &page.ended {
                c::EndedGroup::Rows { rows, .. } => rows.as_slice(),
                c::EndedGroup::None => &[],
            };
            let mut found = None;
            for line in page.live.iter().chain(ended) {
                if line.chip.task == self.number {
                    found = Some(line.clone());
                    break;
                }
            }
            if found.is_some() {
                self.line = found;
            } else {
                if self.pages < 8 {
                    if !self.ended_group
                        && let Some(after) = &page.live_after
                    {
                        self.read_line(Some(after.clone()), None, out);
                        return;
                    }
                    if let c::EndedGroup::Rows { after: Some(after), .. } = &page.ended {
                        self.ended_group = true;
                        self.read_line(None, Some(after.clone()), out);
                        return;
                    }
                }
                self.line = None;
            }
        }
        self.line_at = self.searching.take();
        self.search(out);
    }
    #[must_use]
    pub fn headline(&self, task: u64) -> Option<Box<[u8]>> {
        for line in &self.milestone.goals {
            if line.chip.task == task {
                return match parts::goal(line)?.report {
                    Some(report) => Some(report.headline),
                    None => None,
                };
            }
        }
        None
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        self.milestone.leave(out);
        out.push(Request::Client(jig_client::Event::Unwatch { handle: self.line_handle }));
    }
}
fn allocate(next: &mut u64) -> Handle {
    let handle = Handle { asker: Owner::Pages.asker(), number: *next };
    *next = next.checked_add(1).expect("page handles fit");
    handle
}
fn read(handle: Handle, read: c::Read, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }));
}

fn own_line_query(task: &c::Task) -> c::TaskQuery {
    c::TaskQuery { tracked: Some(task.priority.is_some()), ..goal::own_line_query(task) }
}
