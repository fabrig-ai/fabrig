//! Keys as acts and focus moves (skein/docs/design/ui/screen.md, 6.2).
use crate::Key;
use alloc::boxed::Box;
use skein_lib::{List, bytes, usize_of};
use skein_ui::{Act, Limits, Modifier, Node, NodeId, Role, Shortcut, State, Tree};
use skein_ui_screen::{Focus, options};

#[derive(PartialEq, Eq, Debug)]
pub enum Command {
    Act { node: NodeId, act: Act },
    Moved,
    Close,
    Nothing,
}

// The binding contract requires Clone; Act owns bytes and has no Clone impl.
// Keep that implementation here without changing skein-ui's public surface.
impl Clone for Command {
    fn clone(&self) -> Self {
        match self {
            Self::Act { node, act } => {
                let act = match act {
                    Act::Press => Act::Press,
                    Act::Input { value } => Act::Input { value: bytes::copy_of(value) },
                    Act::Submit => Act::Submit,
                    Act::Toggle => Act::Toggle,
                    Act::Drawing { data } => Act::Drawing { data: bytes::copy_of(data) },
                    Act::Near { edge } => Act::Near { edge: *edge },
                };
                Self::Act { node: *node, act }
            }
            Self::Moved => Self::Moved,
            Self::Close => Self::Close,
            Self::Nothing => Self::Nothing,
        }
    }
}

#[must_use]
pub fn command(key: &Key, tree: &Tree, focus: &mut Focus, limits: &Limits) -> Command {
    let node = match focus.node {
        Some(id) => tree.find(id),
        None => None,
    };
    if let Some(command) = option_command(key, tree, focus, limits) {
        return command;
    }
    let page = u32::from(focus.rows.saturating_sub(1).max(1));
    match key {
        Key::Control(b'c') => Command::Close,
        Key::Tab | Key::Down => move_focus(tree, focus, true),
        Key::BackTab | Key::Up => move_focus(tree, focus, false),
        Key::PageUp => {
            let followed = focus.follow;
            focus.follow = false;
            let moved = scroll(focus, focus.scroll.saturating_sub(page));
            if followed { Command::Moved } else { moved }
        }
        Key::PageDown => scroll(focus, focus.scroll.saturating_add(page)),
        Key::Escape => escape(tree, focus.node),
        Key::Control(letter) => shortcut(tree, char::from(*letter), Modifier::Primary),
        Key::Alt(letter) => shortcut(tree, *letter, Modifier::Alt),
        Key::Enter => {
            let Some(node) = node else { return Command::Nothing };
            let act = if node.role.is_text_field() {
                Act::Submit
            } else if node.role.is_toggle() {
                Act::Toggle
            } else {
                Act::Press
            };
            act_on(tree, node, act)
        }
        Key::Char(ch) => {
            if let Some(node) = node {
                if node.role.is_text_field() {
                    let mut buffer = [0; 4];
                    return edit(tree, node, focus, ch.encode_utf8(&mut buffer).as_bytes(), Edit::Insert, limits);
                }
                if *ch == ' ' {
                    let act = if node.role.is_toggle() { Act::Toggle } else { Act::Press };
                    return act_on(tree, node, act);
                }
            }
            shortcut(tree, *ch, Modifier::Plain)
        }
        Key::Paste(text) => {
            let Some(node) = node else { return Command::Nothing };
            if node.role == Role::TextField {
                let text = field_paste(text);
                edit(tree, node, focus, &text, Edit::Insert, limits)
            } else {
                edit(tree, node, focus, text, Edit::Insert, limits)
            }
        }
        Key::Backspace | Key::Delete => {
            let Some(node) = node else { return Command::Nothing };
            let edit = if *key == Key::Backspace { Edit::Before } else { Edit::After };
            edit_value(tree, node, focus, edit, limits)
        }
        Key::Left | Key::Right | Key::Home | Key::End => {
            let Some(node) = node else { return Command::Nothing };
            caret(key, tree, node, focus)
        }
    }
}

