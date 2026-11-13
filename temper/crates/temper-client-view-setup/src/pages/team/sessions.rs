//! The role's sessions after its wakes (client/pages/team.md, 1).
use super::{paragraph, text};
use crate::{
    look,
    words::{Word, Words},
};
use skein_lib::List;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::pages::team::Session;

pub(super) fn draw(b: &mut Builder, w: &Words, rows: &List<Session>) {
    b.open(Role::Region);
    b.style(look::TEAM_SESSIONS);
    paragraph(b, &w.say(Word::TeamSessions, &[]), look::TEAM_KICKER);
    b.open(Role::List);
    for session in rows {
        b.open(Role::Item);
        b.style(look::TEAM_SESSION);
        let length = if let Some(end) = session.end {
            w.time_span(end.as_nanos().saturating_sub(session.start.as_nanos()) / 60_000_000_000)
        } else {
            w.say(Word::TeamSessionRunning, &[])
        };
        paragraph(
            b,
            &w.say(Word::TeamSessionLine, &[Hole::Name(w.session_clock(session.start)), Hole::Name(length)]),
            look::TEAM_SESSION_TIME,
        );
        if let Some(words) = &session.words {
            text(b, Role::Paragraph, words);
        }
        paragraph(b, &w.say(Word::TeamCount, &[Hole::Number(session.spent)]), look::TEAM_SESSION_COST);
        b.close();
    }
    b.close();
    paragraph(b, &w.say(Word::TeamSessionClosing, &[]), look::TEAM_SESSION_CLOSING);
    b.close();
}
