//! Rendering reads use DOM and CSS parameters, not page evaluation (skein/docs/design/testing/browser-vocabulary.md, 1, and skein/docs/design/testing/browser-operations.md, 1).

use super::{
    Browser, OpKind, Operation, PendingKind, Step as BrowserStep, field_array, field_u64, protocol_detail, quad_rect,
};
use crate::boundary::{Element, Event, Node, Property, Refusal};
use crate::{
    Limits,
    params::Params,
    wire::decode::{Array, Value},
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Set, Stack, Token};

#[derive(Debug)]
pub(super) enum Read {
    Elements { selector: Option<Box<[u8]>>, tree: Option<List<Located>> },
    Style { node: Node, waiting_css: bool },
    Rect(Node),
}

#[derive(Debug)]
pub(super) struct Located {
    frontend: u64,
    element: Element,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    Tree,
    Select,
    StyleRoot,
    StyleNode,
    Style,
    Rect,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Css {
    Disabled,
    Enabling,
    Enabled,
}

impl Browser {
    pub(super) fn read_render(&mut self, env: &Env<Limits>, page: Token, op: Token, read: Read) {
        let until = env.now.saturating_add(env.limits.answer);
        if self.admit_op(env, page, op, OpKind::Render { read, until }) {
            self.start_render(env, op);
        }
    }

    pub(super) fn start_render(&mut self, env: &Env<Limits>, op: Token) {
        let Some(Operation { kind: OpKind::Render { read, until }, .. }) = self.op_mut(op) else { return };
        if env.now >= *until {
            self.finish_refused(op, Refusal::Timeout);
            return;
        }
        match read {
            Read::Elements { tree, .. } => {
                *tree = None;
                self.issue_step(env, op, b"DOM.getDocument", &Params::Tree, BrowserStep::Render(Step::Tree));
            }
            Read::Style { waiting_css, .. } => {
                *waiting_css = false;
                self.issue_step(env, op, b"DOM.getDocument", &Params::Root, BrowserStep::Render(Step::StyleRoot));
            }
            Read::Rect(node) => {
                let params = Params::Backend(*node);
                self.issue_step(env, op, b"DOM.getBoxModel", &params, BrowserStep::Render(Step::Rect));
            }
        }
    }

    pub(super) fn render_reply(&mut self, env: &Env<Limits>, op: Token, method: &[u8], step: Step, result: Value<'_>) {
        let Some(Operation { kind: OpKind::Render { until, .. }, .. }) = self.op(op) else { return };
        if env.now >= *until {
            self.finish_refused(op, Refusal::Timeout);
            return;
        }
        let answered = match step {
            Step::Tree => self.render_tree(env, op, result),
            Step::Select => self.selected_elements(env, op, result),
            Step::StyleRoot => self.style_root(env, op, result),
            Step::StyleNode => self.style_node(env, op, result),
            Step::Style => {
                if let Some(style) = properties(env, result) {
                    self.render_done(op, Event::Style { op, style });
                    Some(())
                } else {
                    None
                }
            }
            Step::Rect => {
                let rect = match super::opt_field(result.get(b"model"), b"border") {
                    Some(border) => match border.array() {
                        Some(points) => quad_rect(points),
                        None => None,
                    },
                    None => None,
                };
                if let Some(rect) = rect {
                    self.render_done(op, Event::Rect { op, rect: Some(rect) });
                    Some(())
                } else {
                    None
                }
            }
        };
        if answered.is_none() {
            self.finish_protocol(op, protocol_detail(method, b"malformed reply", env.limits.text));
        }
    }

    pub(super) fn render_done(&mut self, op: Token, event: Event) {
        if self.remove_op(op).is_some() {
            self.events.push_back(event);
        }
    }

    fn render_tree(&mut self, env: &Env<Limits>, op: Token, result: Value<'_>) -> Option<()> {
        let root = result.get(b"root")?;
        let node = field_u64(root, b"nodeId")?;
        let Operation { kind: OpKind::Render { read: Read::Elements { selector, tree }, .. }, .. } = self.op_mut(op)?
        else {
            return None;
        };
        if let Some(selector) = selector {
            // The selector reply identifies frontend ids, not their order or depth.
            // Keep just the element rows from this tree, at most one per message byte.
            let (located, _) = elements(root, env.limits.message, env.limits.text)?;
            *tree = Some(located);
            let selector = selector.clone();
            self.issue_step(
                env,
                op,
                b"DOM.querySelectorAll",
                &Params::Selector { node, selector: &selector },
                BrowserStep::Render(Step::Select),
            );
        } else {
            let (located, more) = elements(root, env.limits.elements, env.limits.text)?;
            self.elements_done(op, located, None, env.limits.elements, more);
        }
        Some(())
    }

    fn selected_elements(&mut self, env: &Env<Limits>, op: Token, result: Value<'_>) -> Option<()> {
        let mut ids = Set::with_capacity(env.limits.message);
        for id in field_array(result, b"nodeIds")? {
            let _inserted = ids.insert(id.u64()?).ok()?;
        }
        let Operation { kind: OpKind::Render { read: Read::Elements { tree, .. }, .. }, .. } = self.op_mut(op)? else {
            return None;
        };
        let located = tree.take()?;
        self.elements_done(op, located, Some(&ids), env.limits.elements, 0);
        Some(())
    }

    fn elements_done(&mut self, op: Token, located: List<Located>, ids: Option<&Set<u64>>, limit: u32, mut more: u32) {
        let mut elements = List::with_capacity(limit);
        for row in located.into_boxed() {
            if let Some(ids) = ids
                && !ids.contains(&row.frontend)
            {
                continue;
            }
            if elements.push(row.element).is_err() {
                more = more.saturating_add(1);
            }
        }
        self.render_done(op, Event::Elements { op, elements, more });
    }

