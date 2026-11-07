//! Bands, bounded row buffers and the preorder walk (skein/docs/design/ui-screen.md, section 5).

mod canvas;
mod roles;
mod words;

use skein_lib::{List, Stack};
use skein_ui::{Node, Role, Tree};

use crate::{Arrange, Cell, Focus, Frame, Paint, Place, RegionLook, Size};
use canvas::{Canvas, Viewport};

#[derive(Clone, Copy, Debug)]
struct Info {
    parent: Option<u32>,
    paint: Paint,
    look: Option<RegionLook>,
    title: bool,
}

fn look(node: &Node, theme: &crate::Theme) -> Option<RegionLook> {
    for look in &theme.regions {
        if Some(look.region) == node.region {
            return Some(*look);
        }
    }
    None
}

fn information(tree: &Tree, theme: &crate::Theme) -> List<Info> {
    let mut out: List<Info> = List::with_capacity(tree.len());
    let mut parents: List<u32> = List::with_capacity(tree.len());
    for at in 0..tree.len() {
        for _ancestor in 0..parents.len() {
            let parent = *parents.last().expect("ancestor exists");
            if parent.saturating_add(tree.get(parent).expect("ancestor exists").size) > at {
                break;
            }
            parents.truncate(parents.len().saturating_sub(1));
        }
        let node = tree.get(at).expect("preorder node exists");
        let placed = look(node, theme);
        let mut region = theme.base;
        let mut tokens = Paint::NONE;
        for ancestor in &parents {
            let n = tree.get(*ancestor).expect("ancestor exists");
            if let Some(look) = look(n, theme) {
                region = look.paint.over(region);
            }
            tokens = styles(n, theme).over(tokens);
        }
        if let Some(look) = placed {
            region = look.paint.over(region);
        }
        tokens = styles(node, theme).over(tokens);
        let parent = parents.last().copied();
        out.push(Info { parent, paint: tokens.over(region), look: placed, title: false }).expect("one entry per node");
        parents.push(at).expect("ancestors fit nodes");
    }
    for at in 0..tree.len() {
        if let Some(look) = out.get(at).expect("node information exists").look
            && let Some(title) = title_node(tree, at, look)
        {
            out.get_mut(title).expect("title information exists").title = true;
        }
    }
    out
}

fn styles(node: &Node, theme: &crate::Theme) -> Paint {
    let mut paint = Paint::NONE;
    for token in &node.styles {
        for look in &theme.styles {
            if *token == look.style {
                paint = look.paint.over(paint);
            }
        }
    }
    paint
}

fn actionable(role: Role) -> bool {
    match role {
        Role::Button
        | Role::Link
        | Role::TextField
        | Role::TextArea
        | Role::Choice
        | Role::Check
        | Role::Option
        | Role::Tab
        | Role::MenuItem => true,
        Role::Page
        | Role::Banner
        | Role::Navigation
        | Role::Main
        | Role::Complementary
        | Role::ContentInfo
        | Role::Form
        | Role::Search
        | Role::Region
        | Role::Group
        | Role::Heading { .. }
        | Role::Paragraph
        | Role::Text
        | Role::List
        | Role::Item
        | Role::Table
        | Role::Row
        | Role::Cell
        | Role::ColumnHeader
        | Role::RowHeader
        | Role::Select
        | Role::Term
        | Role::Description
        | Role::ChoiceGroup
        | Role::Dialog
        | Role::Status
        | Role::Alert
        | Role::Log
        | Role::Tree
        | Role::TreeItem
        | Role::Tabs
        | Role::Progress
        | Role::Time
        | Role::Code
        | Role::Quote
        | Role::Separator
        | Role::Menu
        | Role::Disclosure
        | Role::Image
        | Role::Figure
        | Role::Slot
        | Role::Drawing => false,
    }
}

