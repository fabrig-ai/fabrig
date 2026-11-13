#![expect(clippy::disallowed_types, reason = "ordinary test code uses collected text and link projections")]
#![expect(clippy::disallowed_macros, reason = "checked-in report passages are compile-time test fixtures")]
#![expect(clippy::disallowed_methods, reason = "ordinary test code collects semantic text for assertions")]

use crate::{Builder, NodeId, Patch, Role, Run, Tree, View, markdown};
use skein_lib::Queue;

fn read(input: &[u8], nesting: u32) -> Tree {
    let bytes = u32::try_from(input.len()).unwrap();
    let limits = crate::Limits {
        nodes: markdown::max_nodes(bytes).unwrap().checked_add(1).unwrap(),
        depth: nesting.checked_mul(2).unwrap().checked_add(2).unwrap().max(5).checked_add(1).unwrap(),
        text_per_node: bytes.checked_mul(2).unwrap(),
        text_bytes: markdown::max_text_bytes(bytes).unwrap(),
        markdown_depth: nesting,
        ..crate::tests::fixtures::LIMITS
    };
    let mut builder = Builder::new(&limits);
    builder.open(Role::Page);
    markdown::read(input, 2, &mut builder);
    builder.close();
    let tree = builder.finish();
    assert!(tree.len() <= limits.nodes);
    tree
}
fn text(tree: &Tree) -> Vec<u8> {
    tree.nodes().iter().flat_map(|node| node.text.as_slice()).flat_map(|run| run.text.iter().copied()).collect()
}

#[test]
fn paragraphs_blocks_runs_links_and_literal_fallbacks() {
    let input = b"# Title\nparagraph *soft* **strong** _under_ __double__ `code`\nnext\n\n- first\n  - nested\n- last\n\n1. numbered\n2. next\n\n> quoted\n\n```text\nraw *stars*\n```\n\n[visit](https://example.org/a) [plain](http://example.org)\n";
    let tree = read(input, 3);
    for role in
        [Role::Heading { level: 3 }, Role::Paragraph, Role::List, Role::Item, Role::Quote, Role::Code, Role::Link]
    {
        assert!(tree.nodes().iter().any(|node| node.role == role), "{role:?}");
    }
    for run in [Run::Plain, Run::Emphasis, Run::Strong, Run::Code] {
        assert!(tree.nodes().iter().any(|node| node.text.as_slice().iter().any(|text| text.run == run)));
    }
    assert!(has_run(&tree, Run::Emphasis, b"under"));
    assert!(has_run(&tree, Run::Strong, b"double"));
    for (input, run, words) in [
        (b"_soft_".as_slice(), Run::Emphasis, b"soft".as_slice()),
        (b"__strong__", Run::Strong, b"strong"),
        (b"_Draft, with the user._", Run::Emphasis, b"Draft, with the user."),
        (b"*a_b*", Run::Emphasis, b"a_b"),
        (b"un*frigging*believable", Run::Emphasis, b"frigging"),
    ] {
        assert!(has_run(&read(input, 3), run, words), "{input:?}");
    }
    let nested = read(b"*a _b_ c*", 3);
    for words in [b"a ".as_slice(), b"b", b" c"] {
        assert!(has_run(&nested, Run::Emphasis, words), "{}", shape(&nested));
    }
    let links: Vec<_> = tree.nodes().iter().filter_map(|node| node.external.as_deref()).collect();
    assert_eq!(links, [b"https://example.org/a".as_slice(), b"http://example.org".as_slice()]);
    assert!(text(&tree).windows(5).any(|bytes| bytes == b"\nnext"));
    for (literal, expected) in [
        (b"<b>*raw*</b>".as_slice(), b"<b>*raw*</b>".as_slice()),
        (b"| **table** | cell |", b"| **table** | cell |"),
        (b"**table** | cell", b"**table** | cell"),
        (b"| **table** | cell |\n| -- |\n", b"| **table** | cell |\n| -- |"),
        (b"before <b>*raw*</b>", b"before <b>*raw*</b>"),
        (b"![*image*](https://example.org)", b"![*image*](https://example.org)"),
        (b"[*unsafe*](javascript:alert)", b"[*unsafe*](javascript:alert)"),
        (b"[open](https://example.org", b"[open](https://example.org"),
        (b"*never closed", b"*never closed"),
        (b"snake_case and read_branch", b"snake_case and read_branch"),
        (b"a__b", b"a__b"),
        (b"1_2 and 3_4", b"1_2 and 3_4"),
        ("é_字 and 字_é".as_bytes(), "é_字 and 字_é".as_bytes()),
        (b"*soft_", b"*soft_"),
        (b"_soft*", b"_soft*"),
        (b"**strong__", b"**strong__"),
        (b"__strong**", b"__strong**"),
        (b"`[inside](https://example.org)`", b"[inside](https://example.org)"),
        (b"```\nunclosed *fence*\n", b"```\nunclosed *fence*\n"),
        (b"        - **too deep**", b"        - **too deep**"),
        (b"[]()", b"[]()"),
        (b"[](https://example.org)", b"[](https://example.org)"),
        (b"[x](https://)", b"[x](https://)"),
    ] {
        let tree = read(literal, 2);
        assert_eq!(text(&tree), expected, "{literal:?}");
        assert!(tree.nodes().iter().all(|node| node.external.is_none()));
    }
    let mut view = View::new(&crate::Limits {
        nodes: 8,
        depth: 4,
        text_per_node: 64,
        text_bytes: 128,
        ..crate::tests::fixtures::LIMITS
    });
    let mut out = Queue::with_capacity(crate::tests::fixtures::LIMITS.patches);
    let mut link_id = NodeId(0);
    for (shown, input) in
        [(1, b"[label](https://example.org/a)".as_slice()), (2, b"[label](https://example.org/b)".as_slice())]
    {
        let mut builder = view.builder();
        builder.open(Role::Page);
        markdown::read(input, 0, &mut builder);
        builder.close();
        view.finish(builder, shown, &mut out);
        let id = view.tree().nodes().iter().find(|node| node.role == Role::Link).unwrap().id;
        if shown == 1 {
            link_id = id;
        } else {
            assert_eq!(id, link_id);
            assert_eq!(view.external(id), Some(b"https://example.org/b".as_slice()));
            if let Some(Patch::Attribute { node, attribute }) = out.pop() {
                assert_eq!(node, id);
                assert_eq!(attribute.external.as_deref(), Some(b"https://example.org/b".as_slice()));
            } else {
                panic!("changed target has an attribute patch");
            }
            assert!(out.is_empty());
        }
        out.clear();
    }
    for unsafe_url in [
        b"https:///path".as_slice(),
        b"http://",
        b"file://example.org",
        b"https://x/<tag>",
        b"https://x/\"",
        b"https://x/ space",
    ] {
        assert!(!markdown::safe_url(unsafe_url));
    }
}

