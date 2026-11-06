//! Patches applied in place, with the host deciding each document operation.
//! Log measurements and view keeping follow skein/docs/design/ui-web.md, 7.

use std::collections::BTreeMap;

use skein_lib::{List, usize_of};
use skein_ui::{
    Act, Limits, LogEdge, Made, Modifier, NodeId, Patch, Role, State,
    scene::{Point, Scene},
};

mod components;
#[cfg(any(test, target_arch = "wasm32"))]
mod fold;
#[cfg(any(test, target_arch = "wasm32"))]
mod measure;
#[cfg(target_arch = "wasm32")]
pub mod web;

use crate::{
    Look,
    dom::{self, Child, Element},
    mapping::{Primary, Words},
    svg,
};

/// The platform half replays these operations in order.
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    target_pointer_width = "64",
    expect(
        clippy::large_enum_variant,
        reason = "component operations carry a whole node without a separate allocation (ui-web.md, 6.1)"
    )
)]
pub enum Op {
    Insert { node: NodeId, parent: NodeId, before: Option<NodeId>, element: Element },
    Remove { node: NodeId },
    Move { node: NodeId, parent: NodeId, before: Option<NodeId> },
    Replace { node: NodeId, element: Element },
    Attributes { node: NodeId, element: Element },
    Text { node: NodeId, element: Element },
    Value { node: NodeId, value: Box<[u8]> },
    Choice { node: NodeId, checked: bool },
    Href { node: NodeId, path: Box<[u8]> },
    Focus { node: NodeId },
    ScrollEnd { node: NodeId },
    HoldView { node: NodeId },
    PutView { node: NodeId },
    Measure { node: NodeId },
    ScrollScreen { node: NodeId, down: bool },
    Component { node: NodeId, component: usize, made: Made },
}

struct Held {
    made: Made,
    parent: NodeId,
    children: Vec<NodeId>,
    element: Element,
    size: Option<(u16, u16)>,
    drawn: Option<Element>,
    href: Option<Box<[u8]>>,
    follow_log: bool,
    end_requested: bool,
    measured: Option<(i32, i32, i32)>,
    near_head: Option<(i32, i32)>,
    near_foot: Option<(i32, i32)>,
}

/// The semantic nodes and their document model, sharing the view's limits.
pub struct Page<'a> {
    look: Look<'a>,
    prefix: &'a [u8],
    primary: Primary,
    words: Words<'a>,
    nodes: BTreeMap<NodeId, Held>,
    roots: Vec<NodeId>,
    links: Vec<NodeId>,
    focused: Option<NodeId>,
    scene: Scene,
    limit: u32,
    act_bytes: u32,
    menu: Option<NodeId>,
    composition_end: Option<(NodeId, Box<[u8]>)>,
}

impl<'a> Page<'a> {
    #[must_use]
    pub fn new(look: Look<'a>, limits: &Limits, prefix: &'a [u8], primary: Primary) -> Self {
        Self {
            look,
            prefix,
            primary,
            words: Words::ENGLISH,
            nodes: BTreeMap::new(),
            roots: Vec::new(),
            links: Vec::new(),
            focused: None,
            scene: Scene::new(limits),
            limit: limits.nodes,
            act_bytes: limits.drawing_bytes,
            menu: None,
            composition_end: None,
        }
    }

    /// Set the page part's words before its first patch.
    pub fn words(&mut self, words: Words<'a>) {
        assert!(self.nodes.is_empty(), "the page's words come before its first patch");
        self.words = words;
    }

