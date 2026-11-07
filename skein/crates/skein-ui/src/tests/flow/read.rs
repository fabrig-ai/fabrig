//! Reading flows and refusing blocks past the subset (skein/docs/design/ui.md, 10.1).
#![expect(clippy::disallowed_types, reason = "ordinary tests collect decoded nodes and Markdown text")]
#![expect(clippy::disallowed_methods, reason = "ordinary tests collect semantic projections")]
#![expect(clippy::disallowed_macros, reason = "acceptance cases are compiled from their unchanged files")]

use crate::flow::{self, Direction, Node};
use crate::{Builder, Limits, Role, Run, Tree, markdown};

fn limits(input: &[u8], drawing_bytes: u32) -> Limits {
    let bytes = u32::try_from(input.len()).unwrap();
    Limits {
        nodes: markdown::max_nodes(bytes).unwrap().checked_add(2).unwrap(),
        depth: 12,
        text_per_node: bytes.checked_mul(2).unwrap().checked_add(1).unwrap(),
        text_bytes: markdown::max_text_bytes(bytes).unwrap().checked_add(1).unwrap(),
        markdown_depth: 4,
        drawing_bytes,
        ..crate::tests::fixtures::LIMITS
    }
}

fn read(input: &[u8], drawing_bytes: u32, growing: bool) -> Tree {
    let limits = limits(input, drawing_bytes);
    let mut builder = Builder::new(&limits);
    builder.open(Role::Page);
    if growing {
        markdown::read_growing(input, 2, |builder| builder.text(Run::Plain, b"~"), &mut builder);
    } else {
        markdown::read(input, 2, &mut builder);
    }
    builder.close();
    let tree = builder.finish();
    assert!(tree.len() <= limits.nodes);
    assert!(tree.nodes().iter().map(crate::Node::text_bytes).sum::<u32>() <= limits.text_bytes);
    tree
}

fn text(tree: &Tree) -> Vec<u8> {
    tree.nodes().iter().flat_map(|node| node.text.iter()).flat_map(|run| run.text.iter().copied()).collect()
}

fn block(body: &str) -> Vec<u8> {
    format!("```mermaid\n{body}\n```\n").into_bytes()
}

fn assert_code(input: &[u8], body: &[u8], drawing_bytes: u32) {
    let tree = read(input, drawing_bytes, false);
    assert_eq!(tree.nodes().iter().map(|node| node.role).collect::<Vec<_>>(), [Role::Page, Role::Code, Role::Text]);
    assert!(tree.nodes().iter().all(|node| node.drawing.is_none()));
    assert_eq!(text(&tree), body);
    assert_eq!(tree.nodes()[2].text.get(0).unwrap().run, Run::Code);
}