#[test]
fn agent_reports_and_malformed_inputs_fit_the_byte_bound() {
    // Verbatim passages from the extraction, module move and
    // smith-protocol-layer coordination logs under ~/src/plans/ (01 to 12),
    // and replies holding tables (13 to 16, ui/markdown.md).
    let reports: [&[u8]; 16] = [
        include_bytes!("markdown/01.md"),
        include_bytes!("markdown/02.md"),
        include_bytes!("markdown/03.md"),
        include_bytes!("markdown/04.md"),
        include_bytes!("markdown/05.md"),
        include_bytes!("markdown/06.md"),
        include_bytes!("markdown/07.md"),
        include_bytes!("markdown/08.md"),
        include_bytes!("markdown/09.md"),
        include_bytes!("markdown/10.md"),
        include_bytes!("markdown/11.md"),
        include_bytes!("markdown/12.md"),
        include_bytes!("markdown/13.md"),
        include_bytes!("markdown/14.md"),
        include_bytes!("markdown/15.md"),
        include_bytes!("markdown/16.md"),
    ];
    for report in reports {
        let _tree = read(report, 4);
    }
    let alphabet = b"*`[]()!#> |-:\\a\n";
    for seed in 0..32_usize {
        let input: Vec<_> =
            (0..48_usize).map(|index| alphabet[(index * 7 + seed * 3 + index / 7) % alphabet.len()]).collect();
        let _tree = read(&input, 3);
    }
}