    fn style_root(&mut self, env: &Env<Limits>, op: Token, result: Value<'_>) -> Option<()> {
        let _root = field_u64(result.get(b"root")?, b"nodeId")?;
        let Operation { page, kind: OpKind::Render { read: Read::Style { node, .. }, .. }, .. } = self.op(op)? else {
            return None;
        };
        let (page, node) = (*page, *node);
        match self.page(page)?.css {
            Css::Enabled => self.push_style_node(env, op, node),
            Css::Disabled | Css::Enabling => {
                let Operation { due, kind: OpKind::Render { read: Read::Style { waiting_css, .. }, until }, .. } =
                    self.op_mut(op)?
                else {
                    return None;
                };
                *waiting_css = true;
                *due = Some(*until);
                if let Some(state) = self.page_mut(page)
                    && let Css::Disabled = state.css
                {
                    state.css = Css::Enabling;
                    if !self.queue_page(env, page, b"CSS.enable", &Params::Empty, PendingKind::Css(page)) {
                        self.css_refused(env, page, Refusal::Limit, b"");
                    }
                }
            }
        }
        Some(())
    }

    fn push_style_node(&mut self, env: &Env<Limits>, op: Token, node: Node) {
        self.issue_step(
            env,
            op,
            b"DOM.pushNodesByBackendIdsToFrontend",
            &Params::PushBackend(node),
            BrowserStep::Render(Step::StyleNode),
        );
    }

    fn style_node(&mut self, env: &Env<Limits>, op: Token, result: Value<'_>) -> Option<()> {
        let node = field_array(result, b"nodeIds")?.next()?.u64()?;
        if node == 0 {
            self.finish_refused(op, Refusal::Gone);
        } else {
            self.issue_step(
                env,
                op,
                b"CSS.getComputedStyleForNode",
                &Params::Node(node),
                BrowserStep::Render(Step::Style),
            );
        }
        Some(())
    }

    pub(super) fn css_enabled(&mut self, env: &Env<Limits>, page: Token) {
        let Some(state) = self.page_mut(page) else { return };
        state.css = Css::Enabled;
        let waiting = self.css_waiters(page, env.limits.ops);
        for op in &waiting {
            let Some(Operation {
                due, kind: OpKind::Render { read: Read::Style { node, waiting_css }, until }, ..
            }) = self.op_mut(*op)
            else {
                continue;
            };
            if env.now >= *until {
                self.finish_refused(*op, Refusal::Timeout);
            } else if *due == Some(*until) {
                // Navigation may instead have scheduled the whole read to restart.
                let node = *node;
                *waiting_css = false;
                *due = None;
                self.push_style_node(env, *op, node);
            }
        }
    }

    pub(super) fn css_refused(&mut self, env: &Env<Limits>, page: Token, why: Refusal, message: &[u8]) {
        let Some(state) = self.page_mut(page) else { return };
        state.css = Css::Disabled;
        for op in &self.css_waiters(page, env.limits.ops) {
            if why == Refusal::Protocol {
                self.finish_protocol(*op, protocol_detail(b"CSS.enable", message, env.limits.text));
            } else {
                self.finish_refused(*op, why);
            }
        }
    }

    fn css_waiters(&self, page: Token, limit: u32) -> List<Token> {
        let mut waiting = List::with_capacity(limit);
        for op in self.ops.iter().flatten() {
            if op.page == page
                && let OpKind::Render { read: Read::Style { waiting_css: true, .. }, .. } = op.kind
            {
                waiting.push(op.token).expect("one waiter per admitted operation");
            }
        }
        waiting
    }
}

fn elements(root: Value<'_>, limit: u32, text: u32) -> Option<(List<Located>, u32)> {
    let mut listed = List::with_capacity(limit);
    let mut more = 0_u32;
    let mut path: Stack<Array<'_>> = Stack::with_capacity(crate::wire::decode::MAX_DEPTH);
    if let Some(children) = root.get(b"children") {
        path.push(children.array()?).ok()?;
    }
    let text = usize::try_from(text).expect("text fits usize");
    // Each visited node and each exhausted child array has a distinct starting byte.
    for _ in 0..root.raw().len() {
        let Some(children) = path.top_mut() else { return Some((listed, more)) };
        if let Some(node) = children.next() {
            if field_u64(node, b"nodeType")? == 1 {
                if listed.room() == 0 {
                    more = more.saturating_add(1);
                } else {
                    let element = Element {
                        node: field_u64(node, b"backendNodeId")?,
                        name: node.get(b"localName")?.decoded_text()?.into_prefix(text),
                        depth: path.len().saturating_sub(1),
                    };
                    let frontend = field_u64(node, b"nodeId")?;
                    listed.push(Located { frontend, element }).expect("room checked before decoding element");
                }
            }
            if let Some(children) = node.get(b"children") {
                path.push(children.array()?).ok()?;
            }
        } else {
            let _exhausted = path.pop();
        }
    }
    None
}

fn properties(env: &Env<Limits>, result: Value<'_>) -> Option<List<Property>> {
    let mut style = List::with_capacity(env.limits.message);
    let text = usize::try_from(env.limits.text).expect("text fits usize");
    for property in field_array(result, b"computedStyle")? {
        let name = property.get(b"name")?.decoded_text()?.into_prefix(text);
        let value = property.get(b"value")?.decoded_text()?.into_prefix(text);
        style.push(Property { name, value }).ok()?;
    }
    Some(style)
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
