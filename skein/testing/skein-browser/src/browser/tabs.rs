//! Other tabs read without attaching or evaluating (skein/docs/design/browser.md, 6).
use super::{Browser, field_array, field_text, protocol_detail};
use crate::boundary::{Event, Refusal, Tab};
use crate::{Limits, wire::decode::Value};
use skein_lib::{Env, List, Token};

impl Browser {
    pub(super) fn tabs_reply(&mut self, env: &Env<Limits>, token: Token, result: Value<'_>) {
        let Some(op) = self.op(token) else { return };
        let Some(page) = self.page(op.page) else { return };
        let Some(person) = self.person(page.person) else { return };
        let Some(context) = &person.context else { return };
        let answer = self.read_tabs(&env.limits, result, context);
        match answer {
            Ok((tabs, more)) => {
                if self.remove_op(token).is_some() {
                    self.events.push_back(Event::Tabs { op: token, tabs, more });
                }
            }
            Err(Refusal::Protocol) => {
                self.finish_protocol(token, protocol_detail(b"Target.getTargets", b"malformed reply", env.limits.text));
            }
            Err(
                why @ (Refusal::Gone
                | Refusal::Hidden
                | Refusal::Covered
                | Refusal::Disabled
                | Refusal::Crashed
                | Refusal::Limit
                | Refusal::Timeout),
            ) => self.finish_refused(token, why),
        }
    }

    fn read_tabs(&self, limits: &Limits, result: Value<'_>, context: &[u8]) -> Result<(List<Tab>, u32), Refusal> {
        let targets = field_array(result, b"targetInfos").ok_or(Refusal::Protocol)?;
        let mut tabs = List::with_capacity(limits.matches);
        let mut more = 0_u32;
        let text = usize::try_from(limits.text).expect("text fits usize");
        for target in targets {
            let kind = field_text(target, b"type").ok_or(Refusal::Protocol)?;
            if kind.as_ref() != b"page" {
                continue;
            }
            let target_context = field_text(target, b"browserContextId");
            if target_context.as_deref() != Some(context) {
                continue;
            }
            let id = field_text(target, b"targetId").ok_or(Refusal::Protocol)?;
            if self.owns_target(&id) {
                continue;
            }
            let url = field_text(target, b"url").ok_or(Refusal::Protocol)?;
            let title = field_text(target, b"title").ok_or(Refusal::Protocol)?;
            let opener = match target.get(b"canAccessOpener") {
                Some(value) => value.bool().ok_or(Refusal::Protocol)?,
                None => false,
            };
            if url.len() > text || title.len() > text {
                return Err(Refusal::Limit);
            }
            if tabs.len() < limits.matches {
                tabs.push(Tab { url, title, opener }).expect("matches checked");
            } else {
                more = more.saturating_add(1);
            }
        }
        Ok((tabs, more))
    }

    fn owns_target(&self, target: &[u8]) -> bool {
        for page in self.pages.iter().flatten() {
            if page.target.as_deref() == Some(target) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
#[path = "tabs_tests.rs"]
mod tests;