    #[expect(
        clippy::too_many_lines,
        reason = "patch ordering keeps log state changes before the final measurements (ui-web.md, 7)"
    )]
    pub fn apply(&mut self, patches: &[Patch], ops: &mut Vec<Op>) {
        if !patches.is_empty() {
            for (node, held) in &self.nodes {
                if held.made.attribute.role == Role::Log && !held.follow_log {
                    ops.push(Op::HoldView { node: *node });
                }
            }
        }
        let mut focus = None;
        for patch in patches {
            match patch {
                Patch::Insert { nodes, .. } => {
                    for made in nodes {
                        if made.attribute.focus.is_some() {
                            focus = Some(made.id);
                        }
                    }
                }
                Patch::Attribute { node, attribute } => {
                    if attribute.focus.is_some() && attribute.focus != self.held(*node).made.attribute.focus {
                        focus = Some(*node);
                    }
                }
                Patch::Remove { .. } | Patch::Move { .. } | Patch::Text { .. } | Patch::Value { .. } => {}
            }
            match patch {
                Patch::Insert { parent, before, nodes } => self.insert(*parent, *before, nodes, ops),
                Patch::Remove { node } => self.remove(*node, ops),
                Patch::Move { node, parent, before } => {
                    let old_parent = self.held(*node).parent;
                    self.detach(*node, old_parent);
                    self.attach(*node, *parent, *before);
                    self.nodes.get_mut(node).expect("held node").parent = *parent;
                    ops.push(Op::Move { node: *node, parent: *parent, before: *before });
                }
                Patch::Text { node, text } => {
                    self.held(*node);
                    self.nodes.get_mut(node).expect("held node").made.text.clone_from(text);
                    self.remap(*node);
                    ops.push(Op::Text { node: *node, element: self.own_element(*node) });
                }
                Patch::Attribute { node, attribute } => {
                    let old_tag = semantic(&self.held(*node).element).tag.clone();
                    let held = self.nodes.get_mut(node).expect("held node");
                    if (attribute.role == Role::Log || (attribute.role == Role::List && attribute.bound))
                        && held.made.attribute.states.as_slice().contains(&State::Busy)
                        && !attribute.states.as_slice().contains(&State::Busy)
                    {
                        held.near_head = None;
                        held.near_foot = None;
                    }
                    if attribute.role == Role::Log {
                        if attribute.end_request.is_some() && attribute.end_request != held.made.attribute.end_request {
                            held.end_requested = true;
                            held.follow_log = true;
                        } else if attribute.states.as_slice().contains(&State::Continues) && !held.end_requested {
                            held.follow_log = false;
                        }
                    }
                    held.made.attribute = attribute.as_ref().clone();
                    self.remap(*node);
                    self.draw(*node);
                    let element = self.own_element(*node);
                    if semantic(&element).tag == old_tag {
                        ops.push(Op::Attributes { node: *node, element });
                    } else {
                        ops.push(Op::Replace { node: *node, element });
                    }
                }
                Patch::Value { node, value } => {
                    self.held(*node);
                    self.nodes.get_mut(node).expect("held node").made.value.clone_from(value);
                    self.remap(*node);
                    let value = value.as_ref().map_or_else(|| Vec::new().into_boxed_slice(), |v| v.text.clone());
                    ops.push(Op::Value { node: *node, value });
                }
            }
            match patch {
                Patch::Attribute { node, .. }
                | Patch::Text { node, .. }
                | Patch::Value { node, .. }
                | Patch::Move { node, .. } => self.component_op(*node, ops),
                Patch::Insert { .. } | Patch::Remove { .. } => {}
            }
        }
        self.collection_sizes(ops);
        self.links.clear();
        let mut order = Vec::new();
        for root in &self.roots {
            self.preorder(*root, &mut order);
        }
        for node in order {
            let a = &self.held(node).made.attribute;
            if a.linked && a.external.is_none() {
                self.links.push(node);
            }
        }
        if let Some(node) = focus.filter(|node| self.nodes.contains_key(node)) {
            self.focused = Some(node);
            ops.push(Op::Focus { node });
        }
        if !patches.is_empty() {
            self.finish_logs(ops);
        }
    }

    fn finish_logs(&self, ops: &mut Vec<Op>) {
        for (node, held) in &self.nodes {
            if held.made.attribute.role == Role::Log {
                if held.follow_log {
                    ops.push(Op::ScrollEnd { node: *node });
                } else {
                    ops.push(Op::PutView { node: *node });
                }
            }
        }
        for (node, held) in &self.nodes {
            if (held.made.attribute.role == Role::Log || held.made.attribute.role == Role::List)
                && held.made.attribute.bound
            {
                ops.push(Op::Measure { node: *node });
            }
        }
    }

    /// The mount's focus listener reports the nearest semantic node.
    pub fn focused(&mut self, node: NodeId) {
        self.held(node);
        self.focused = Some(node);
    }

    pub fn resized(&mut self, node: NodeId, width: u16, height: u16, ops: &mut Vec<Op>) {
        self.held(node);
        let held = self.nodes.get_mut(&node).expect("held node");
        if held.made.attribute.drawing.is_none() {
            return;
        }
        if held.size == Some((width, height)) {
            return;
        }
        held.size = Some((width, height));
        self.draw(node);
        ops.push(Op::Attributes { node, element: self.own_element(node) });
    }

    #[must_use]
    pub fn html(&self) -> String {
        let mut outer = Element::new("");
        for root in &self.roots {
            outer.append(self.render(*root));
        }
        dom::print(&outer)
    }

    #[must_use]
    pub fn links(&self) -> &[NodeId] {
        &self.links
    }

    pub fn set_href(&mut self, node: NodeId, path: &[u8], ops: &mut Vec<Op>) {
        if self.held(node).href.as_deref() == Some(path) {
            return;
        }
        self.nodes.get_mut(&node).expect("held node").href = Some(path.into());
        self.remap(node);
        ops.push(Op::Href { node, path: path.into() });
    }

    fn collection_sizes(&mut self, ops: &mut Vec<Op>) {
        let items: Vec<_> = self
            .nodes
            .iter()
            .filter_map(|(node, held)| held.made.attribute.role.is_list_item().then_some(*node))
            .collect();
        for node in items {
            let mut parent = self.held(node).parent;
            let mut count = None;
            while parent != NodeId(0) {
                let held = self.held(parent);
                if self.held(node).made.attribute.role == Role::TreeItem && held.made.attribute.role == Role::TreeItem {
                    break;
                }
                if held.made.attribute.role.is_list() {
                    count = held.made.attribute.count.map(|count| count.to_string());
                    break;
                }
                parent = held.parent;
            }
            let held = self.nodes.get_mut(&node).expect("held item");
            let element = semantic_mut(&mut held.element);
            if element.attribute("aria-setsize") == count.as_deref() {
                continue;
            }
            if let Some(count) = count {
                element.set("aria-setsize", &count);
            } else {
                element.remove("aria-setsize");
            }
            ops.push(Op::Attributes { node, element: self.own_element(node) });
        }
    }

    fn held(&self, node: NodeId) -> &Held {
        self.nodes.get(&node).expect("patch names a held node")
    }

    fn siblings(&mut self, parent: NodeId) -> &mut Vec<NodeId> {
        if parent == NodeId(0) {
            &mut self.roots
        } else {
            self.held(parent);
            &mut self.nodes.get_mut(&parent).expect("held parent").children
        }
    }

    fn attach(&mut self, node: NodeId, parent: NodeId, before: Option<NodeId>) {
        let siblings = self.siblings(parent);
        let index = before.map_or(siblings.len(), |before| {
            siblings.iter().position(|sibling| *sibling == before).expect("patch names a held sibling")
        });
        siblings.insert(index, node);
    }

    fn detach(&mut self, node: NodeId, parent: NodeId) {
        let siblings = self.siblings(parent);
        let index = siblings.iter().position(|sibling| *sibling == node).expect("held child has its parent");
        siblings.remove(index);
    }

    fn insert(&mut self, parent: NodeId, before: Option<NodeId>, nodes: &[Made], ops: &mut Vec<Op>) {
        assert!(!nodes.is_empty(), "insert holds a subtree");
        let mut open = Vec::new();
        for (index, made) in nodes.iter().enumerate() {
            assert!(self.nodes.len() < usize_of(self.limit), "page fits the view's node limit");
            while open.last().is_some_and(|(_, end)| index >= *end) {
                open.pop();
            }
            let node_parent = open.last().map_or(parent, |(node, _)| *node);
            let element = components::element(made, &self.look, self.prefix, self.primary, &self.words);
            let held = Held {
                made: made.clone(),
                parent: node_parent,
                children: Vec::new(),
                element,
                size: None,
                drawn: None,
                href: None,
                follow_log: !made.attribute.states.as_slice().contains(&State::Continues),
                end_requested: false,
                measured: None,
                near_head: None,
                near_foot: None,
            };
            assert!(self.nodes.insert(made.id, held).is_none(), "inserted node has a fresh identity");
            self.attach(made.id, node_parent, if index == 0 { before } else { None });
            let end = index + usize_of(made.size);
            assert!(made.size > 0 && end <= nodes.len(), "insert subtree is contained");
            open.push((made.id, end));
        }
        let node = nodes[0].id;
        ops.push(Op::Insert { node, parent, before, element: self.render(node) });
        for made in nodes {
            self.component_op(made.id, ops);
        }
    }

    fn preorder(&self, node: NodeId, out: &mut Vec<NodeId>) {
        out.push(node);
        for child in &self.held(node).children {
            self.preorder(*child, out);
        }
    }

    fn remove(&mut self, node: NodeId, ops: &mut Vec<Op>) {
        let parent = self.held(node).parent;
        let mut removed = Vec::new();
        self.preorder(node, &mut removed);
        let took_focus = self.focused.is_some_and(|focus| removed.contains(&focus));
        let mut ancestor = parent;
        let mut focus = None;
        if took_focus {
            while ancestor != NodeId(0) {
                let held = self.held(ancestor);
                if held.made.attribute.role.is_focusable() {
                    focus = Some(ancestor);
                    break;
                }
                ancestor = held.parent;
            }
        }
        self.detach(node, parent);
        if self.menu.is_some_and(|menu| removed.contains(&menu)) {
            self.menu = None;
        }
        for node in removed {
            self.nodes.remove(&node);
        }
        ops.push(Op::Remove { node });
        if took_focus {
            if focus.is_none() {
                let mut order = Vec::new();
                for root in &self.roots {
                    self.preorder(*root, &mut order);
                }
                focus = order.into_iter().find(|node| self.held(*node).made.attribute.role == Role::Main);
            }
            self.focused = focus;
            if let Some(node) = focus {
                ops.push(Op::Focus { node });
            }
        }
    }

    fn remap(&mut self, node: NodeId) {
        let held = self.nodes.get_mut(&node).expect("held node");
        held.element = components::element(&held.made, &self.look, self.prefix, self.primary, &self.words);
        if held.made.attribute.role == Role::Menu {
            menu_element(&mut held.element, self.menu == Some(node));
        }
        if held.made.attribute.linked
            && held.made.attribute.external.is_none()
            && let Some(path) = &held.href
        {
            semantic_mut(&mut held.element).set("href", &String::from_utf8_lossy(path));
        }
    }

    fn draw(&mut self, node: NodeId) {
        let held = self.nodes.get_mut(&node).expect("held node");
        held.drawn = None;
        if components::index(&held.made, &self.look).is_some() {
            return;
        }
        if let Some(drawing) = &held.made.attribute.drawing
            && let Some((width, height)) = held.size
        {
            self.scene.clear();
            if (self.look.draw)(drawing.kind, &drawing.data, width, height, &mut self.scene) && !self.scene.refused() {
                held.drawn = Some(svg::draw(&self.scene, self.look.names, width, height));
            }
        }
    }

    fn own_element(&self, node: NodeId) -> Element {
        let held = self.held(node);
        let mut element = held.element.clone();
        if held.made.attribute.role == Role::Log && !held.follow_log {
            semantic_mut(&mut element).set("data-log-away", "");
        }
        if let Some(svg) = &held.drawn {
            let core = semantic_mut(&mut element);
            core.set("data-drawn", "");
            core.children.insert(0, Child::Element(svg.clone()));
        }
        element
    }

    fn render(&self, node: NodeId) -> Element {
        let mut element = self.own_element(node);
        let core = semantic_mut(&mut element);
        let destination = if core.tag == "pre" {
            match core.children.first_mut().expect("code has a code element") {
                Child::Element(contents) => contents,
                Child::Text(_) => unreachable!("code is an element"),
            }
        } else {
            core
        };
        let description = if self.held(node).made.attribute.role.describes_inside()
            && self.held(node).made.attribute.description.is_some()
        {
            destination.children.pop()
        } else {
            None
        };
        let state = if self.held(node).made.attribute.states.as_slice().contains(&State::Completed) {
            destination.children.pop()
        } else {
            None
        };
        let mut tree_group = None;
        let mut option_descriptions = Vec::new();
        for child in &self.held(node).children {
            let rendered = self.render(*child);
            if self.held(node).made.attribute.role == Role::TreeItem {
                if self.held(*child).made.attribute.role == Role::TreeItem {
                    let group = tree_group.get_or_insert_with(|| {
                        let mut group = Element::new("div");
                        group.set("role", "group");
                        group
                    });
                    group.append(rendered);
                    continue;
                }
                if let Some(group) = tree_group.take() {
                    destination.append(group);
                }
            }
            if self.held(node).made.attribute.role == Role::Select && rendered.tag.is_empty() {
                for part in rendered.children {
                    match part {
                        Child::Element(part) if part.attribute("data-node").is_some() => destination.append(part),
                        part @ (Child::Element(_) | Child::Text(_)) => option_descriptions.push(part),
                    }
                }
            } else {
                destination.append(rendered);
            }
        }
        if let Some(group) = tree_group {
            destination.append(group);
        }
        if let Some(state) = state {
            destination.children.push(state);
        }
        if let Some(description) = description {
            destination.children.push(description);
        }
        if !option_descriptions.is_empty() {
            if !element.tag.is_empty() {
                let mut pair = Element::new("");
                pair.append(element);
                element = pair;
            }
            element.children.extend(option_descriptions);
        }
        element
    }
}

