use skein_lib::{List, Writer};

#[must_use]
pub fn text(tree: &skein_ui::Tree) -> Box<[u8]> {
    nodes_text(tree.nodes())
}
/// The text of a node and its descendants, in tree order.
#[must_use]
pub fn node_text(tree: &skein_ui::Tree, index: usize) -> Box<[u8]> {
    let end = index
        .checked_add(usize::try_from(tree.nodes()[index].size).expect("fixture subtree fits"))
        .expect("fixture index fits");
    nodes_text(&tree.nodes()[index..end])
}
#[must_use]
pub fn nodes_text(nodes: &[skein_ui::Node]) -> Box<[u8]> {
    let mut bytes = 0_usize;
    for node in nodes {
        for run in &node.text {
            bytes = bytes.checked_add(run.text.len()).expect("fixture text fits");
        }
    }
    let mut writer = Writer::new(bytes);
    for node in nodes {
        for run in &node.text {
            writer.put(&run.text).expect("text counted");
        }
    }
    writer.finish()
}
/// Type the number from the last terminal print, checking the complete control line.
#[must_use]
pub fn terminal_command(
    tree: &skein_ui::Tree,
    limits: &skein_ui::Limits,
    control: skein_ui::NodeId,
    words: &[u8],
    context: &[u8],
) -> (skein_ui::NodeId, skein_ui::Act) {
    let mut print = skein_ui_text::Print::new(limits).expect("terminal print bounds");
    skein_ui_text::render(tree, b"temper", None, &mut print);
    assert!(skein_lib::bytes::find(print.text.as_slice(), context).is_some(), "control context stays visible");
    let mut number = None;
    for (index, node) in print.numbers.as_slice().iter().enumerate() {
        if *node == control {
            number = Some(u64::try_from(index).unwrap().checked_add(1).unwrap());
        }
    }
    let number = skein_lib::Decimal::of(number.expect("control offers a terminal number"));
    let line = crate::words::accounts::join(&[b"[", number.as_bytes(), b"] ", words, b"\n"]);
    assert!(skein_lib::bytes::find(print.text.as_slice(), &line).is_some(), "numbered control line");
    let skein_ui_text::Command::Act { node, first, .. } =
        skein_ui_text::command(number.as_bytes(), Some(print.numbers.as_slice()), tree, limits.text_per_node)
    else {
        panic!("printed number activates its control")
    };
    assert_eq!(node, control);
    (node, first)
}
#[must_use]
pub fn find(tree: &skein_ui::Tree, role: skein_ui::Role, name: &[u8]) -> Option<skein_ui::NodeId> {
    for node in tree.nodes() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return Some(node.id);
        }
    }
    None
}
#[must_use]
pub fn person_find(tree: &skein_ui::Tree, role: skein_ui::Role, name: &[u8]) -> Option<skein_ui::NodeId> {
    use skein_fake_person::{Face, Find, Found, TreeFace};
    match TreeFace::new(tree).find(&Find::exactly_named(role, name)) {
        Found::Node(id) => Some(id),
        Found::Absent | Found::Pending => None,
    }
}
#[must_use]
pub fn frame_statuses(tree: &skein_ui::Tree) -> Box<[Box<[u8]>]> {
    let mut names = List::with_capacity(tree.len());
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&crate::look::TOP) || node.styles.as_slice().contains(&crate::look::RAIL) {
            let end = index.checked_add(usize::try_from(node.size).expect("banner fits")).expect("banner end fits");
            for child in &tree.nodes()[index..end] {
                if child.role == skein_ui::Role::Status
                    && let Some(name) = &child.name
                {
                    names.push(skein_lib::bytes::copy_of(name)).expect("tree bounds statuses");
                }
            }
        }
    }
    names.into_boxed()
}

#[must_use]
pub fn main_text(tree: &skein_ui::Tree) -> Box<[u8]> {
    let mut main = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == skein_ui::Role::Main {
            let end = index
                .checked_add(usize::try_from(node.size).expect("fixture subtree fits"))
                .expect("fixture index fits");
            main = tree.nodes().get(index..end);
        }
    }
    let main = main.expect("fixture main");
    let mut bytes = 0_usize;
    for node in main {
        for run in &node.text {
            bytes = bytes.checked_add(run.text.len()).expect("fixture text fits");
        }
    }
    let mut writer = Writer::new(bytes);
    for node in main {
        for run in &node.text {
            writer.put(&run.text).expect("main text counted");
        }
    }
    writer.finish()
}

/// The styles of each node of `role`, in tree order.
#[must_use]
pub fn styles(tree: &skein_ui::Tree, role: skein_ui::Role) -> Box<[Box<[skein_ui::Style]>]> {
    let mut found = List::with_capacity(u32::try_from(tree.nodes().len()).expect("nodes fit"));
    for node in tree.nodes() {
        if node.role == role {
            found.push(Box::from(node.styles.as_slice())).expect("a node per style list");
        }
    }
    found.into_boxed()
}