#[expect(clippy::use_debug, reason = "tree shape assertions show the role and run variants by name")]
fn shape(tree: &Tree) -> String {
    use core::fmt::Write;
    let mut shape = String::new();
    let mut ends = Vec::new();
    for (index, node) in tree.nodes().iter().enumerate() {
        while ends.last().is_some_and(|end| *end <= index) {
            ends.pop();
        }
        write!(shape, "{}{:?}", "  ".repeat(ends.len()), node.role).unwrap();
        for run in &node.text {
            write!(shape, " {:?} b\"{}\"", run.run, run.text.escape_ascii()).unwrap();
        }
        if let Some(target) = &node.external {
            write!(shape, " to b\"{}\"", target.escape_ascii()).unwrap();
        }
        shape.push('\n');
        ends.push(index.checked_add(usize::try_from(node.size).unwrap()).unwrap());
    }
    shape
}

#[test]
fn table_cells_keep_runs_and_their_own_counts() {
    assert_eq!(
        shape(&read(include_bytes!("markdown/13.md"), 1)),
        r#"Page
  Paragraph
    Text Plain b"The checks are ready for review."
  Table
    Row
      ColumnHeader
        Text Plain b"Check"
      ColumnHeader
        Text Plain b"Result"
      ColumnHeader
        Text Plain b"Reference"
    Row
      Cell
        Text Strong b"Ready"
      Cell
        Text Code b"pass"
      Cell
        Link to b"https://example.org/report"
          Text Plain b"report"
    Row
      Cell
        Text Plain b"Short"
      Cell
        Text Plain b"kept"
    Row
      Cell
        Text Plain b"Extra"
      Cell
        Text Plain b"cells"
      Cell
        Text Plain b"stay"
      Cell
        Text Plain b"here"
  Paragraph
    Text Plain b"I will keep the report with the change."
"#
    );
}

#[test]
fn quoted_tables_stay_text_and_indented_tables_follow_lists() {
    assert_eq!(
        shape(&read(include_bytes!("markdown/14.md"), 1)),
        r#"Page
  Paragraph
    Text Plain b"The quoted reply keeps its table as text."
  Quote
    Text Plain b"| a | b |"
  Quote
    Text Plain b"|---|---|"
  Quote
    Text Plain b"| 1 | 2 |"
  List
    Item
      Text Plain b"The indented summary follows this item."
  Table
    Row
      ColumnHeader
        Text Plain b"Check"
      ColumnHeader
        Text Plain b"Result"
    Row
      Cell
        Text Plain b"Build"
      Cell
        Text Plain b"passed"
  Paragraph
    Text Plain b"That summary is ready."
"#
    );
}

#[test]
fn broken_tables_stay_text_and_blocks_end_rows() {
    assert_eq!(
        shape(&read(include_bytes!("markdown/15.md"), 1)),
        r#"Page
  Paragraph
    Text Plain b"These incomplete summaries stay readable."
    Text Plain b"\n"
    Text Plain b"| **Mismatch** | cell |"
    Text Plain b"\n"
    Text Plain b"| -- |"
  Paragraph
    Text Plain b"| **No delimiter** | cell |"
    Text Plain b"\n"
    Text Plain b"The next line is prose."
  Paragraph
    Text Plain b"| Bad | cell |"
    Text Plain b"\n"
    Text Plain b"|--x|--|"
  Table
    Row
      ColumnHeader
        Text Plain b"End"
      ColumnHeader
        Text Plain b"Reason"
    Row
      Cell
        Text Plain b"one"
      Cell
        Text Plain b"text"
  Paragraph
    Text Plain b"This line ends the table."
  Table
    Row
      ColumnHeader
        Text Plain b"End"
      ColumnHeader
        Text Plain b"Reason"
    Row
      Cell
        Text Plain b"two"
      Cell
        Text Plain b"heading"
  Heading { level: 3 }
    Text Plain b"Details"
  Table
    Row
      ColumnHeader
        Text Plain b"End"
      ColumnHeader
        Text Plain b"Reason"
    Row
      Cell
        Text Plain b"three"
      Cell
        Text Plain b"list"
  List
    Item
      Text Plain b"The list ends the last table."
"#
    );
}

