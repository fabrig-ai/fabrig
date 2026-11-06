//! Selector reads use DOM parameters only, with no page evaluation.
use super::{Browser, OpKind, Operation, Step, field_array, field_u64, protocol_detail};
use crate::boundary::{Event, Read, Refusal};
use crate::{Limits, params::Params, wire::decode::Value};
use alloc::boxed::Box;
use skein_lib::{Env, Token};

impl Browser {
    pub(super) fn start_attribute(&mut self, env: &Env<Limits>, token: Token) {
        self.issue_step(env, token, b"DOM.getDocument", &Params::Root, Step::AttributeRoot);
    }
    pub(super) fn attribute_reply(
        &mut self,
        env: &Env<Limits>,
        token: Token,
        method: &[u8],
        step: Step,
        result: Value<'_>,
    ) {
        match step {
            Step::AttributeRoot => {
                let node = match result.get(b"root") {
                    Some(root) => field_u64(root, b"nodeId"),
                    None => None,
                };
                let Some(node) = node else {
                    self.finish_protocol(token, protocol_detail(method, b"malformed reply", env.limits.text));
                    return;
                };
                let Some((selector, _)) = self.attribute_fields(token) else { return };
                let selector = Box::from(selector);
                self.issue_step(
                    env,
                    token,
                    b"DOM.querySelector",
                    &Params::Selector { node, selector: &selector },
                    Step::AttributeSelector,
                );
            }
            Step::AttributeSelector => {
                let Some(node) = field_u64(result, b"nodeId") else {
                    self.finish_protocol(token, protocol_detail(method, b"malformed reply", env.limits.text));
                    return;
                };
                if node == 0 {
                    self.attribute_done(token, Read::NoElement);
                } else {
                    self.issue_step(env, token, b"DOM.getAttributes", &Params::Node(node), Step::AttributeValue);
                }
            }
            Step::AttributeValue => self.attribute_value(env, token, result),
            Step::History
            | Step::Go
            | Step::Query
            | Step::Box
            | Step::Scroll
            | Step::Accessibility
            | Step::Quads
            | Step::Metrics
            | Step::Hit
            | Step::Describe
            | Step::MouseMove
            | Step::MouseDown
            | Step::MouseUp
            | Step::TypeNode { .. }
            | Step::Focus
            | Step::Insert
            | Step::KeyDown
            | Step::KeyUp
            | Step::Character { .. }
            | Step::Tabs
            | Step::Address
            | Step::Snapshot
            | Step::Screenshot => unreachable!("only attribute steps reach the attribute reader"),
        }
    }
    fn attribute_value(&mut self, env: &Env<Limits>, token: Token, result: Value<'_>) {
        let method = b"DOM.getAttributes";
        let Some(mut attributes) = field_array(result, b"attributes") else {
            self.finish_protocol(token, protocol_detail(method, b"malformed reply", env.limits.text));
            return;
        };
        let Some((_, name)) = self.attribute_fields(token) else { return };
        let name = Box::from(name);
        for _ in 0..env.limits.message {
            let Some(key) = attributes.next() else {
                self.attribute_done(token, Read::NoAttribute);
                return;
            };
            let Some(value) = attributes.next() else {
                self.finish_protocol(token, protocol_detail(method, b"malformed reply", env.limits.text));
                return;
            };
            let Some(key) = key.text() else {
                self.finish_protocol(token, protocol_detail(method, b"malformed reply", env.limits.text));
                return;
            };
            if key == name {
                let Some(value) = value.text() else {
                    self.finish_protocol(token, protocol_detail(method, b"malformed reply", env.limits.text));
                    return;
                };
                if value.len() > usize::try_from(env.limits.text).expect("text fits usize") {
                    self.finish_refused(token, Refusal::Limit);
                } else {
                    self.attribute_done(token, Read::Value(value));
                }
                return;
            }
        }
        self.finish_refused(token, Refusal::Limit);
    }
    fn attribute_fields(&self, token: Token) -> Option<(&[u8], &[u8])> {
        match self.op(token) {
            Some(Operation { kind, .. }) => match kind {
                OpKind::Attribute { selector, name, .. } => Some((selector, name)),
                OpKind::Go(_)
                | OpKind::Find { .. }
                | OpKind::Press { .. }
                | OpKind::Type { .. }
                | OpKind::Key { .. }
                | OpKind::Tabs
                | OpKind::Address
                | OpKind::Snapshot
                | OpKind::Screenshot => None,
            },
            None => None,
        }
    }
    fn attribute_done(&mut self, token: Token, read: Read) {
        if self.remove_op(token).is_some() {
            self.events.push_back(Event::Attribute { op: token, read });
        }
    }
}

#[cfg(test)]
#[path = "attribute_tests.rs"]
mod tests;