fn focus_requests(tree: &Tree, focus: &mut Focus) {
    for node in tree.nodes() {
        if let Some(request) = node.focus
            && request > focus.honoured
        {
            focus.node = Some(node.id);
            focus.honoured = request;
            focus.caret = match &node.value {
                Some(value) => u32::try_from(value.text.len()).expect("node text fits u32"),
                None => 0,
            };
        }
    }
    if let Some(id) = focus.node {
        let valid = match tree.find(id) {
            Some(node) => actionable(node.role) && tree.bound(node.id).is_some() && !tree.disabled(node.id),
            None => false,
        };
        if !valid {
            focus.node = None;
        }
    }
}

/// Lays the tree into the frame, honours focus requests and keeps the focus visible
/// (skein/docs/design/ui-screen.md, section 5).
pub fn render(tree: &Tree, theme: &crate::Theme, size: Size, focus: &mut Focus, frame: &mut Frame) {
    focus_requests(tree, focus);
    frame.clear(size, Cell { glyph: ' ', paint: theme.base });
    let size = frame.size();
    if size.rows == 0 || size.columns == 0 {
        focus.scroll = 0;
        frame.set_scroll(0);
        return;
    }
    let info = information(tree, theme);
    let mut bands = List::with_capacity(tree.len());
    for at in 0..tree.len() {
        if let Some(look) = info.get(at).expect("node information exists").look {
            match look.place {
                Place::Top | Place::Bottom => {
                    let mut canvas = Canvas::new(size.columns, theme.base, Viewport::measure(), tree.len());
                    walk(tree, &info, Some(at), theme, focus, &mut canvas, frame);
                    bands
                        .push(Band { at, place: look.place, rows: canvas.rows(), shown: 0, origin: 0 })
                        .expect("bands fit nodes");
                }
                Place::Body => {}
            }
        }
    }
    let mut left = size.rows;
    // Bottom bands reserve their room before top bands (5.2).
    for band in bands.iter_mut().rev() {
        if band.place == Place::Bottom {
            reserve(band, &mut left);
        }
    }
    for band in bands.iter_mut() {
        if band.place == Place::Top {
            reserve(band, &mut left);
        }
    }
    let body_rows = left;
    let mut top = 0_u16;
    let mut bottom = size.rows;
    for band in bands.iter_mut().rev() {
        if band.place == Place::Bottom {
            bottom = bottom.saturating_sub(band.shown);
            band.origin = bottom;
        }
    }
    for band in bands.iter_mut() {
        if band.place == Place::Top {
            band.origin = top;
            top = top.saturating_add(band.shown);
        }
        let viewport = Viewport { first: 0, origin: band.origin, height: band.shown, draw: true };
        let mut canvas = Canvas::new(size.columns, theme.base, viewport, tree.len());
        walk(tree, &info, Some(band.at), theme, focus, &mut canvas, frame);
    }
    let mut measured = Canvas::new(size.columns, theme.base, Viewport::measure(), tree.len());
    walk(tree, &info, None, theme, focus, &mut measured, frame);
    let last = measured.rows().saturating_sub(u32::from(body_rows.max(1)));
    focus.scroll = focus.scroll.min(last);
    if body_rows != 0
        && let Some((first, end)) = measured.focus_rows()
    {
        if first < focus.scroll {
            focus.scroll = first;
        } else if end > focus.scroll.saturating_add(u32::from(body_rows)) {
            focus.scroll = end.saturating_sub(u32::from(body_rows)).min(first);
        }
    }
    frame.set_scroll(focus.scroll);
    let viewport = Viewport { first: focus.scroll, origin: top, height: body_rows, draw: true };
    let mut canvas = Canvas::new(size.columns, theme.base, viewport, tree.len());
    walk(tree, &info, None, theme, focus, &mut canvas, frame);
}

#[derive(Debug)]
struct Band {
    at: u32,
    place: Place,
    rows: u32,
    shown: u16,
    origin: u16,
}

fn reserve(band: &mut Band, left: &mut u16) {
    band.shown = u16::try_from(band.rows.min(u32::from(*left))).expect("room fits u16");
    *left = left.saturating_sub(band.shown);
}

#[derive(Debug)]
struct Parent {
    end: u32,
    indent: u16,
    next: Option<u64>,
    region: Option<RegionLook>,
}