fn option_command(key: &Key, tree: &Tree, focus: &mut Focus, limits: &Limits) -> Option<Command> {
    let menu = options::menu(tree, focus.node);
    focus.active = match menu {
        Some(menu) => options::active(tree, menu.id, focus.active),
        None => None,
    };
    let active = tree.find(focus.active?)?;
    let menu = menu?;
    match key {
        Key::Down => Some(move_active(tree, menu, focus, true)),
        Key::Up => Some(move_active(tree, menu, focus, false)),
        Key::Enter => Some(act_on(tree, active, Act::Press)),
        Key::Tab => {
            let field = tree.find(focus.node?)?;
            let name = active.name.as_deref()?;
            if !name.starts_with(value(field)) {
                return None;
            }
            let other = usize_of(field.text_bytes()).saturating_sub(value(field).len());
            if name.len().saturating_add(other) > usize_of(limits.text_per_node) {
                return Some(Command::Nothing);
            }
            focus.caret = u32::try_from(name.len()).expect("item name fits node text bound");
            Some(Command::Act { node: field.id, act: Act::Input { value: bytes::copy_of(name) } })
        }
        Key::Char(_)
        | Key::Control(_)
        | Key::Alt(_)
        | Key::Paste(_)
        | Key::BackTab
        | Key::Backspace
        | Key::Delete
        | Key::Left
        | Key::Right
        | Key::Home
        | Key::End
        | Key::PageUp
        | Key::PageDown
        | Key::Escape => None,
    }
}

fn move_active(tree: &Tree, menu: &Node, focus: &mut Focus, forward: bool) -> Command {
    let mut left = 0;
    let mut before = None;
    let mut after = None;
    let mut seen = false;
    for node in tree.nodes() {
        if node.id == menu.id {
            left = menu.size.saturating_sub(1);
            continue;
        }
        if left == 0 {
            continue;
        }
        left = left.saturating_sub(1);
        if node.role != Role::MenuItem || tree.disabled(node.id) {
            continue;
        }
        if Some(node.id) == focus.active {
            seen = true;
        } else if seen {
            after = Some(node.id);
            break;
        } else {
            before = Some(node.id);
        }
    }
    let next = if forward { after } else { before };
    if let Some(next) = next {
        focus.active = Some(next);
        Command::Moved
    } else {
        Command::Nothing
    }
}