#[test]
fn escaped_and_code_pipes_keep_their_cells() {
    assert_eq!(
        shape(&read(include_bytes!("markdown/16.md"), 1)),
        r#"Page
  Paragraph
    Text Plain b"The scanner keeps pipes inside their cells."
  Table
    Row
      ColumnHeader
        Text Plain b"Form"
      ColumnHeader
        Text Plain b"Value"
    Row
      Cell
        Text Plain b"a "
        Text Plain b"| b"
      Cell
        Text Code b"x | y"
    Row
      Cell
        Text Code b"a"
        Text Code b"|b"
      Cell
        Text Plain b"escaped"
    Row
      Cell
        Text Plain b"loose `"
      Cell
        Text Plain b"tail"
    Row
      Cell
      Cell
      Cell
    Row
      Cell
        Text Plain b"<b>"
        Text Plain b"|**raw**</b>"
      Cell
        Text Plain b"final "
        Text Plain b"|"
  Paragraph
    Text Plain b"The plain reply keeps a \\| b as written."
"#
    );
}

#[test]
fn empty_table_rows_fit_the_declared_byte_bound() {
    // ui/markdown.md: the row of three empty cells is the worst table row.
    const ROWS: usize = 128;
    let input = format!("|||\n|-|-|\n{}", "||||\n".repeat(ROWS));
    let tree = read(input.as_bytes(), 0);
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Row).count(), ROWS + 1);
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Cell).count(), ROWS * 3);
}

const KEPT: [&[u8]; 18] = [
    include_bytes!("markdown/01.md"),
    include_bytes!("markdown/02.md"),
    include_bytes!("markdown/03.md"),
    include_bytes!("markdown/04.md"),
    include_bytes!("markdown/05.md"),
    include_bytes!("markdown/06.md"),
    include_bytes!("markdown/07.md"),
    include_bytes!("markdown/08.md"),
    include_bytes!("markdown/09.md"),
    include_bytes!("markdown/10.md"),
    include_bytes!("markdown/11.md"),
    include_bytes!("markdown/12.md"),
    include_bytes!("markdown/13.md"),
    include_bytes!("markdown/14.md"),
    include_bytes!("markdown/15.md"),
    include_bytes!("markdown/16.md"),
    include_bytes!("markdown/17.md"),
    include_bytes!("markdown/18.md"),
];

fn growing(input: &[u8], with_end: bool) -> (Tree, usize) {
    let bytes = u32::try_from(input.len()).unwrap();
    let limits = crate::Limits {
        nodes: markdown::max_nodes(bytes).unwrap().checked_add(2).unwrap(),
        depth: 12,
        text_per_node: bytes.checked_mul(2).unwrap().checked_add(1).unwrap(),
        text_bytes: markdown::max_text_bytes(bytes).unwrap().checked_add(1).unwrap(),
        markdown_depth: 4,
        ..crate::tests::fixtures::LIMITS
    };
    let mut builder = Builder::new(&limits);
    builder.open(Role::Page);
    let mut calls = 0_usize;
    markdown::read_growing(
        input,
        2,
        |builder| {
            calls = calls.checked_add(1).unwrap();
            if with_end {
                builder.text(Run::Plain, b"~");
            }
        },
        &mut builder,
    );
    builder.close();
    let tree = builder.finish();
    assert!(tree.len() <= limits.nodes);
    let charged: u32 = tree.nodes().iter().map(crate::Node::text_bytes).sum();
    assert!(charged <= limits.text_bytes);
    (tree, calls)
}

fn has_run(tree: &Tree, run: Run, words: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|text| text.run == run && text.text.as_ref() == words))
}

#[test]
fn a_bold_still_open_is_bold() {
    for (input, run, words) in [
        (b"a **bold wo".as_slice(), Run::Strong, b"bold wo".as_slice()),
        (b"a *soft wo", Run::Emphasis, b"soft wo"),
        (b"a __bold wo", Run::Strong, b"bold wo"),
        (b"a _soft wo", Run::Emphasis, b"soft wo"),
        (b"a `code wo", Run::Code, b"code wo"),
        (b"**first\nsecond", Run::Strong, b"first"),
    ] {
        let (tree, calls) = growing(input, false);
        assert!(has_run(&tree, run, words), "{}", shape(&tree));
        assert_eq!(calls, 1);
    }
    assert_eq!(
        shape(&growing(b"**closed\n\n**open", false).0),
        "Page\n  Paragraph\n    Text Plain b\"**closed\"\n  Paragraph\n    Text Strong b\"open\"\n"
    );
}

