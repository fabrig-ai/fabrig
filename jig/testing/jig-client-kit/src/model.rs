//! A platform tree changed only by patches (skein's ui/README.md, 7).
use skein_ui::{Made, NodeId, Patch, Tree};
#[derive(Debug)]
struct Entry {
    node: Made,
    parent: NodeId,
    edited: bool,
}

fn entries(nodes: Box<[Made]>, parent: NodeId) -> Result<Vec<Entry>, String> {
    let mut rows = Vec::with_capacity(nodes.len());
    let mut stack: Vec<(NodeId, u32)> = Vec::new();
    for (index, node) in nodes.into_vec().into_iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| "small test tree")?;
        while stack.last().is_some_and(|(_, end)| index >= *end) {
            stack.pop();
        }
        let parent = stack.last().map_or(parent, |(id, _)| *id);
        stack.push((node.id, index.checked_add(node.size).ok_or("small subtree")?));
        rows.push(Entry { node, parent, edited: false });
    }
    Ok(rows)
}

fn position(rows: &[Entry], id: NodeId) -> Result<usize, String> {
    rows.iter().position(|row| row.node.id == id).ok_or_else(|| "patch names a present node".into())
}

fn sizes(rows: &mut [Entry]) -> Result<(), String> {
    // Preorder makes every subtree contiguous. Close each ancestor once,
    // when its next sibling begins, rather than scanning every child.
    let mut open: Vec<usize> = Vec::new();
    for index in 0..rows.len() {
        while open.last().is_some_and(|parent| rows[*parent].node.id != rows[index].parent) {
            let parent = open.pop().ok_or("open ancestor")?;
            rows[parent].node.size = u32::try_from(index - parent).map_err(|_| "small subtree")?;
        }
        if rows[index].parent != NodeId(0) && open.is_empty() {
            return Err("preorder parent is present".into());
        }
        open.push(index);
    }
    while let Some(parent) = open.pop() {
        rows[parent].node.size = u32::try_from(rows.len() - parent).map_err(|_| "small subtree")?;
    }
    Ok(())
}

fn insertion(rows: &[Entry], parent: NodeId, before: Option<NodeId>) -> Result<usize, String> {
    match before {
        Some(id) => position(rows, id),
        None => {
            if parent == NodeId(0) {
                Ok(rows.len())
            } else {
                let at = position(rows, parent)?;
                let end = at.checked_add(skein_lib::length::usize_of(rows[at].node.size)).ok_or("small tree")?;
                rows.get(..end).ok_or("small tree")?;
                Ok(end)
            }
        }
    }
}

fn apply(rows: &mut Vec<Entry>, patch: Patch) -> Result<(), String> {
    match patch {
        Patch::Insert { parent, before, nodes } => {
            let at = insertion(rows, parent, before)?;
            drop(rows.splice(at..at, entries(nodes, parent)?));
            sizes(rows)?;
        }
        Patch::Remove { node } => {
            let at = position(rows, node)?;
            let end = at.checked_add(skein_lib::length::usize_of(rows[at].node.size)).ok_or("small tree")?;
            rows.get(at..end).ok_or("small tree")?;
            drop(rows.drain(at..end));
            sizes(rows)?;
        }
        Patch::Move { node, parent, before } => {
            let at = position(rows, node)?;
            let end = at.checked_add(skein_lib::length::usize_of(rows[at].node.size)).ok_or("small tree")?;
            rows.get(at..end).ok_or("small tree")?;
            let mut moved: Vec<_> = rows.drain(at..end).collect();
            moved.first_mut().ok_or("small subtree")?.parent = parent;
            sizes(rows)?;
            let to = insertion(rows, parent, before)?;
            drop(rows.splice(to..to, moved));
            sizes(rows)?;
        }
        Patch::Text { node, text } => {
            let at = position(rows, node)?;
            rows[at].node.text = text;
        }
        Patch::Attribute { node, attribute } => {
            let at = position(rows, node)?;
            rows[at].node.attribute = *attribute;
        }
        Patch::Value { node, value } => {
            let at = position(rows, node)?;
            if rows[at].node.value.as_ref().map(|field| field.written) == value.as_ref().map(|field| field.written) {
                return Err("a value patch changes the write count".into());
            }
            rows[at].node.value = value;
            rows[at].edited = false;
        }
    }
    Ok(())
}

