//! A platform tree changed only by patches (skein's ui/README.md, 7).
use skein_ui::{Made, NodeId, Patch, Tree};
#[derive(Clone, Debug)]
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

fn sizes(rows: &mut [Entry], root: NodeId) -> Result<(), String> {
    // Preorder makes every subtree contiguous. Close each ancestor once,
    // when its next sibling begins, rather than scanning every child.
    let mut open: Vec<usize> = Vec::new();
    for index in 0..rows.len() {
        while open.last().is_some_and(|parent| rows[*parent].node.id != rows[index].parent) {
            let parent = open.pop().ok_or("open ancestor")?;
            rows[parent].node.size = u32::try_from(index - parent).map_err(|_| "small subtree")?;
        }
        if rows[index].parent != root && open.is_empty() {
            return Err("preorder parent is present".into());
        }
        open.push(index);
    }
    while let Some(parent) = open.pop() {
        rows[parent].node.size = u32::try_from(rows.len() - parent).map_err(|_| "small subtree")?;
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
    // Sorted by identity, with one offset per live node: churn cannot retain retired identities.
    positions: Vec<(NodeId, u32)>,
}
impl Model {
    fn position(&self, id: NodeId) -> Result<usize, String> {
        self.positions
            .binary_search_by_key(&id, |(node, _)| *node)
            .map(|at| skein_lib::length::usize_of(self.positions[at].1))
            .map_err(|_| "patch names a present node".into())
    }
    fn insertion(&self, parent: NodeId, before: Option<NodeId>) -> Result<usize, String> {
        let end = if parent == NodeId(0) {
            self.rows.len()
        } else {
            let at = self.position(parent)?;
            at.checked_add(skein_lib::length::usize_of(self.rows[at].node.size)).ok_or("small tree")?
        };
        self.rows.get(..end).ok_or("small tree")?;
        let at = before.map_or(Ok(end), |id| self.position(id))?;
        // A child starts before a sibling, or at the end of its parent's subtree.
        if at != end && (at > end || self.rows[at].parent != parent) {
            return Err("preorder parent is present".into());
        }
        Ok(at)
    }
    fn resize_ancestors(&mut self, mut parent: NodeId, removed: u32, added: u32) -> Result<(), String> {
        while parent != NodeId(0) {
            let at = self.position(parent)?;
            let row = &mut self.rows[at];
            row.node.size =
                row.node.size.checked_sub(removed).and_then(|size| size.checked_add(added)).ok_or("small subtree")?;
            parent = row.parent;
        }
        Ok(())
    }
    fn insert(&mut self, parent: NodeId, before: Option<NodeId>, rows: Vec<Entry>) -> Result<(), String> {
        let at = self.insertion(parent, before)?;
        let count = u32::try_from(rows.len()).map_err(|_| "small subtree")?;
        self.resize_ancestors(parent, 0, count)?;
        let start = u32::try_from(at).map_err(|_| "small tree")?;
        for (_, position) in &mut self.positions {
            if *position >= start {
                *position = position.checked_add(count).ok_or("small tree")?;
            }
        }
        for (offset, row) in rows.iter().enumerate() {
            let position = u32::try_from(at + offset).map_err(|_| "small tree")?;
            let index = self.positions.binary_search_by_key(&row.node.id, |(node, _)| *node);
            match index {
                Ok(index) => self.positions[index].1 = position,
                Err(index) => self.positions.insert(index, (row.node.id, position)),
            }
        }
        drop(self.rows.splice(at..at, rows));
        Ok(())
    }
    fn remove(&mut self, node: NodeId) -> Result<Vec<Entry>, String> {
        let at = self.position(node)?;
        let size = self.rows[at].node.size;
        let end = at.checked_add(skein_lib::length::usize_of(size)).ok_or("small tree")?;
        self.rows.get(at..end).ok_or("small tree")?;
        self.resize_ancestors(self.rows[at].parent, size, 0)?;
        let removed: Vec<_> = self.rows.drain(at..end).collect();
        let start = u32::try_from(at).map_err(|_| "small tree")?;
        let end = u32::try_from(end).map_err(|_| "small tree")?;
        self.positions.retain_mut(|(_, position)| {
            if (start..end).contains(position) {
                return false;
            }
            if *position >= end {
                *position -= size;
            }
            true
        });
        Ok(removed)
    }
    /// Mirrors the platform’s local field edit.
    pub fn input(&mut self, node: NodeId, text: &[u8]) {
        let at = self.position(node).expect("patch names a present node");
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
        match patch {
            Patch::Insert { parent, before, nodes } => {
                let mut rows = entries(nodes, parent)?;
                sizes(&mut rows, parent)?;
                self.insert(parent, before, rows)?;
            }
            Patch::Remove { node } => {
                self.remove(node)?;
            }
            Patch::Move { node, parent, before } => {
                let mut rows = self.remove(node)?;
                rows.first_mut().ok_or("small subtree")?.parent = parent;
                self.insert(parent, before, rows)?;
            }
            Patch::Text { node, text } => {
                let at = self.position(node)?;
                self.rows[at].node.text = text;
            }
            Patch::Attribute { node, attribute } => {
                let at = self.position(node)?;
                self.rows[at].node.attribute = *attribute;
            }
            Patch::Value { node, value } => {
                let at = self.position(node)?;
                if self.rows[at].node.value.as_ref().map(|field| field.written)
                    == value.as_ref().map(|field| field.written)
                {
                    return Err("a value patch changes the write count".into());
                }
                self.rows[at].node.value = value;
                self.rows[at].edited = false;
            }
        }
        Ok(())
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
    /// Whether a node remains in the platform projection after a patch.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.positions.binary_search_by_key(&node, |(id, _)| *id).is_ok()
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

#[test]
fn indexed_positions_and_incremental_sizes_agree_with_scans_after_each_patch() {
    use skein_ui::{Key, Owner, Role, Run};
    let limits = skein_ui::Limits {
        nodes: 24,
        depth: 6,
        text_per_node: 32,
        text_bytes: 256,
        patches: 64,
        ..skein_ui::Limits::EMPTY
    };
    let key = |a| Key { owner: Owner { number: 1 }, kind: 1, a, b: 0 };
    let mut view = skein_ui::View::new(&limits);
    let mut patches = skein_lib::Queue::with_capacity(limits.patches);
    let mut model = Model::default();
    for (revision, groups, items) in
        [(0, [1, 2], vec![10, 11]), (1, [1, 2], vec![11, 12, 10]), (2, [2, 1], vec![12, 10])]
    {
        let mut builder = view.builder();
        builder.open(Role::Page);
        builder.open(Role::List);
        for group in groups {
            builder.open(Role::Item);
            builder.key(key(group));
            builder.name(if revision == 0 { b"before" } else { b"after" });
            if group == 1 {
                builder.open(Role::List);
                for item in &items {
                    builder.open(Role::Item);
                    builder.key(key(*item));
                    builder.open(Role::TextField);
                    builder.value(if revision == 0 { b"old" } else { b"new" }, revision);
                    builder.close();
                    builder.close();
                }
                builder.close();
            } else {
                builder.open(Role::Text);
                builder.run(Run::Plain, if revision == 0 { b"before" } else { b"after" });
                builder.close();
            }
            builder.close();
        }
        builder.close();
        builder.close();
        view.finish(builder, revision, &mut patches);
        while let Some(patch) = patches.pop() {
            model.apply(patch);
            assert_eq!(model.positions.len(), model.rows.len(), "no retired identity remains indexed");
            for row in &model.rows {
                assert_eq!(
                    model.position(row.node.id).ok(),
                    model.rows.iter().position(|candidate| candidate.node.id == row.node.id),
                    "position of {:?}",
                    row.node.id,
                );
            }
            let mut recomputed = model.rows.clone();
            sizes(&mut recomputed, NodeId(0)).expect("whole preorder");
            assert_eq!(
                model.rows.iter().map(|row| row.node.size).collect::<Vec<_>>(),
                recomputed.iter().map(|row| row.node.size).collect::<Vec<_>>(),
                "incremental subtree sizes",
            );
        }
        model.check(view.tree());
    }
}