#[test]
fn a_delimiter_alone_at_the_end_makes_no_text() {
    for input in [b"word **".as_slice(), b"word *", b"word __", b"word _", b"word `"] {
        assert_eq!(text(&growing(input, false).0), b"word ");
    }
}

#[test]
fn a_link_still_being_written_shows_its_label() {
    for input in [b"[label](htt".as_slice(), b"[label", b"[label]", b"[label](", b"[label](https://example.org"] {
        let (tree, calls) = growing(input, false);
        assert_eq!(text(&tree), b"label");
        assert!(tree.nodes().iter().all(|node| node.external.is_none()));
        assert_eq!(calls, 1);
    }
    for input in [b"![image](htt".as_slice(), b"[label](javascript:bad)", b"<b>*raw*"] {
        assert_eq!(shape(&growing(input, false).0), shape(&read(input, 4)));
    }
}

#[test]
fn a_marker_being_written_waits_for_its_words() {
    for input in
        [b"".as_slice(), b"##", b"## ", b"-", b"- ", b"*", b"* ", b"4", b"4.", b"4. ", b">", b"> ", b"[", b"**", b"`"]
    {
        let (tree, calls) = growing(input, true);
        assert!(text(&tree).is_empty(), "{input:?}");
        assert_eq!(calls, 0, "{input:?}");
        assert_eq!(tree.len(), 1, "{input:?}");
    }
    for input in [b" ".as_slice(), b"\n", b"\t\r\n"] {
        assert_eq!(growing(input, true).1, 0);
    }
    assert_eq!(text(&growing(b"words\n4", false).0), b"words");
    assert_eq!(text(&growing(b"words\n4x", false).0), b"words\n4x");
}

#[test]
fn a_fence_still_open_is_code() {
    let (tree, calls) = growing(b"```\nlet a", false);
    assert_eq!(shape(&tree), "Page\n  Code\n    Text Code b\"let a\"\n");
    assert_eq!(calls, 1);
}

#[test]
fn a_fence_being_opened_or_closed_shows_no_backticks() {
    for input in [b"```ru".as_slice(), b"`", b"``", b"```"] {
        let (tree, calls) = growing(input, true);
        assert_eq!(text(&tree).len(), 0);
        assert_eq!(calls, 0);
    }
    for input in [b"```\nlet a\n`".as_slice(), b"```\nlet a\n``"] {
        assert_eq!(text(&growing(input, false).0), b"let a\n");
    }
}

#[test]
fn a_delimiter_row_being_written_waits() {
    for input in [b"| a | b |\n|--".as_slice(), b"| a | b |\n|", b"| a | b |\n|--:"] {
        assert_eq!(shape(&growing(input, false).0), shape(&read(b"| a | b |", 4)));
    }
    assert_eq!(shape(&growing(b"| a | b |\n|--|--|", false).0), shape(&read(b"| a | b |\n|--|--|", 4)));
}

#[test]
fn the_end_stands_after_the_last_word_in_the_block_that_holds_it() {
    for (input, role) in [
        (b"word".as_slice(), Role::Paragraph),
        (b"# word", Role::Heading { level: 3 }),
        (b"- word", Role::Item),
        (b"4. word", Role::Item),
        (b"> word", Role::Quote),
        (b"```\nword", Role::Code),
        (b"word\n\n", Role::Paragraph),
        (b"# word\n\n", Role::Heading { level: 3 }),
        (b"- word\n\n", Role::Item),
        (b"> word\n\n", Role::Quote),
        (b"```\nword\n```\n\n", Role::Code),
        (b"word\n##", Role::Paragraph),
        (b"word\n\n**", Role::Paragraph),
        (b"word\n\n# **", Role::Paragraph),
        (b"word\n```ru", Role::Paragraph),
        (b"| word |\n|--|\n| last |", Role::Cell),
        (b"| word |\n|--|", Role::ColumnHeader),
    ] {
        let (tree, calls) = growing(input, true);
        assert_eq!(calls, 1, "{input:?}");
        let end = tree.nodes().iter().position(|node| node.text.iter().any(|run| run.text.as_ref() == b"~")).unwrap();
        let (at, parent) = tree
            .nodes()
            .iter()
            .enumerate()
            .take(end)
            .rev()
            .find(|(at, node)| at.checked_add(usize::try_from(node.size).unwrap()).unwrap() > end)
            .unwrap();
        assert_eq!(parent.role, role, "{input:?}");
        assert!(
            tree.nodes()[at.checked_add(1).unwrap()..end]
                .iter()
                .any(|node| node.text.iter().any(|run| run.text.iter().any(|byte| !byte.is_ascii_whitespace()))),
            "{input:?}"
        );
        assert!(text(&tree).ends_with(b"~"));
    }
}