#[expect(clippy::too_many_lines, reason = "the preorder walk keeps region entry, role layout and region exit together")]
fn walk(
    tree: &Tree,
    info: &List<Info>,
    root: Option<u32>,
    theme: &crate::Theme,
    focus: &Focus,
    out: &mut Canvas,
    frame: &mut Frame,
) {
    let (begin, end) = match root {
        Some(at) => (at, at.saturating_add(tree.get(at).expect("band root exists").size)),
        None => (0, tree.len()),
    };
    let mut parents: Stack<Parent> = Stack::with_capacity(tree.len());
    let mut skip = begin;
    for at in begin..end {
        if at < skip {
            continue;
        }
        for _ancestor in 0..parents.len() {
            if parents.top().expect("ancestor exists").end > at {
                break;
            }
            let parent = parents.pop().expect("ancestor exists");
            out.finish_annotated(tree, info, frame);
            if let Some(region) = parent.region {
                out.leave_region(region, frame);
            }
        }
        let n = tree.get(at).expect("preorder node exists");
        let data = *info.get(at).expect("node information exists");
        let until = at.saturating_add(n.size);
        if let Some(look) = data.look {
            let separate = match look.place {
                Place::Top | Place::Bottom => root != Some(at),
                Place::Body => false,
            };
            if separate {
                skip = until;
                continue;
            }
        }
        let indent = match parents.top() {
            Some(p) => p.indent,
            None => 0,
        };
        if data.title {
            let indent = indent.saturating_add(2);
            descriptions(tree, at, until, indent, info, theme, focus, out, frame);
            skip = until;
            continue;
        }
        let mut region = data.look;
        if let Some(look) = region {
            out.enter_region(look, data.paint, frame);
            if let Some(title) = title_node(tree, at, look) {
                let mut title_line = Canvas::first_line(out.columns(), theme.base, tree.len());
                content(tree, title, info, theme, false, &mut title_line, frame);
                title_line.finish(frame);
                out.title(&title_line, look, frame);
            }
            if look.arrange == Arrange::Spread {
                spread(tree, at, info, theme, focus, out, frame);
                out.leave_region(look, frame);
                skip = until;
                continue;
            }
        }
        out.start(indent);
        let mut number = None;
        if n.role == Role::Item
            && let Some(parent) = parents.top_mut()
        {
            number = parent.next;
            if let Some(next) = &mut parent.next {
                *next = next.saturating_add(1);
            }
        }
        let within_item = out.within_item();
        let description_indent = out.description_indent();
        let child_indent = node(tree, at, info, theme, focus, number, out, frame);
        match child_indent {
            Some(indent) => {
                if !(n.role.is_list_item() || n.role == Role::Disclosure && n.name.is_some()) {
                    let described_indent = if within_item { description_indent } else { indent };
                    descriptions(tree, at, at.saturating_add(1), described_indent, info, theme, focus, out, frame);
                }
                let next = ordered(n);
                parents.push(Parent { end: until, indent, next, region }).expect("parents fit nodes");
            }
            None => {
                let indent = if n.role == Role::Row && roles::empty_header_row(tree, at) {
                    out.indent()
                } else {
                    description_indent
                };
                descriptions(tree, at, until, indent, info, theme, focus, out, frame);
                if let Some(look) = region.take() {
                    out.leave_region(look, frame);
                }
                skip = until;
            }
        }
    }
    for _ancestor in 0..parents.len() {
        let parent = parents.pop().expect("ancestor exists");
        out.finish_annotated(tree, info, frame);
        if let Some(region) = parent.region {
            out.leave_region(region, frame);
        }
    }
    out.finish(frame);
}

#[expect(clippy::manual_find, reason = "step code scans without closures")]
fn title_node(tree: &Tree, at: u32, look: RegionLook) -> Option<u32> {
    let border = look.border?;
    if !border.title {
        return None;
    }
    let root = tree.get(at).expect("region exists");
    for child in at.saturating_add(1)..at.saturating_add(root.size) {
        if tree.get(child).expect("descendant exists").role == (Role::Heading { level: 1 }) {
            return Some(child);
        }
    }
    None
}

