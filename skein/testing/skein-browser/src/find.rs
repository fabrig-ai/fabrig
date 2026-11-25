//! Name comparisons and points used by browser operations.

use alloc::boxed::Box;
use skein_lib::Map;

use crate::boundary::{Point, Rect};
use crate::wire::decode::{Array, Text, Value};

/// The query's bytes occur unbroken, with case kept.
pub(crate) fn words(name: &[u8], query: &[u8]) -> bool {
    if query.is_empty() {
        return true;
    }
    for part in name.windows(query.len()) {
        if part == query {
            return true;
        }
    }
    false
}

pub(crate) fn matching_name<'a>(node: Value<'a>, query: &[u8]) -> Option<Text<'a>> {
    let name = full_name(node).unwrap_or(Text::Plain(b""));
    words(name.bytes(), query).then_some(name)
}

/// Index the scoped AX tree once. Paragraphs have no accessible name; the
/// rendered words are their `StaticText` descendants, not the paragraph's label.
pub(crate) struct Paragraphs<'a> {
    nodes: Map<Box<[u8]>, Value<'a>>,
}
impl<'a> Paragraphs<'a> {
    pub(crate) fn new(nodes: Array<'a>) -> Self {
        let mut by_id = Map::with_capacity(u32::try_from(nodes.clone().count()).expect("AX reply is bounded"));
        for node in nodes {
            if let Some(id) = node_id(node) {
                by_id.insert(id, node).expect("AX reply node count");
            }
        }
        Self { nodes: by_id }
    }

    #[expect(clippy::disallowed_types, reason = "bounded stack of borrowed AX child iterators")]
    pub(crate) fn matches(&self, node: Value<'_>, mut text: &[u8]) -> bool {
        use alloc::vec::Vec;
        if role(node).as_deref() != Some(b"paragraph") || ignored(node) {
            return false;
        }
        let mut stack = Vec::new();
        if let Some(children) = child_ids(node) {
            stack.push(children);
        }
        // A malformed reply cannot make the walk loop or visit more nodes
        // than were in the scoped tree.
        let mut remaining = self.nodes.len();
        while let Some(children) = stack.last_mut() {
            let Some(child) = children.next() else {
                let _finished = stack.pop();
                continue;
            };
            let Some(rest) = remaining.checked_sub(1) else { return false };
            remaining = rest;
            let Some(id) = child.text() else { return false };
            let Some(child) = self.nodes.get(id.as_ref()).copied() else { return false };
            match role(child).as_deref() {
                Some(b"StaticText") => {
                    if !ignored(child) {
                        let Some(words) = full_name(child) else { return false };
                        let Some(rest) = text.strip_prefix(words.bytes()) else { return false };
                        text = rest;
                    }
                }
                Some(b"InlineTextBox") => {}
                Some(_) | None => {
                    if let Some(children) = child_ids(child) {
                        stack.push(children);
                    }
                }
            }
        }
        text.is_empty()
    }
}

fn node_id(node: Value<'_>) -> Option<Box<[u8]>> {
    node.get(b"nodeId")?.text()
}

fn child_ids(node: Value<'_>) -> Option<Array<'_>> {
    node.get(b"childIds")?.array()
}

fn role(node: Value<'_>) -> Option<Box<[u8]>> {
    node.get(b"role")?.get(b"value")?.text()
}

fn ignored(node: Value<'_>) -> bool {
    match node.get(b"ignored") {
        Some(ignored) => ignored.bool() == Some(true),
        None => false,
    }
}

fn full_name(node: Value<'_>) -> Option<Text<'_>> {
    node.get(b"name")?.get(b"value")?.decoded_text()
}

/// The box's right and bottom edges are outside it.
pub(crate) fn point_in(rect: Rect, at: Point) -> bool {
    u64::from(at.x) < rect.width && u64::from(at.y) < rect.height
}

pub(crate) fn position(rect: Rect, at: Option<Point>) -> Option<(i64, i64)> {
    if rect.width == 0 || rect.height == 0 {
        return None;
    }
    let (x, y) = match at {
        Some(point) => {
            if !point_in(rect, point) {
                return None;
            }
            (i64::from(point.x), i64::from(point.y))
        }
        None => (i64::try_from(rect.width / 2).unwrap_or(i64::MAX), i64::try_from(rect.height / 2).unwrap_or(i64::MAX)),
    };
    Some((rect.x.saturating_add(x), rect.y.saturating_add(y)))
}

#[cfg(test)]
mod tests {
    use super::{Paragraphs, node_id, words};
    use crate::wire::decode::Document;

    const CASES: &[(&[u8], bool)] = &[(b"3 failing", true), (b"3 passed", false), (b"Failing", false), (b"", true)];

    #[test]
    fn paragraph_matching_uses_its_whole_visible_text_not_its_name() {
        let bytes = br#"[
          {"nodeId":"1","role":{"value":"paragraph"},"name":{"value":""},"childIds":["2","3"]},
          {"nodeId":"2","role":{"value":"StaticText"},"name":{"value":"Read "},"childIds":["4"]},
          {"nodeId":"3","role":{"value":"link"},"name":{"value":"Link label"},"childIds":["5"]},
          {"nodeId":"4","role":{"value":"InlineTextBox"},"name":{"value":"Read "}},
          {"nodeId":"5","role":{"value":"StaticText"},"name":{"value":"this."}},
          {"nodeId":"6","role":{"value":"paragraph"},"name":{"value":"Read this."},"childIds":["7"]},
          {"nodeId":"7","role":{"value":"StaticText"},"name":{"value":"Read this. Later."}},
          {"nodeId":"8","role":{"value":"generic"},"childIds":["9"]},
          {"nodeId":"9","role":{"value":"StaticText"},"name":{"value":"Read this."}},
          {"nodeId":"10","role":{"value":"paragraph"},"childIds":["11"]},
          {"nodeId":"11","role":{"value":"StaticText"},"name":{"value":"Read this."}}
        ]"#;
        let document = Document::parse(bytes, 2048).expect("scoped AX reply");
        let nodes = document.root().array().expect("AX nodes");
        let paragraphs = Paragraphs::new(nodes.clone());
        let mut found = skein_lib::List::with_capacity(2);
        for node in nodes {
            if paragraphs.matches(node, b"Read this.") {
                found.push(node_id(node).expect("id of matched node")).expect("two matches");
            }
        }
        assert_eq!(found.as_slice(), [Box::from(b"1".as_slice()), Box::from(b"10".as_slice())]);
    }

    #[test]
    fn words_match_a_name_holding_them_unbroken() {
        for (query, expected) in CASES {
            assert_eq!(words(b"Checks: 3 failing, 2 passed", query), *expected, "query {query:?}");
        }
    }
}