#[test]
fn an_empty_last_cell_keeps_the_end_where_its_next_word_will_go() {
    let (tree, calls) = growing(b"| a |\n|--|\n| c | **", true);
    assert_eq!(calls, 1);
    let cell = tree.nodes().iter().rposition(|node| node.role == Role::Cell).expect("last cell");
    let end = tree.nodes().iter().position(|node| node.text.iter().any(|run| run.text.as_ref() == b"~")).unwrap();
    assert!(end > cell && end < cell.checked_add(usize::try_from(tree.nodes()[cell].size).unwrap()).unwrap());
    assert!(tree.nodes()[cell..end].iter().all(|node| node.text.is_empty()));
}

#[test]
fn closed_blocks_read_as_read_reads_them() {
    for input in [
        b"a **bold**".as_slice(),
        b"a *soft*",
        b"a __bold__",
        b"a _soft_",
        b"_soft_",
        b"__strong__",
        b"_Draft, with the user._",
        b"*a _b_ c*",
        b"un*frigging*believable",
        b"snake_case",
        b"a `code`",
        b"[label](https://example.org)",
        b"# heading",
        b"- first\n- last",
        b"> quoted",
        b"```\ncode\n```",
        b"| a | b |\n|--|--|\n| c | d |",
    ] {
        assert_eq!(shape(&growing(input, false).0), shape(&read(input, 4)));
    }
    for report in KEPT {
        let mut fence = false;
        let mut start = 0;
        for (end, byte) in report.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }
            let line = &report[start..end];
            if line.starts_with(b"```") {
                fence = !fence;
            }
            if line.is_empty() && !fence {
                let prefix = &report[..=end];
                assert_eq!(shape(&growing(prefix, false).0), shape(&read(prefix, 4)));
            }
            start = end.checked_add(1).unwrap();
        }
        assert_eq!(shape(&growing(report, false).0), shape(&read(report, 4)));
    }
}

#[test]
fn every_cut_of_the_kept_texts_stays_in_its_bounds() {
    for report in KEPT {
        let report = core::str::from_utf8(report).unwrap();
        for end in 0..=report.len() {
            if report.is_char_boundary(end) {
                let (tree, calls) = growing(&report.as_bytes()[..end], true);
                assert!(
                    tree.len()
                        <= markdown::max_nodes(u32::try_from(end).unwrap())
                            .unwrap()
                            .checked_add(1)
                            .unwrap()
                            .checked_add(u32::try_from(calls).unwrap())
                            .unwrap()
                );
                assert!(calls <= 1);
            }
        }
    }
}

fn orders(tree: &Tree) -> Vec<Option<u32>> {
    tree.nodes().iter().filter(|node| node.role == Role::List).map(|node| node.ordered).collect()
}

#[test]
fn a_numbered_list_says_its_order_and_its_marker_leaves_its_words() {
    for (input, expected, words) in [
        ("1. a\n2. b\n", vec![Some(1)], "ab"),
        ("4. d\n5. e\n", vec![Some(4)], "de"),
        ("0. z\n", vec![Some(0)], "z"),
        ("999999999. n\n", vec![Some(999_999_999)], "n"),
        ("- a\n- b\n", vec![None], "ab"),
        ("1. \n", vec![Some(1)], ""),
    ] {
        let tree = read(input.as_bytes(), 3);
        assert_eq!(orders(&tree), expected, "{input:?}");
        assert_eq!(text(&tree), words.as_bytes(), "{input:?}");
    }
}