fn semantic(element: &Element) -> &Element {
    if element.tag.is_empty() {
        for child in &element.children {
            if let Child::Element(child) = child
                && child.attribute("data-node").is_some()
            {
                return child;
            }
        }
        if let Some(Child::Element(first)) = element.children.first() {
            return first;
        }
        unreachable!("node fragment has its semantic element");
    }
    element
}

pub(crate) fn semantic_mut(element: &mut Element) -> &mut Element {
    if element.tag.is_empty() {
        let index = element
            .children
            .iter()
            .position(|child| match child {
                Child::Element(child) => child.attribute("data-node").is_some(),
                Child::Text(_) => false,
            })
            .unwrap_or(0);
        match element.children.get_mut(index).expect("fragment has its semantic element") {
            Child::Element(child) => return child,
            Child::Text(_) => unreachable!("semantic element is an element"),
        }
    }
    element
}

/// Platform events as data; shortcut interpretation belongs to the page.
/// For an unmapped press, `focus_taken` preserves an outside control's focus.
#[derive(Clone, Debug)]
pub enum Event {
    Click { node: NodeId, modifier: bool },
    PressOutside { focus_taken: bool },
    Input { node: NodeId, value: Box<[u8]>, composing: bool },
    CompositionEnd { node: NodeId, value: Box<[u8]> },
    Enter { node: NodeId, shift: bool, composing: bool },
    Change { node: NodeId },
    Choose { select: NodeId, option: NodeId },
    Cancel { dialog: NodeId },
    Closed { dialog: NodeId },
    Focused { node: NodeId },
    Scrolled { node: NodeId, top: i32, height: i32, viewport: i32 },
    Drawing { node: NodeId, under: Box<[u32]>, x: u16, y: u16 },
    Key { key: Box<[u8]>, code: Box<[u8]>, control: bool, meta: bool, alt: bool, shift: bool, in_field: bool },
}