#[expect(clippy::too_many_arguments, reason = "role rendering borrows the walk's inputs")]
#[expect(clippy::match_same_arms, reason = "the role contract keeps one arm per newly mapped role")]
fn node(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &crate::Theme,
    focus: &Focus,
    number: Option<u64>,
    out: &mut Canvas,
    frame: &mut Frame,
) -> Option<u16> {
    let n = tree.get(at).expect("node exists");
    let paint = info.get(at).expect("node information exists").paint;
    let focused = focus.node == Some(n.id);
    out.tag_focus(focused);
    let indent = out.indent();
    match n.role {
        Role::Page | Role::Group | Role::Slot => Some(indent),
        Role::Table => Some(indent),
        Role::Row => {
            roles::row(tree, at, info, theme, focused, out, frame);
            None
        }
        Role::Cell => roles::line(tree, at, info, theme, focused, out, frame),
        Role::ColumnHeader => roles::line(tree, at, info, theme, focused, out, frame),
        Role::RowHeader => roles::line(tree, at, info, theme, focused, out, frame),
        Role::Term => roles::line(tree, at, info, theme, focused, out, frame),
        Role::Description => Some(indent.saturating_add(2)),
        Role::Tree => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::TreeItem => Some(roles::item(tree, at, info, theme, focused, None, out, frame)),
        Role::Menu => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::MenuItem => control(tree, at, info, theme, focus, out, frame),
        Role::ChoiceGroup => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Choice => control(tree, at, info, theme, focus, out, frame),
        Role::Select => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Option => control(tree, at, info, theme, focus, out, frame),
        Role::Check => control(tree, at, info, theme, focus, out, frame),
        Role::Tabs => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Tab => control(tree, at, info, theme, focus, out, frame),
        Role::Log => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Progress => roles::line(tree, at, info, theme, focused, out, frame),
        Role::Code => roles::line(tree, at, info, theme, focused, out, frame),
        Role::Quote => roles::line(tree, at, info, theme, focused, out, frame),
        Role::Figure => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Dialog => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Disclosure => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Drawing => Some(roles::named(tree, at, info, focused, out, frame)),
        Role::Image => roles::line(tree, at, info, theme, focused, out, frame),
        Role::Banner
        | Role::Navigation
        | Role::Main
        | Role::Complementary
        | Role::ContentInfo
        | Role::Form
        | Role::Search
        | Role::Region
        | Role::List => {
            if let Some(name) = &n.name {
                out.safe(name, paint, focused, frame);
                words::colon(name, paint, out, frame);
                words::states(n, paint, out, frame);
                out.finish_annotated(tree, info, frame);
                Some(indent.saturating_add(2))
            } else {
                Some(indent)
            }
        }
        Role::Item => Some(roles::item(tree, at, info, theme, focused, number, out, frame)),
        Role::Heading { level } => {
            for _mark in 0..level {
                out.safe(b"#", paint, focused, frame);
            }
            out.safe(b" ", paint, focused, frame);
            content(tree, at, info, theme, focused, out, frame);
            words::states(n, paint, out, frame);
            out.finish_annotated(tree, info, frame);
            None
        }
        Role::Paragraph | Role::Text | Role::Time | Role::Status | Role::Alert | Role::Separator => {
            out.line();
            if n.role == Role::Alert {
                out.safe(b"! ", paint, focused, frame);
            }
            if n.role == Role::Separator {
                out.safe(b"--", paint, focused, frame);
            } else {
                content(tree, at, info, theme, focused, out, frame);
            }
            words::states(n, paint, out, frame);
            out.finish_annotated(tree, info, frame);
            None
        }
        Role::Button | Role::Link | Role::TextField | Role::TextArea => {
            control(tree, at, info, theme, focus, out, frame)
        }
    }
}