#[test]
fn every_number_shows_as_written() {
    // Top-level roles also pin continuation paragraphs and outer-list item counts.
    for (input, expected, roles, outer_items) in [
        ("1. a\n3. b\n", vec![Some(1), Some(3)], vec![Role::List, Role::List], 1),
        ("1. a\n1. b\n", vec![Some(1), Some(1)], vec![Role::List, Role::List], 1),
        ("3. a\n2. b\n", vec![Some(3), Some(2)], vec![Role::List, Role::List], 1),
        ("1. a\n\n2. b\n", vec![Some(1), Some(2)], vec![Role::List, Role::List], 1),
        ("1. a\n   under\n2. b\n", vec![Some(1), Some(2)], vec![Role::List, Role::Paragraph, Role::List], 1),
        ("3. a\n\n   under\n   it\n4. b\n", vec![Some(3), Some(4)], vec![Role::List, Role::Paragraph, Role::List], 1),
        ("1. a\n   - x\n2. b\n", vec![Some(1), None], vec![Role::List], 2),
        ("1. a\n   1. x\n   2. y\n2. b\n", vec![Some(1), Some(1)], vec![Role::List], 2),
        ("- a\n1. b\n", vec![None, Some(1)], vec![Role::List, Role::List], 1),
        ("1. a\n- b\n", vec![Some(1), None], vec![Role::List, Role::List], 1),
        ("Steps:\n1. a\n2. b\n", vec![Some(1)], vec![Role::Paragraph, Role::List], 2),
        ("1. a\n\nbefore\n9. after\n", vec![Some(1)], vec![Role::List, Role::Paragraph], 1),
    ] {
        let tree = read(input.as_bytes(), 3);
        assert_eq!(orders(&tree), expected, "{input:?}");
        let mut at = 1;
        let mut top = Vec::new();
        while at < tree.len() {
            let node = tree.get(at).unwrap();
            top.push(node.role);
            at += node.size;
        }
        assert_eq!(top, roles, "{input:?}");
        let outer_at = tree.nodes().iter().position(|node| node.role == Role::List).unwrap();
        let outer = tree.get(u32::try_from(outer_at).unwrap()).unwrap();
        let mut child = u32::try_from(outer_at).unwrap() + 1;
        let mut items = 0_u32;
        while child < u32::try_from(outer_at).unwrap() + outer.size {
            let node = tree.get(child).unwrap();
            assert_eq!(node.role, Role::Item);
            items += 1;
            child += node.size;
        }
        assert_eq!(items, outer_items, "{input:?}");
    }
}

#[test]
fn a_line_that_merely_starts_with_a_number_stays_text() {
    for input in [
        "3.14 is pi",
        "3 apples",
        "1.5 million",
        "3) three",
        "1.",
        "1.five",
        "007. bond",
        "1234567890. ten digits",
        "before\n42. after",
        "before\n2. after",
    ] {
        let tree = read(input.as_bytes(), 3);
        assert!(orders(&tree).is_empty(), "{input:?}");
        assert_eq!(text(&tree), input.as_bytes(), "{input:?}");
    }
}

#[test]
fn reports_read_their_numbers_as_written() {
    for (input, expected) in [
        (include_bytes!("markdown/01.md").as_slice(), vec![Some(3)]),
        (include_bytes!("markdown/03.md").as_slice(), vec![Some(1)]),
        (include_bytes!("markdown/05.md").as_slice(), vec![]),
        (include_bytes!("markdown/17.md").as_slice(), vec![Some(1), None, Some(4), Some(7), Some(1)]),
    ] {
        assert_eq!(orders(&read(input, 3)), expected, "{}", input.escape_ascii());
    }
    assert!(text(&read(include_bytes!("markdown/05.md"), 3)).windows(3).any(|word| word == b"4. "));
}

#[test]
fn a_growing_numbered_list_keeps_its_order_and_its_end() {
    let input = b"4. first\n5. **last**";
    let (tree, calls) = growing(input, true);
    assert_eq!(orders(&tree), [Some(4)]);
    assert_eq!(orders(&tree), orders(&read(input, 4)));
    assert_eq!(text(&tree), b"firstlast~");
    assert_eq!(calls, 1);
    let item = tree.nodes().iter().rposition(|node| node.role == Role::Item).unwrap();
    let end = tree.nodes().iter().position(|node| node.text.iter().any(|run| run.text.as_ref() == b"~")).unwrap();
    assert!(end > item && end < item.checked_add(usize::try_from(tree.nodes()[item].size).unwrap()).unwrap());
}
