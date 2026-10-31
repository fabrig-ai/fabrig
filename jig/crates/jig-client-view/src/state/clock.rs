//! Wall time is for the view; deadlines are monotonic. A minute's tick is
//! armed only while a work page shows wall times; client/domain/client.md, 13.
use crate::{
    Ui,
    state::{objects::Body, page::Page, ui::Timer},
};
use skein_lib::Duration;
/// Whether page policy shows times, and whether its one tick is armed.
#[derive(Debug)]
pub(crate) struct State {
    pub page_times: bool,
    pub armed: bool,
}
fn showing(domain: &Ui) -> bool {
    match domain.page {
        Page::Work { .. } => {}
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {
            return false;
        }
    }
    if domain.clock.page_times
        || !domain.turn_rows().rows.is_empty()
        || !domain.history_rows.rows.is_empty()
        || !domain.conversation_rows.rows.is_empty()
    {
        return true;
    }
    for (_, id) in &domain.objects.index {
        let object = domain.objects.slab.get(*id).expect("indexed object exists");
        if !object.sources.is_empty() {
            match &object.body {
                Some(
                    Body::Task(_)
                    | Body::OwnerProposal(_)
                    | Body::OwnerEscalation(_)
                    | Body::Question(_)
                    | Body::OwnerPersonTask(_)
                    | Body::Result(_)
                    | Body::Note { .. },
                ) => return true,
                Some(Body::Historical(_) | Body::Chip(_)) | None => {}
            }
        }
    }
    false
}
pub(crate) fn sync(domain: &mut Ui) {
    if showing(domain) {
        if !domain.clock.armed {
            domain
                .timers
                .arm(Timer::Minute, domain.now.saturating_add(Duration::from_secs(60)))
                .expect("one minute timer fits");
            domain.clock.armed = true;
        }
    } else {
        domain.timers.cancel(Timer::Minute);
        domain.clock.armed = false;
    }
}
pub(crate) fn minute(domain: &mut Ui) {
    domain.clock.armed = false;
    if showing(domain) {
        domain.changed();
    }
    sync(domain);
}

/// ISO calendar minute in UTC, checked against the wall clock's representable range.
#[must_use]
pub(crate) fn deadline(value: &[u8]) -> Option<skein_lib::Wall> {
    if value.len() != 16 || *value.get(10)? != b' ' || *value.get(13)? != b':' {
        return None;
    }
    let midnight = skein_ui::date::wall(skein_ui::date::parse(value.get(..10)?)?)?;
    let hour = skein_ui::number(value.get(11..13)?)?;
    let minute = skein_ui::number(value.get(14..16)?)?;
    if hour > 23 || minute > 59 {
        return None;
    }
    let seconds = midnight.as_secs().checked_add(hour.checked_mul(3600)?)?.checked_add(minute.checked_mul(60)?)?;
    Some(skein_lib::Wall::from_nanos(seconds.checked_mul(1_000_000_000)?))
}