#[expect(clippy::manual_map, reason = "step code uses an explicit match instead of Option combinators")]
fn ordered(n: &Node) -> Option<u64> {
    if n.role == Role::List {
        match n.ordered {
            Some(first) => Some(u64::from(first)),
            None => None,
        }
    } else {
        None
    }
}

fn control(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &crate::Theme,
    focus: &Focus,
    out: &mut Canvas,
    frame: &mut Frame,
) -> Option<u16> {
    let n = tree.get(at).expect("control exists");
    let paint = info.get(at).expect("control information exists").paint;
    let focused = focus.node == Some(n.id);
    let paint = if focused { theme.focus.over(paint) } else { paint };
    out.clip_line(n.role == Role::TextField);
    out.control(focused, theme.glyphs.focus, paint, frame);
    words::mark(n, paint, out, frame);
    content(tree, at, info, theme, focused, out, frame);
    if n.role.is_text_field() {
        out.safe(b": ", paint, focused, frame);
        out.field(n, focus, theme, paint, frame);
        words::entry(n, paint, out, frame);
    }
    if tree.disabled(n.id) && !n.states.as_slice().contains(&skein_ui::State::Disabled) {
        out.safe(words::DISABLED, paint, focused, frame);
    }
    words::states(n, paint, out, frame);
    out.finish_annotated(tree, info, frame);
    out.clip_line(false);
    out.tag_focus(false);
    None
}

fn content(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &crate::Theme,
    focused: bool,
    out: &mut Canvas,
    frame: &mut Frame,
) {
    let root = tree.get(at).expect("content root exists");
    let end = at.saturating_add(root.size);
    let mut found = false;
    let mut external: Stack<u32> = Stack::with_capacity(tree.len());
    for index in at..end {
        for _link in 0..external.len() {
            let link = *external.top().expect("link exists");
            if link.saturating_add(tree.get(link).expect("link exists").size) > index {
                break;
            }
            external_url(tree, external.pop().expect("link exists"), info, theme, focused, out, frame);
        }
        let node = tree.get(index).expect("content descendant exists");
        let mut paint = info.get(index).expect("node information exists").paint;
        if focused {
            paint = theme.focus.over(paint);
        }
        for run in &node.text {
            found = true;
            match run.run {
                skein_ui::Run::Code => {
                    out.content(b"`", paint, focused, tree, info, frame);
                    out.content(&run.text, paint, focused, tree, info, frame);
                    out.content(b"`", paint, focused, tree, info, frame);
                }
                skein_ui::Run::Plain | skein_ui::Run::Emphasis | skein_ui::Run::Strong | skein_ui::Run::Link { .. } => {
                    out.content(&run.text, paint, focused, tree, info, frame);
                }
            }
        }
        if node.external.is_some() {
            external.push(index).expect("links fit nodes");
        }
    }
    if !found
        && root.role != Role::Progress
        && let Some(name) = &root.name
    {
        let mut paint = info.get(at).expect("node information exists").paint;
        if focused {
            paint = theme.focus.over(paint);
        }
        out.content(name, paint, focused, tree, info, frame);
    }
    for _link in 0..external.len() {
        external_url(tree, external.pop().expect("link exists"), info, theme, focused, out, frame);
    }
}

fn external_url(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &crate::Theme,
    focused: bool,
    out: &mut Canvas,
    frame: &mut Frame,
) {
    let n = tree.get(at).expect("link exists");
    if let Some(url) = &n.external {
        let mut paint = info.get(at).expect("node information exists").paint;
        if focused {
            paint = theme.focus.over(paint);
        }
        out.safe(b" <", paint, focused, frame);
        out.safe(url, paint, focused, frame);
        out.safe(b">", paint, focused, frame);
    }
}