fn equal(rows: &[Entry], tree: &Tree) -> Result<(), String> {
    if rows.len() != skein_lib::length::usize_of(tree.len()) {
        return Err("platform-visible node count".into());
    }
    for (row, node) in rows.iter().zip(tree.nodes().iter()) {
        let made = &row.node;
        if (made.id, made.size, made.value.as_ref().map(|field| field.written))
            != (node.id, node.size, node.value.as_ref().map(|field| field.written))
        {
            return Err("platform-visible node".into());
        }
        if !row.edited && made.value != node.value {
            return Err("platform-visible field text".into());
        }
        let attribute = &made.attribute;
        if (attribute.role, &attribute.name, attribute.linked, &attribute.external, attribute.bound, &attribute.drawing)
            != (
                node.role,
                &node.name,
                node.link.is_some(),
                &node.external,
                node.binding.is_some() || node.link.is_some(),
                &node.drawing,
            )
        {
            return Err("platform-visible attributes".into());
        }
        if !(attribute.states.iter().eq(node.states.iter())) {
            return Err("platform-visible states".into());
        }
        if !(attribute.styles.iter().eq(node.styles.iter())) {
            return Err("platform-visible styles".into());
        }
        if !(made.text.iter().eq(node.text.iter())) {
            return Err("platform-visible text runs".into());
        }
    }
    Ok(())
}

/// The platform’s tree, independent of the renderer’s retained tree.
#[derive(Debug, Default)]
pub struct Model {
    rows: Vec<Entry>,
}
impl Model {
    /// Mirrors the platform’s local field edit.
    pub fn input(&mut self, node: NodeId, text: &[u8]) {
        let at = position(&self.rows, node).expect("patch names a present node");
        if let Some(value) = self.rows[at].node.value.as_mut() {
            value.text = Box::from(text);
            self.rows[at].edited = true;
        }
    }
    /// Applies one platform patch without rebuilding from the view.
    pub fn apply(&mut self, patch: Patch) {
        if let Err(why) = self.check_apply(patch) {
            panic!("{why}");
        }
    }
    /// Returns a finding if a platform patch breaks the model.
    /// After a finding, the model may contain a partially applied patch.
    pub fn check_apply(&mut self, patch: Patch) -> Result<(), String> {
        apply(&mut self.rows, patch)
    }
    /// Checks the complete platform projection against the view.
    pub fn check(&self, tree: &Tree) {
        if let Err(why) = equal(&self.rows, tree) {
            panic!("{why}");
        }
    }
    /// Node identities in the platform projection before a changed page.
    pub fn ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.rows.iter().map(|row| row.node.id)
    }
    /// Returns the platform projection for an external observer.
    #[must_use]
    pub fn nodes(&self) -> Vec<Made> {
        self.rows.iter().map(|row| row.node.clone()).collect()
    }
}

#[test]
fn local_typing_survives_unchanged_writes_and_same_count_value_patches_are_rejected() {
    let limits =
        skein_ui::Limits { nodes: 1, depth: 1, text_per_node: 8, text_bytes: 8, patches: 8, ..skein_ui::Limits::EMPTY };
    let mut view = skein_ui::View::new(&limits);
    let mut patches = skein_lib::Queue::with_capacity(limits.patches);
    let mut builder = view.builder();
    builder.open(skein_ui::Role::TextField);
    builder.value(b"100", 89);
    builder.close();
    view.finish(builder, 1, &mut patches);
    let mut model = Model::default();
    while let Some(patch) = patches.pop() {
        model.apply(patch);
    }
    let node = view.tree().get(0).expect("field").id;
    model.input(node, b"**h");
    model.apply(Patch::Text { node, text: Box::default() });
    model.check(view.tree());
    assert_eq!(model.rows[0].node.value.as_ref().expect("field").text.as_ref(), b"**h");
    let same_count = Patch::Value { node, value: Some(skein_ui::Value { text: Box::from(*b"100"), written: 89 }) };
    assert_eq!(
        model.check_apply(same_count).expect_err("same-count value patch"),
        "a value patch changes the write count"
    );
}