#[test]
fn a_block_in_the_subset_reads_as_its_flow() {
    let cases: &[(&str, Direction, &[Node<'_>])] = &[
        ("graph LR\na-->b", Direction::LeftToRight, &[node(b"a"), node(b"b")]),
        ("flowchart TD\na --> b", Direction::TopToBottom, &[node(b"a"), node(b"b")]),
        ("flowchart TB\na --> b", Direction::TopToBottom, &[node(b"a"), node(b"b")]),
        ("graph LR\na[ first ] --> b[\" second \"]", Direction::LeftToRight, &[node(b"first"), node(b"second")]),
        (
            "graph LR\na[A<br> B] --> b[\"C<br/> D\"]:::strong",
            Direction::LeftToRight,
            &[
                Node { name: b"A", caption: Some(b"B"), strong: false },
                Node { name: b"C", caption: Some(b"D"), strong: true },
            ],
        ),
        ("graph LR\na-->b-->c", Direction::LeftToRight, &[node(b"a"), node(b"b"), node(b"c")]),
        ("graph LR\na --> b\nb --> c", Direction::LeftToRight, &[node(b"a"), node(b"b"), node(b"c")]),
        ("graph LR\nb --> c\na --> b", Direction::LeftToRight, &[node(b"a"), node(b"b"), node(b"c")]),
        (
            "graph LR\nb[Label]:::strong\na --> b",
            Direction::LeftToRight,
            &[node(b"a"), Node { name: b"Label", caption: None, strong: true }],
        ),
        (
            "\n%% comment\ngraph LR\n\t\nclassDef strong anything\naccDescr: ignored\n%% comment\na-->b",
            Direction::LeftToRight,
            &[node(b"a"), node(b"b")],
        ),
        ("graph LR\na_1-->b-2-->c--3", Direction::LeftToRight, &[node(b"a_1"), node(b"b-2"), node(b"c--3")]),
        ("graph LR\r\n\ta --> b \t\r", Direction::LeftToRight, &[node(b"a"), node(b"b")]),
        ("graph LR\na[\"a]b\"]-->b", Direction::LeftToRight, &[node(b"a]b"), node(b"b")]),
        ("graph LR\na[a\"b]-->b", Direction::LeftToRight, &[node(b"a\"b"), node(b"b")]),
    ];
    for (body, direction, nodes) in cases {
        let input = block(body);
        let tree = read(&input, 4096, false);
        assert_eq!(tree.len(), 4, "{body}");
        assert_eq!(tree.nodes()[1].role, Role::Drawing, "{body}");
        assert_eq!(tree.nodes()[2].role, Role::Paragraph, "{body}");
        assert_eq!(tree.nodes()[3].role, Role::Text, "{body}");
        let drawing = tree.nodes()[1].drawing.as_ref().unwrap();
        assert_eq!(drawing.kind, flow::KIND);
        let (found, decoded) = flow::decode(&drawing.data).unwrap();
        assert_eq!(found, *direction, "{body}");
        assert_eq!(decoded.collect::<Vec<_>>(), *nodes, "{body}");
    }
    for info in [b"``` mermaid \t\n".as_slice(), b"```\tmermaid\t\r\n"] {
        assert!(flow::read::read(info, b"graph LR\na-->b", &limits(info, 64)).is_some());
    }
}

const fn node(name: &[u8]) -> Node<'_> {
    Node { name, caption: None, strong: false }
}

#[test]
fn a_block_past_the_subset_stays_code() {
    for (body, why) in [
        ("graph LR\na-->b\na-->c", "branch"),
        ("graph LR\na-->c\nb-->c", "merge"),
        ("graph LR\na-->b-->a", "cycle"),
        ("graph LR\na-->a", "self edge"),
        ("graph LR\na-->b\nc-->d", "two paths"),
        ("graph LR\na-->b\nc", "node on no edge"),
        ("graph LR\na", "no edge"),
        ("graph LR\na-->|label|b", "edge label"),
        ("graph LR\na---b", "undirected edge"),
        ("graph LR\na==>b", "another arrow"),
        ("graph LR\na(round)-->b", "round"),
        ("graph LR\na{diamond}-->b", "diamond"),
        ("graph LR\nsubgraph group\na-->b", "subgraph"),
        ("graph LR\na-->b\nstyle a color:red", "style"),
        ("graph LR\na-->b\nclick a handler", "click"),
        ("graph LR\na:::other-->b", "other class"),
        ("graph LR\na[A<br>B<br/>C]-->b", "third label line"),
        ("graph LR\naccTitle: One\naccTitle: Two\na-->b", "second title"),
        ("graph LR\na[A]-->b\na[A]", "labelled twice"),
        ("graph LR\na[]-->b", "empty label"),
        ("graph LR\na[\"\"]-->b", "empty quoted label"),
        ("graph LR\na[ <br>B]-->b", "empty name"),
        ("graph LR\na[A<br> ]-->b", "empty caption"),
        ("graph LR\na[<b>A</b>]-->b", "HTML label"),
        ("flowchart RL\na-->b", "other direction"),
        ("a-->b", "no first line"),
        ("graph LR\naccTitle: \t\na-->b", "empty title"),
        ("graph LR\na[\"A\"B\"]-->b", "unclosed quoted label"),
        ("graph LR\na[A\x01B]-->b", "control character in plain text"),
        ("graph LR\na-->", "no edge target"),
    ] {
        let input = block(body);
        assert!(flow::read::read(b"```mermaid", body.as_bytes(), &limits(&input, 4096)).is_none(), "{why}");
        assert_code(&input, format!("{body}\n").as_bytes(), 4096);
    }
    for info in ["Mermaid", "mermaid x", "mermaid`", "```mermaid"] {
        let input = format!("```{info}\ngraph LR\na-->b\n```\n");
        assert_code(input.as_bytes(), b"graph LR\na-->b\n", 4096);
    }
}

#[test]
fn the_cases_read_as_they_say() {
    let cases: &[(&[u8], &str, Direction, &[Node<'_>])] = &[
        (
            include_bytes!("../../../../../docs/design/draft/diagrams/how-hello-works.md"),
            include_str!("../../../../../docs/design/draft/diagrams/how-hello-works.fallback.txt"),
            Direction::LeftToRight,
            &[
                Node { name: b"Design draft", caption: Some(b"you write it"), strong: false },
                Node { name: b"Goal", caption: Some(b"you start it"), strong: false },
                Node { name: b"Checked changes", caption: Some(b"CI and a review"), strong: false },
                Node { name: b"main", caption: Some(b"the merge queue"), strong: true },
            ],
        ),
        (
            include_bytes!("../../../../../docs/design/draft/diagrams/review.md"),
            include_str!("../../../../../docs/design/draft/diagrams/review.fallback.txt"),
            Direction::TopToBottom,
            &[
                node(b"A change pushed"),
                node(b"A review by an agent"),
                Node { name: b"Landed", caption: None, strong: true },
            ],
        ),
    ];
    for (input, fallback, direction, nodes) in cases {
        let tree = read(input, 4096, false);
        assert_eq!(
            tree.nodes().iter().map(|node| node.role).collect::<Vec<_>>(),
            [Role::Page, Role::Drawing, Role::Paragraph, Role::Text]
        );
        assert_eq!(tree.nodes()[1].size, 3);
        assert_eq!(tree.nodes()[2].size, 2);
        assert_eq!(tree.nodes()[3].text.len(), 1);
        let (name, words) = fallback.trim_end_matches('\n').split_once('\n').unwrap();
        assert_eq!(tree.nodes()[1].name.as_deref(), if name.is_empty() { None } else { Some(name.as_bytes()) });
        assert_eq!(text(&tree), words.as_bytes());
        let drawing = tree.nodes()[1].drawing.as_ref().unwrap();
        assert_eq!(drawing.kind, flow::KIND);
        let (found, decoded) = flow::decode(&drawing.data).unwrap();
        assert_eq!(found, *direction);
        assert_eq!(decoded.collect::<Vec<_>>(), *nodes);
    }
    let input = include_bytes!("../../../../../docs/design/draft/diagrams/stays-code.md");
    let body = input.strip_prefix(b"```mermaid\n").unwrap().strip_suffix(b"```\n").unwrap();
    assert_code(input, body, 4096);
    assert_eq!(text(&read(input, 4096, false)), text(&read(&[b"```\n", body, b"```\n"].concat(), 4096, false)));
}

#[test]
fn a_flow_past_its_bounds_stays_code() {
    let body = b"graph LR\na-->b";
    for (drawing_bytes, why) in [(0, "no drawing bytes"), (8, "one less than data"), (7, "nodes past a quarter")] {
        let input = block(core::str::from_utf8(body).unwrap());
        assert!(flow::read::read(b"```mermaid", body, &limits(&input, drawing_bytes)).is_none(), "{why}");
        assert_code(&input, &[body.as_slice(), b"\n"].concat(), drawing_bytes);
    }
    for (body, text_per_node, why) in [
        (b"graph LR\na-->b".as_slice(), 9, "fallback one byte too long"),
        (b"graph LR\naccTitle: A long title\na-->b", 11, "title one byte too long"),
    ] {
        let mut limits = limits(body, 4096);
        limits.text_per_node = text_per_node;
        assert!(flow::read::read(b"```mermaid", body, &limits).is_none(), "{why}");
        // Fenced code needs the caller's ordinary Markdown text allowance.
        assert_code(&block(core::str::from_utf8(body).unwrap()), &[body, b"\n"].concat(), 0);
    }
    let mut exact = limits(body, 9);
    exact.text_per_node = 10;
    let flow = flow::read::read(b"```mermaid", body, &exact).unwrap();
    assert_eq!(flow.data.len(), 9);
    assert_eq!(flow.fallback.as_slice(), b"a; then b.");
    let path = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let chained = path
        .iter()
        .map(|byte| core::str::from_utf8(core::slice::from_ref(byte)).unwrap())
        .collect::<Vec<_>>()
        .join("-->");
    let input = block(&format!("graph LR\n{chained}"));
    let tree = read(&input, 4096, false);
    assert_eq!(tree.nodes()[1].role, Role::Drawing);
    assert!(tree.len() <= markdown::max_nodes(u32::try_from(input.len()).unwrap()).unwrap().checked_add(1).unwrap());
    assert!(
        tree.nodes().iter().map(crate::Node::text_bytes).sum::<u32>()
            <= markdown::max_text_bytes(u32::try_from(input.len()).unwrap()).unwrap()
    );
}

#[test]
fn a_flow_being_written() {
    let input = b"```mermaid\ngraph LR\na-->b\n";
    for (growing, role, words) in
        [(false, Role::Paragraph, input.as_slice()), (true, Role::Code, b"graph LR\na-->b\n".as_slice())]
    {
        let tree = read(input, 4096, growing);
        assert_eq!(tree.nodes()[1].role, role);
        assert!(tree.nodes().iter().all(|node| node.drawing.is_none()));
        assert_eq!(text(&tree), if growing { [words, b"~"].concat() } else { words.to_vec() });
    }
    for suffix in [b"```".as_slice(), b"```\n", b"```\n\n"] {
        let closed = [input.as_slice(), suffix].concat();
        let tree = read(&closed, 4096, true);
        assert_eq!(tree.nodes()[1].role, Role::Drawing);
        assert_eq!(tree.nodes()[1].size, 4);
        assert_eq!(tree.nodes()[2].role, Role::Paragraph);
        assert_eq!(tree.nodes()[2].size, 3);
        assert_eq!(tree.nodes()[4].text.get(0).unwrap().text.as_ref(), b"~");
        assert_eq!(text(&tree), b"a; then b.~");
    }
}