#[expect(clippy::too_many_arguments, reason = "descriptions follow the consumed subtree in preorder")]
fn descriptions(
    tree: &Tree,
    begin: u32,
    end: u32,
    description_indent: u16,
    info: &List<Info>,
    theme: &crate::Theme,
    focus: &Focus,
    out: &mut Canvas,
    frame: &mut Frame,
) {
    let indent = out.indent();
    for at in begin..end {
        let n = tree.get(at).expect("described node exists");
        if let Some(bytes) = &n.description {
            out.finish(frame);
            out.start(description_indent);
            let focused = focus.node == Some(n.id);
            let mut paint = info.get(at).expect("node information exists").paint;
            if focused {
                paint = theme.focus.over(paint);
            }
            out.tag_focus(focused);
            out.safe(bytes, paint, focused, frame);
            out.tag_focus(false);
            out.finish(frame);
        }
    }
    out.start(indent);
}

fn spread(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &crate::Theme,
    focus: &Focus,
    out: &mut Canvas,
    frame: &mut Frame,
) {
    let n = tree.get(at).expect("spread region exists");
    let mut children = List::with_capacity(n.size);
    let mut skip = at.saturating_add(1);
    for child in at.saturating_add(1)..at.saturating_add(n.size) {
        if child < skip {
            continue;
        }
        if !info.get(child).expect("child information exists").title {
            children.push(child).expect("children fit nodes");
        }
        skip = child.saturating_add(tree.get(child).expect("child exists").size);
    }
    let mut right = out.room();
    for (number, child) in children.iter().enumerate().rev() {
        let mut line = Canvas::first_line(out.room(), theme.base, tree.len());
        first_child_line(tree, *child, info, theme, focus, &mut line, frame);
        if number == 0 {
            out.spread_line(&line, 0, right);
        } else {
            let length = line.first_length().min(right);
            right = right.saturating_sub(length);
            out.spread_line(&line, right, length);
            right = right.saturating_sub(2);
        }
    }
    out.blank_line(frame);
}

/// Spread consumes a child's first role line without re-entering band layout.
/// Like the main walk, it descends with an ancestor stack, never recursion.
fn first_child_line(
    tree: &Tree,
    at: u32,
    info: &List<Info>,
    theme: &crate::Theme,
    focus: &Focus,
    out: &mut Canvas,
    frame: &mut Frame,
) {
    let end = at.saturating_add(tree.get(at).expect("spread child exists").size);
    let mut parents: Stack<Parent> = Stack::with_capacity(tree.len());
    let mut skip = at;
    for child in at..end {
        if child < skip {
            continue;
        }
        for _ancestor in 0..parents.len() {
            if parents.top().expect("ancestor exists").end > child {
                break;
            }
            let _parent = parents.pop().expect("ancestor exists");
            out.finish_annotated(tree, info, frame);
        }
        let n = tree.get(child).expect("child descendant exists");
        if info.get(child).expect("child information exists").title {
            skip = child.saturating_add(n.size);
            continue;
        }
        let indent = match parents.top() {
            Some(parent) => parent.indent,
            None => 0,
        };
        out.start(indent);
        let mut number = None;
        if n.role == Role::Item
            && let Some(parent) = parents.top_mut()
        {
            number = parent.next;
            if let Some(next) = &mut parent.next {
                *next = next.saturating_add(1);
            }
        }
        let within_item = out.within_item();
        let description_indent = out.description_indent();
        match node(tree, child, info, theme, focus, number, out, frame) {
            Some(indent) => {
                if !(n.role.is_list_item() || n.role == Role::Disclosure && n.name.is_some()) {
                    let described_indent = if within_item { description_indent } else { indent };
                    descriptions(
                        tree,
                        child,
                        child.saturating_add(1),
                        described_indent,
                        info,
                        theme,
                        focus,
                        out,
                        frame,
                    );
                }
                let next = ordered(n);
                parents
                    .push(Parent { end: child.saturating_add(n.size), indent, next, region: None })
                    .expect("ancestors fit nodes");
            }
            None => {
                let indent = if n.role == Role::Row && roles::empty_header_row(tree, child) {
                    out.indent()
                } else {
                    description_indent
                };
                descriptions(tree, child, child.saturating_add(n.size), indent, info, theme, focus, out, frame);
                skip = child.saturating_add(n.size);
            }
        }
    }
    out.finish_annotated(tree, info, frame);
}