impl Page<'_> {
    pub fn act(&mut self, event: Event, ops: &mut Vec<Op>) -> Option<(NodeId, Act)> {
        let (node, act) = self.event(event, ops)?;
        if act == Act::Press {
            return self.press(node, ops);
        }
        Some((node, act))
    }

    /// A pointer's press on this node leaves the focus where it was (ui-web.md, 7).
    #[must_use]
    pub fn press_keeps_focus(&self, node: NodeId) -> bool {
        self.held(node).made.attribute.states.as_slice().contains(&State::ToLogEnd)
    }

    fn press(&mut self, node: NodeId, ops: &mut Vec<Op>) -> Option<(NodeId, Act)> {
        if self.held(node).made.attribute.states.as_slice().contains(&State::ToLogEnd) {
            let mut logs = Vec::new();
            for (log, held) in &mut self.nodes {
                if held.made.attribute.role == Role::Log {
                    held.end_requested = true;
                    held.follow_log = true;
                    logs.push(*log);
                }
            }
            for log in logs {
                ops.push(Op::Attributes { node: log, element: self.own_element(log) });
                ops.push(Op::ScrollEnd { node: log });
            }
        }
        self.held(node).made.attribute.bound.then_some((node, Act::Press))
    }

    fn event(&mut self, event: Event, ops: &mut Vec<Op>) -> Option<(NodeId, Act)> {
        match event {
            Event::Click { node, modifier } => self.click(node, modifier, ops),
            Event::PressOutside { focus_taken } => {
                self.close_menu_outside(None, focus_taken, ops);
                None
            }
            Event::Input { node, value, composing } => {
                if self.composition_end.take().is_some_and(|(ended, text)| ended == node && text == value) {
                    return None;
                }
                let a = &self.held(node).made.attribute;
                if !composing && enabled(a) && a.bound && a.role.is_text_field() {
                    Some((node, Act::Input { value }))
                } else {
                    None
                }
            }
            Event::CompositionEnd { node, value } => {
                self.composition_end = Some((node, value.clone()));
                let a = &self.held(node).made.attribute;
                if enabled(a) && a.bound && a.role.is_text_field() { Some((node, Act::Input { value })) } else { None }
            }
            Event::Enter { node, shift, composing } => {
                self.composition_end = None;
                let a = &self.held(node).made.attribute;
                if !composing
                    && enabled(a)
                    && a.bound
                    && a.states.as_slice().contains(&State::SubmitsOnEnter)
                    && (a.role == Role::TextField || (a.role == Role::TextArea && !shift))
                {
                    Some((node, Act::Submit))
                } else {
                    None
                }
            }
            Event::Change { node } => {
                let a = &self.held(node).made.attribute;
                if enabled(a) && a.bound && a.role.is_toggle() {
                    let state = if a.role == Role::Choice { State::Selected } else { State::Pressed };
                    ops.push(Op::Choice { node, checked: a.states.as_slice().contains(&state) });
                    Some((node, Act::Toggle))
                } else {
                    None
                }
            }
            Event::Choose { select, option } => self.choose(select, option, ops),
            Event::Cancel { dialog } => {
                self.held(dialog);
                ops.push(Op::Attributes { node: dialog, element: self.own_element(dialog) });
                if self.menu.is_some() {
                    self.close_menu(true, ops);
                    return None;
                }
                let mut order = Vec::new();
                self.preorder(dialog, &mut order);
                order.into_iter().find_map(|node| {
                    let a = &self.held(node).made.attribute;
                    (enabled(a) && a.bound && a.shortcut == Some(skein_ui::Shortcut::ESCAPE))
                        .then_some((node, Act::Press))
                })
            }
            Event::Closed { dialog } => {
                self.held(dialog);
                ops.push(Op::Attributes { node: dialog, element: self.own_element(dialog) });
                None
            }
            Event::Focused { node } => {
                self.focused(node);
                None
            }
            Event::Scrolled { node, top, height, viewport } => self.scrolled(node, top, height, viewport, ops),
            Event::Drawing { node, under, x, y } => {
                self.close_menu_outside(Some(node), false, ops);
                let a = &self.held(node).made.attribute;
                if !enabled(a) || !a.bound || a.role != Role::Drawing || self.held(node).size.is_none() {
                    return None;
                }
                self.draw(node);
                self.held(node).drawn.as_ref()?;
                let mut data = List::with_capacity(self.act_bytes);
                if !self.scene.refused() && self.scene.act_at(&under, Point { x, y }, &mut data) {
                    Some((node, Act::Drawing { data: data.into_boxed() }))
                } else {
                    None
                }
            }
            event @ Event::Key { .. } => self.key(event, ops),
        }
    }

    // A measure and a person's scroll share the same rules (ui-web.md, 7).
    fn scrolled(
        &mut self,
        node: NodeId,
        top: i32,
        height: i32,
        viewport: i32,
        ops: &mut Vec<Op>,
    ) -> Option<(NodeId, Act)> {
        let held = self.nodes.get_mut(&node).expect("held node");
        let log = held.made.attribute.role == Role::Log;
        let bound_list = held.made.attribute.role == Role::List && held.made.attribute.bound;
        if (!log && !bound_list) || viewport <= 0 {
            return None;
        }
        let continues = held.made.attribute.states.as_slice().contains(&State::Continues);
        let busy = held.made.attribute.states.as_slice().contains(&State::Busy);
        let at_end = top.saturating_add(viewport) >= height.saturating_sub(1) && !continues;
        let mut changed = false;
        if log {
            if held.measured.is_some_and(|(before, _, _)| top < before) || at_end {
                held.end_requested = false;
            }
            held.measured = Some((top, height, viewport));
            let follow = at_end || held.end_requested;
            changed = held.follow_log != follow;
            held.follow_log = follow;
        }
        let foot = height.saturating_sub(top).saturating_sub(viewport);
        let zone = i64::from(viewport) * 3;
        if i64::from(top) * 2 >= zone {
            held.near_head = if busy { Some((top, height)) } else { None };
        }
        if !continues || i64::from(foot) * 2 >= zone {
            held.near_foot = if busy { Some((foot, height)) } else { None };
        }
        let edge = if !held.made.attribute.bound {
            None
        } else if log && i64::from(top) * 2 < zone && report_near(&mut held.near_head, top, height, viewport, busy) {
            Some(LogEdge::Head)
        } else if continues
            && (log || !busy)
            && i64::from(foot) * 2 < zone
            && report_near(&mut held.near_foot, foot, height, viewport, busy)
        {
            Some(LogEdge::Foot)
        } else {
            None
        };
        if changed {
            ops.push(Op::Attributes { node, element: self.own_element(node) });
        }
        edge.map(|edge| (node, Act::Near { edge }))
    }

    fn choose(&self, select: NodeId, option: NodeId, ops: &mut Vec<Op>) -> Option<(NodeId, Act)> {
        let held = self.held(select);
        if held.made.attribute.role != Role::Select || !enabled(&held.made.attribute) {
            return None;
        }
        let options: Vec<_> = held
            .children
            .iter()
            .copied()
            .filter(|child| self.held(*child).made.attribute.role == Role::Option)
            .collect();
        if !options.contains(&option) {
            return None;
        }
        for node in options {
            ops.push(Op::Choice {
                node,
                checked: self.held(node).made.attribute.states.as_slice().contains(&State::Selected),
            });
        }
        let a = &self.held(option).made.attribute;
        (enabled(a) && a.bound).then_some((option, Act::Press))
    }

    fn pressed(&self, node: NodeId) -> NodeId {
        let mut at = node;
        while at != NodeId(0) {
            let held = self.held(at);
            if held.made.attribute.role.is_pressable() {
                return at;
            }
            at = held.parent;
        }
        node
    }

    fn click(&mut self, node: NodeId, modifier: bool, ops: &mut Vec<Op>) -> Option<(NodeId, Act)> {
        self.close_menu_outside(Some(node), false, ops);
        let node = self.pressed(node);
        let a = &self.held(node).made.attribute;
        if !enabled(a) {
            return None;
        }
        if a.role == Role::Menu {
            let open = self.menu != Some(node);
            self.close_menu(true, ops);
            if open {
                self.menu = Some(node);
                menu_element(&mut self.nodes.get_mut(&node).expect("held menu").element, true);
                ops.push(Op::Attributes { node, element: self.own_element(node) });
                if let Some(item) = self.menu_items(node).first() {
                    ops.push(Op::Focus { node: *item });
                }
            }
            return None;
        }
        if a.role == Role::Link && (modifier || a.external.is_some()) {
            return None;
        }
        if (a.bound || a.states.as_slice().contains(&State::ToLogEnd)) && a.role.is_pressable() {
            if a.role == Role::MenuItem {
                self.close_menu(true, ops);
            }
            Some((node, Act::Press))
        } else {
            None
        }
    }

    fn key(&mut self, event: Event, ops: &mut Vec<Op>) -> Option<(NodeId, Act)> {
        let Event::Key { key, code, control, meta, alt, shift, in_field } = event else { unreachable!("key event") };
        if shift {
            return None;
        }
        let modifier = if !control && !meta && !alt {
            Some(Modifier::Plain)
        } else if alt && !control && !meta {
            Some(Modifier::Alt)
        } else if !alt
            && match self.primary {
                Primary::Control => control && !meta,
                Primary::Meta => meta && !control,
            }
        {
            Some(Modifier::Primary)
        } else {
            None
        }?;
        if key.as_ref() == b"PageUp" || key.as_ref() == b"PageDown" {
            if modifier == Modifier::Plain
                && self.menu.is_none()
                && !self.nodes.values().any(|held| held.made.attribute.role == Role::Dialog)
            {
                let mut order = Vec::new();
                for root in &self.roots {
                    self.preorder(*root, &mut order);
                }
                if let Some(node) = order.into_iter().find(|node| {
                    let held = self.held(*node);
                    held.made.attribute.role == Role::Log
                        && held.measured.is_some_and(|(_, height, viewport)| height > viewport)
                }) {
                    ops.push(Op::ScrollScreen { node, down: key.as_ref() == b"PageDown" });
                    ops.push(Op::Measure { node });
                }
            }
            return None;
        }
        if modifier == Modifier::Plain && self.menu_key(&key, ops) {
            return None;
        }
        if modifier == Modifier::Plain && key.as_ref() == b"Escape" {
            return self.form_escape();
        }
        if in_field && modifier == Modifier::Plain {
            return None;
        }
        let key = shortcut_key(&key, &code)?;
        let mut order = Vec::new();
        for root in &self.roots {
            self.preorder(*root, &mut order);
        }
        for node in order {
            let a = &self.held(node).made.attribute;
            if enabled(a) && a.bound && a.shortcut == Some(skein_ui::Shortcut { key, modifier }) {
                return Some((node, Act::Press));
            }
        }
        None
    }

    fn form_escape(&self) -> Option<(NodeId, Act)> {
        let mut ancestor = self.focused?;
        let mut form = None;
        while ancestor != NodeId(0) {
            let held = self.held(ancestor);
            if held.made.attribute.role == Role::Dialog {
                return None;
            }
            if form.is_none() && held.made.attribute.role == Role::Form {
                form = Some(ancestor);
            }
            ancestor = held.parent;
        }
        let mut order = Vec::new();
        self.preorder(form?, &mut order);
        order.into_iter().find_map(|node| {
            let a = &self.held(node).made.attribute;
            (enabled(a) && a.bound && a.shortcut == Some(skein_ui::Shortcut::ESCAPE)).then_some((node, Act::Press))
        })
    }

    fn menu_items(&self, node: NodeId) -> Vec<NodeId> {
        let mut order = Vec::new();
        self.preorder(node, &mut order);
        order
            .into_iter()
            .filter(|node| {
                let a = &self.held(*node).made.attribute;
                a.role == Role::MenuItem && enabled(a)
            })
            .collect()
    }

    fn menu_key(&mut self, key: &[u8], ops: &mut Vec<Op>) -> bool {
        if self.menu.is_none()
            && key == b"ArrowDown"
            && let Some(node) = self.focused
            && self.held(node).made.attribute.role == Role::Menu
        {
            self.click(node, false, ops);
            return true;
        }
        let Some(menu) = self.menu else {
            return false;
        };
        if key == b"Escape" {
            self.close_menu(true, ops);
            return true;
        }
        let items = self.menu_items(menu);
        if items.is_empty() {
            return false;
        }
        let current = items.iter().position(|node| Some(*node) == self.focused);
        let index = match key {
            b"ArrowDown" => current.map_or(0, |n| (n + 1) % items.len()),
            b"ArrowUp" => current.map_or(items.len() - 1, |n| (n + items.len() - 1) % items.len()),
            b"Home" => 0,
            b"End" => items.len() - 1,
            _ => return false,
        };
        ops.push(Op::Focus { node: items[index] });
        true
    }

    fn close_menu_outside(&mut self, node: Option<NodeId>, focus_taken: bool, ops: &mut Vec<Op>) {
        let Some(menu) = self.menu else {
            return;
        };
        let mut inside = Vec::new();
        self.preorder(menu, &mut inside);
        if node.is_some_and(|node| inside.contains(&node)) {
            return;
        }
        let restore_focus = !focus_taken && self.focused.is_none_or(|node| inside.contains(&node));
        self.close_menu(restore_focus, ops);
    }

    fn close_menu(&mut self, restore_focus: bool, ops: &mut Vec<Op>) {
        if let Some(node) = self.menu.take() {
            menu_element(&mut self.nodes.get_mut(&node).expect("held menu").element, false);
            ops.push(Op::Attributes { node, element: self.own_element(node) });
            if restore_focus {
                ops.push(Op::Focus { node });
            }
        }
    }
}

