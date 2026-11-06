//! Poll tab reads until the script's deadline (skein/docs/design/ui-web.md, 14.2).
use super::TabSeen;
use crate::BrowserFace;
use skein_browser::boundary::Tab;
use skein_lib::{List, Time};
pub(super) struct Awaiting {
    expected: Box<[TabSeen]>,
    until: Time,
    again: Time,
    last: Option<(List<Tab>, u32)>,
}
impl Awaiting {
    pub(super) fn new(expected: Box<[TabSeen]>, until: Time) -> Self {
        Self { expected, until, again: Time::ZERO, last: None }
    }
    fn matches(&self, tabs: &List<Tab>, more: u32) -> bool {
        more == 0
            && usize::try_from(tabs.len()).expect("tab count fits") == self.expected.len()
            && self.expected.iter().zip(tabs.iter()).all(|(expected, tab)| {
                expected.title.as_deref().is_none_or(|title| title == tab.title.as_ref())
                    && expected
                        .address
                        .as_deref()
                        .is_none_or(|address| skein_lib::bytes::find(&tab.url, address).is_some())
                    && expected.opener.is_none_or(|opener| opener == tab.opener)
            })
    }
    pub(super) fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> bool {
        if let Some(answer) = face.take_tabs() {
            let met = self.matches(&answer.0, answer.1);
            self.last = Some(answer);
            self.again = now.saturating_add(skein_browser::Limits::default().poll);
            if met && now <= self.until {
                return true;
            }
        }
        if now >= self.until {
            let seen = self.last.as_ref().map_or_else(
                || "no reply".into(),
                |(tabs, more)| {
                    let tabs: Vec<_> = tabs
                        .iter()
                        .map(|tab| {
                            format!(
                                "title {:?}, address {:?}, opener {}",
                                String::from_utf8_lossy(&tab.title),
                                String::from_utf8_lossy(&tab.url),
                                tab.opener
                            )
                        })
                        .collect();
                    format!("[{}], {more} more", tabs.join("; "))
                },
            );
            face.fail(format!("tabs {:?} not met within its bound; saw {seen}", self.expected));
            return true;
        }
        if now >= self.again && !face.pending() {
            face.tabs();
        }
        false
    }
}