fn available(tree: &Tree, node: &Node) -> bool {
    tree.bound(node.id).is_some() && !tree.disabled(node.id)
}
fn actionable(node: &Node) -> bool {
    node.role.is_text_field() || node.role.is_toggle() || skein_ui::takes(node.role, false, &Act::Press)
}
fn act_on(tree: &Tree, node: &Node, act: Act) -> Command {
    if available(tree, node)
        && skein_ui::takes(node.role, node.states.as_slice().contains(&State::SubmitsOnEnter), &act)
    {
        Command::Act { node: node.id, act }
    } else {
        Command::Nothing
    }
}
fn move_focus(tree: &Tree, focus: &mut Focus, forward: bool) -> Command {
    let mut first = None;
    let mut last = None;
    let mut before = None;
    let mut after = None;
    let mut seen = false;
    for node in tree.nodes() {
        if actionable(node) && available(tree, node) {
            if first.is_none() {
                first = Some(node);
            }
            last = Some(node);
            if seen && after.is_none() {
                after = Some(node);
            }
            if Some(node.id) == focus.node {
                seen = true;
            } else if !seen {
                before = Some(node);
            }
        }
    }
    let next = if forward {
        after.or(first)
    } else if seen {
        before.or(last)
    } else {
        last
    };
    let Some(next) = next else { return Command::Nothing };
    let caret = u32::try_from(value(next).len()).expect("node value fits u32");
    if focus.node == Some(next.id) && focus.caret == caret {
        return Command::Nothing;
    }
    focus.active = None;
    focus.node = Some(next.id);
    focus.caret = caret;
    Command::Moved
}
fn scroll(focus: &mut Focus, to: u32) -> Command {
    if focus.scroll == to {
        return Command::Nothing;
    }
    focus.scroll = to;
    Command::Moved
}
fn shortcut(tree: &Tree, ch: char, modifier: Modifier) -> Command {
    let Ok(key) = u8::try_from(ch) else { return Command::Nothing };
    if !(key.is_ascii_lowercase() || key.is_ascii_digit() || key == b'\\' && modifier == Modifier::Primary) {
        return Command::Nothing;
    }
    for node in tree.nodes() {
        if node.shortcut == Some(Shortcut { key, modifier }) && available(tree, node) {
            let command = act_on(tree, node, Act::Press);
            if command != Command::Nothing {
                return command;
            }
        }
    }
    Command::Nothing
}
fn escape(tree: &Tree, focused: Option<NodeId>) -> Command {
    let mut dialog = None;
    let mut form = None;
    let mut at = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if Some(node.id) == focused {
            at = Some(index);
        }
    }
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Dialog {
            dialog = Some((index, node));
        }
        if node.role == Role::Form
            && let Some(at) = at
        {
            let end = index.saturating_add(usize_of(node.size));
            if index <= at && at < end {
                form = Some((index, node));
            }
        }
    }
    if let Some((start, root)) = dialog.or(form) {
        let end = start.saturating_add(usize_of(root.size));
        for node in tree.nodes().get(start..end).expect("subtree is inside tree") {
            if node.shortcut == Some(Shortcut::ESCAPE) && available(tree, node) {
                let command = act_on(tree, node, Act::Press);
                if command != Command::Nothing {
                    return command;
                }
            }
        }
    }
    Command::Nothing
}
fn value(node: &Node) -> &[u8] {
    match &node.value {
        Some(value) => &value.text,
        None => b"",
    }
}
// A UTF-8 continuation belongs to the preceding character. Invalid tree bytes
// remain editable as individual bytes, as they remain visible as replacements.
fn boundary(text: &[u8], at: usize) -> bool {
    match text.get(at) {
        Some(byte) => byte & 0xc0 != 0x80,
        None => true,
    }
}
fn offset(node: &Node, caret: u32) -> usize {
    let text = value(node);
    let mut at = usize_of(caret).min(text.len());
    for _ in 0_u8..4 {
        if at == 0 || boundary(text, at) {
            break;
        }
        at = at.saturating_sub(1);
    }
    at
}
fn previous(text: &[u8], at: usize) -> usize {
    let mut before = 0;
    for index in 0..at {
        if boundary(text, index) {
            before = index;
        }
    }
    before
}
fn following(text: &[u8], at: usize) -> usize {
    for index in at.saturating_add(1)..text.len() {
        if boundary(text, index) {
            return index;
        }
    }
    text.len()
}
fn caret(key: &Key, tree: &Tree, node: &Node, focus: &mut Focus) -> Command {
    if !node.role.is_text_field() || !available(tree, node) {
        return Command::Nothing;
    }
    let text = value(node);
    let at = offset(node, focus.caret);
    let to = match key {
        Key::Left => previous(text, at),
        Key::Right => following(text, at),
        Key::Home => 0,
        Key::End => text.len(),
        Key::Char(_)
        | Key::Paste(_)
        | Key::Enter
        | Key::Tab
        | Key::BackTab
        | Key::Backspace
        | Key::Delete
        | Key::Up
        | Key::Down
        | Key::PageUp
        | Key::PageDown
        | Key::Escape
        | Key::Control(_)
        | Key::Alt(_) => return Command::Nothing,
    };
    let to = u32::try_from(to).expect("value length fits u32");
    if to == focus.caret {
        return Command::Nothing;
    }
    focus.caret = to;
    Command::Moved
}
#[derive(Clone, Copy)]
enum Edit {
    Insert,
    Before,
    After,
}
fn edit_value(tree: &Tree, node: &Node, focus: &mut Focus, kind: Edit, limits: &Limits) -> Command {
    edit(tree, node, focus, b"", kind, limits)
}
fn edit(tree: &Tree, node: &Node, focus: &mut Focus, text: &[u8], kind: Edit, limits: &Limits) -> Command {
    if !node.role.is_text_field() || !available(tree, node) {
        return Command::Nothing;
    }
    let old = value(node);
    let at = offset(node, focus.caret);
    let (start, end) = match kind {
        Edit::Insert => (at, at),
        Edit::Before => (previous(old, at), at),
        Edit::After => (at, following(old, at)),
    };
    if start == end && text.is_empty() {
        return Command::Nothing;
    }
    let length = old.len().saturating_sub(end.saturating_sub(start)).saturating_add(text.len());
    let other = usize_of(node.text_bytes()).saturating_sub(old.len());
    if length.saturating_add(other) > usize_of(limits.text_per_node) {
        return Command::Nothing;
    }
    let mut new = List::with_capacity(u32::try_from(length).expect("input fits node limit"));
    for part in [old.get(..start).expect("caret is in value"), text, old.get(end..).expect("caret is in value")] {
        for byte in part {
            new.push(*byte).expect("input fits computed length");
        }
    }
    focus.caret = u32::try_from(start.saturating_add(text.len())).expect("caret fits input");
    Command::Act { node: node.id, act: Act::Input { value: new.into_boxed() } }
}
fn field_paste(text: &[u8]) -> Box<[u8]> {
    let mut out = List::with_capacity(u32::try_from(text.len()).expect("paste length fits u32"));
    let mut cr = false;
    let mut kept = 0;
    for byte in text {
        if *byte == b'\n' && cr {
            cr = false;
            continue;
        }
        cr = *byte == b'\r';
        let newline = *byte == b'\r' || *byte == b'\n';
        out.push(if newline { b' ' } else { *byte }).expect("normalized paste does not grow");
        if !newline {
            kept = out.len();
        }
    }
    out.truncate(kept);
    out.into_boxed()
}
