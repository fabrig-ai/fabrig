//! A project's settings page (temper/docs/design/draft/playground-reset.md, 5).
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use skein_lib::Decimal;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::Domain;

pub(crate) fn settings(b: &mut Builder, root: &Domain, w: &Words, _bindings: &mut Bindings, project: u32) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    let mut listed = None;
    if let Some(me) = root.pages().me(root.client()) {
        for row in &me.projects {
            if row.project == project {
                listed = Some(row);
                break;
            }
        }
    }
    let (title, description) = if let Some(row) = listed {
        let number = Decimal::of(u64::from(project));
        (
            w.say(Word::ProjectSettingsTitle, &[Hole::Name(skein_lib::bytes::copy_of(row.name.bytes()))]),
            w.say(Word::ProjectSettingsNumber, &[Hole::Name(skein_lib::bytes::copy_of(number.as_bytes()))]),
        )
    } else {
        (w.say(Word::ProjectSettings, &[]), w.say(Word::ProjectGone, &[]))
    };
    crate::cards::text(b, Role::Heading { level: 1 }, &title);
    crate::cards::text(b, Role::Paragraph, &description);
    b.close();
}
