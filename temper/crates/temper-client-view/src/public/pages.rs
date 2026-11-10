//! Each anonymous page draws its public records (temper/docs/design/client/web/public.md, 3).
use super::{Draw, heading};
use crate::words::Word;
use jig_client_protocol_wire::public_v1 as j;
use skein_lib::Duration;
use skein_ui::{Builder, Role};
use temper_client_domain::{address::Public, public::Shown};
use temper_client_protocol_wire::public_v1 as f;
impl Draw<'_> {
    pub(super) fn main(&mut self, b: &mut Builder, shown: Shown<'_>) {
        match shown {
            Shown::Reading => {
                heading(b, &self.words.say(Word::PublicProjects, &[]));
                self.paragraph(b, Word::PublicReading);
            }
            Shown::Unavailable => {
                heading(b, &self.words.say(Word::PublicUnavailable, &[]));
            }
            Shown::Unknown => {
                heading(b, &self.words.say(Word::PublicUnknown, &[]));
                self.link(b, &self.words.say(Word::ReadPublic, &[]), Public::Projects);
            }
            Shown::Projects { projects, as_of } => self.projects(b, projects, as_of),
            Shown::Board { board, landings, unavailable, as_of, companion_as_of, .. } => {
                self.board(b, board, landings, unavailable, as_of, companion_as_of);
            }
            Shown::Task { card, change, unavailable, as_of, companion_as_of, .. } => {
                self.card(b, card, change, unavailable, as_of, companion_as_of);
            }
            Shown::Landings { application, landings, as_of } => {
                heading(b, &self.words.say(Word::PublicLandings, &[]));
                self.landings(b, application, landings);
                self.as_of(b, as_of);
            }
            Shown::Landing { application, landing, as_of } => {
                heading(b, &self.words.say(Word::PublicLandings, &[]));
                self.landing(b, application, landing);
                self.as_of(b, as_of);
            }
        }
    }
    fn projects(&mut self, b: &mut Builder, projects: &j::PublicProjects, as_of: Option<Duration>) {
        heading(b, &self.words.say(Word::PublicProjects, &[]));
        if projects.projects().is_empty() {
            self.paragraph(b, Word::PublicNoProjects);
        }
        b.open(Role::List);
        b.style(crate::look::PUBLIC_ROWS);
        for project in projects.projects() {
            b.open(Role::Item);
            b.style(crate::look::PUBLIC_ROW);
            b.open(Role::Paragraph);
            b.style(crate::look::PUBLIC_LINE);
            self.link(b, project.name(), Public::Board { project: project.project() });
            b.close();
            b.close();
        }
        b.close();
        self.as_of(b, as_of);
    }
    fn board(
        &mut self,
        b: &mut Builder,
        board: &j::PublicBoard,
        landings: Option<(&j::PublicApplication, &f::PublicLandings)>,
        unavailable: bool,
        as_of: Option<Duration>,
        companion_as_of: Option<Duration>,
    ) {
        b.open(Role::Region);
        b.style(crate::look::PUBLIC_INTRO);
        heading(b, board.name());
        b.open(Role::Paragraph);
        b.style(crate::look::PUBLIC_LEDE);
        self.text(b, Word::PublicBoardLede);
        b.close();
        b.open(Role::Paragraph);
        b.style(crate::look::PUBLIC_LEDE);
        self.text(b, Word::PublicBoardHeld);
        b.close();
        b.close();
        if board.live().is_empty() && board.ended().is_empty() {
            self.paragraph(b, Word::PublicNoGoals);
        }
        b.open(Role::List);
        b.style(crate::look::PUBLIC_ROWS);
        for goal in board.live().iter().chain(board.ended()) {
            b.open(Role::Item);
            self.row(b, goal.line(), false);
            b.open(Role::List);
            b.style(crate::look::PUBLIC_ROWS);
            for line in goal.tree() {
                b.open(Role::Item);
                self.row(b, line, false);
                b.close();
            }
            b.close();
            self.folded(b, goal.folded());
            b.close();
        }
        b.close();
        self.folded(b, board.live_folded());
        self.as_of(b, as_of);
        if let Some((application, landings)) = landings
            && (!landings.queues().is_empty() || !landings.landings().is_empty())
        {
            b.open(Role::Region);
            b.style(crate::look::PUBLIC_SECTION);
            b.open(Role::Heading { level: 2 });
            self.link(b, &self.words.say(Word::PublicLandings, &[]), Public::Landings { project: board.project() });
            b.close();
            self.queues(b, application, landings);
            if landings.landings().is_empty() {
                self.paragraph(b, Word::PublicNoLandings);
            }
            self.as_of(b, companion_as_of);
            b.close();
        }
        if unavailable {
            self.paragraph(b, Word::PublicUnavailable);
        }
    }
    fn card(
        &mut self,
        b: &mut Builder,
        card: &j::PublicCard,
        change: Option<(&j::PublicApplication, &f::PublicChange)>,
        unavailable: bool,
        as_of: Option<Duration>,
        companion_as_of: Option<Duration>,
    ) {
        heading(b, &self.identity(card.line().task(), card.line().goal()));
        b.open(Role::List);
        b.style(crate::look::PUBLIC_ROWS);
        b.open(Role::Item);
        self.row(b, card.line(), true);
        b.close();
        b.close();
        b.open(Role::Navigation);
        for task in card.lineage() {
            self.task_link(b, *task, false);
        }
        b.close();
        b.open(Role::List);
        b.style(crate::look::PUBLIC_ROWS);
        for line in card.delegates() {
            b.open(Role::Item);
            self.row(b, line, false);
            b.close();
        }
        b.close();
        self.folded(b, card.folded());
        self.as_of(b, as_of);
        if let Some((application, change)) = change {
            self.change(b, application, change);
            self.as_of(b, companion_as_of);
        }
        if unavailable {
            self.paragraph(b, Word::PublicUnavailable);
        }
    }
}