fn enabled(attribute: &skein_ui::Attribute) -> bool {
    !attribute.states.as_slice().contains(&State::Disabled)
}
fn report_near(reported: &mut Option<(i32, i32)>, distance: i32, height: i32, viewport: i32, busy: bool) -> bool {
    // A height change can move the view without a person's scroll.
    if busy && reported.is_some_and(|(_, at_height)| at_height != height) {
        *reported = Some((distance, height));
        return false;
    }
    let report = reported.map_or(!busy, |(last, at_height)| {
        (!busy && at_height != height) || (i64::from(last) - i64::from(distance)) * 4 >= i64::from(viewport)
    });
    if report || reported.is_none() {
        *reported = Some((distance, height));
    }
    report
}
fn shortcut_key(key: &[u8], code: &[u8]) -> Option<u8> {
    if key == b"\\" || code == b"Backslash" || code == b"IntlBackslash" {
        return Some(b'\\');
    }
    if key.len() == 1 && key[0].is_ascii_alphanumeric() {
        return Some(key[0].to_ascii_lowercase());
    }
    let letter = code.strip_prefix(b"Key").or_else(|| code.strip_prefix(b"Digit"))?;
    if letter.len() == 1 && letter[0].is_ascii_alphanumeric() { Some(letter[0].to_ascii_lowercase()) } else { None }
}
fn menu_element(element: &mut Element, open: bool) {
    if let Some(Child::Element(button)) = element.children.first_mut() {
        button.set("aria-expanded", if open { "true" } else { "false" });
    }
    let menu = semantic_mut(element);
    if open {
        menu.remove("hidden");
    } else {
        menu.set("hidden", "");
    }
}
