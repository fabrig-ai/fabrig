# Markdown

What people and programs write (replies, results, notes) is read into
nodes by a bounded reader of skein-ui's own:

- **the subset:** paragraphs and line breaks; headings, below the levels
  the caller gives; emphasis, strong and inline code; fenced code, of
  which a flow is a drawing (1); lists,
  nested to the limit; quotes; tables (below); links whose target is a
  checked `http` or `https` URL, shown as text otherwise;
- **emphasis and strong,** by either delimiter: `*` or `_` around words
  is emphasis, `**` or `__` strong. A delimiter closes only a run its own
  byte opened, so `*soft_` stays text. An `_` or `__` with a word's
  character on both sides, an ASCII letter or digit or a byte of a
  character past ASCII, is a character, as CommonMark has it, so
  `snake_case`, `read_branch` and `a__b` stay as written; `*` keeps its
  reading within a word, `un*frigging*believable`;
- **the rest is text:** HTML, images, a delimiter never closed, a list
  nested too deep, a table's lines that make no table are shown as the
  characters they are, never dropped, and nothing is ever made into
  markup of the platform's;
- **text still being written:** `read_growing` reads a text that may yet
  grow, a reply as it arrives, with `read`'s subset. What a blank line
  outside a fence has closed reads as `read` reads it, so its nodes stay
  the same whatever follows. The last block is read as if it were
  closed, and shows no marker being written:
  - *a run still open* holds to the end of the line: a bold still open is
    bold, an emphasis emphasis, a code span code, a link's label its
    text. A delimiter alone at the end of the text, a link's target still
    being written and its `](` make no text yet;
  - *a marker still being written* waits for its words: a last line that
    is only a heading's `#`s, a list's `-`, `*` or digits with or without
    their `.`, or a quote's `>`, with or without the space after it, makes
    no node yet;
  - *a fence still open* is code, its body so far. Its opening line makes
    no node until it ends, nor does a last line of backticks that may
    close it;
  - *a table's delimiter still being written:* a last line of `|`, `-`,
    `:` and spaces under a line that could head a table makes no node, and
    the line above it reads as a paragraph's until the delimiter holds;
  - *its end:* once the text holds a word, the caller's `end` adds its
    nodes after the last word, inside the paragraph, heading, list item,
    quote, cell or code that holds it; a last cell with no word yet holds
    them, where its next word goes. A text that ends closed reads as
    `read` reads it, with `end`'s nodes added, so its committed text
    draws the same blocks;
- **numbered lists,** whose items' markers are a number, a dot and a
  space:
  - *a number:* one to nine digits, either `0` or with no leading zero,
    so that it shows as written. The marker leaves the item's words, and
    the list says it is ordered, from its first item's number (`README.md`, 3.2);
  - *as written:* a list's numbers go up by one. An item whose number is
    not the next ends its list and opens another from that number, at the
    same nesting, so `1.` then `3.`, or `1.` then `1.`, show as written,
    each a list of its own. A list a blank line or a line under an item
    ended is followed by one that says the number it goes on from: `1.`,
    a line under it and `2.` read 1 and 2, two lists with a paragraph
    between them;
  - *a line that merely starts with a number* stays text. One not of
    that form always does: `3.14`, `3 apples`, `3) three`, `1.` alone,
    `007. x`, a number of ten digits. One of that form does within a
    paragraph, whose wrapped sentence may start with a year, unless its
    number is `1` or the next of the last numbered list that anything but
    an item closed: that ends the paragraph and starts a list. Nothing
    else reads a number's sense, so a line alone that says `2026. A year.`
    is a list from 2026, and shows as written;
- **tables,** GFM's pipe tables, at the top level, where a paragraph
  stands:
  - *what makes one:* a header line, one the reader would otherwise read
    as a paragraph's, and the delimiter line after it, each holding a `|`
    that no backslash escapes, with as many cells each. A delimiter cell
    is dashes, with a colon at either end or at both (`---`, `:--`,
    `--:`, `:-:`), and spaces around. Otherwise both lines are what they
    were: a paragraph's lines, a line holding a `|` shown as written. A
    table ends the paragraph before it;
  - *its cells:* a row's line is cut at each `|`, past the spaces at its
    ends and one leading and one trailing `|`, and each cell drops the
    spaces at its ends. A `\|` is a `|` in its cell, its backslash
    dropped, within a code span too; a `|` inside a code span cuts no
    cell, the line's backticks pairing from the left and a last unpaired
    one being a character. A cell holds its runs as a paragraph's line
    does, plain where it holds a `<`;
  - *its rows:* the lines after the delimiter that hold a `|` no
    backslash escapes. A blank line ends it, as does a line holding none,
    or one that starts another block: a fence, a heading, a quote, a list
    item. A row with fewer cells than the header keeps fewer, and a row
    with more keeps them, past the header's columns: nothing is dropped,
    and no cell is made from no bytes;
  - *its nodes:* a table of rows, the header's of column headers, the
    others' of cells, each holding its runs' text nodes. The delimiter
    line makes no node: no property of `README.md`, 3.2 is an alignment, so its
    colons are read for its form alone, and a column's alignment is the
    look's;
  - *in a quote or a list item,* whose lines hold their runs and no
    paragraph, a table's lines stay what they are: each `> ` line a quote
    of its text, and a line indented under an item, which ends the list,
    so that an indented table follows its list, at the top level;
- **a step machine:** one pass over the lines, with a bounded stack of
  open blocks and of open emphasis; no recursion. A line that could head
  a table is read with the line after it;
- **bounded:** at most two nodes per input byte, plus two, and twice the
  input's bytes of text, which an owner counts (`README.md`, section 13). A numbered
  item that opens its own list keeps them: `1. ` and its newline, four
  bytes, make a list and an item. Its nodes
  nest at most `max(5, 2 × markdown_depth + 2)` below its parent: a list
  item's link and its text at the deepest list, or a table's row, cell,
  link and text. A table keeps the bounds line by line. A row's line
  makes its row, at most one cell more than its `|`s, and at most a text
  node a byte of its cells; the header's line, followed by its newline
  and the delimiter, which makes no node, also makes the table. A row
  of empty cells is the worst: `||||` and its newline, five bytes, make
  a row and three cells. `| a | b |`, `|---|---|` and `| 1 | 2 |`, with
  their newlines, are 30 bytes making 11 nodes and 4 bytes of text. A
  cell's text is its bytes, a link's label twice, less a `\|`'s
  backslash. Cells are found by a scan of their line, keeping no
  offsets, so a table adds no scratch to the reader's `markdown_bound`.
  `read_growing` keeps the same bounds, and its caller counts `end`'s
  nodes;
- **fed real text:** its tests read texts kept as files beside them.

## 1. Flows

A closed fence whose info string is `mermaid` and whose body is in the
subset below is a flow. Every other fence, and one past the subset whole,
stays fenced code, as any fence does while it is still open.

- **The fence:** its opening line three backticks, then `mermaid`, with
  spaces or tabs around it. Its body is read line by line, each line's
  spaces and tabs at its ends dropped.
- **Its first line:** after any blank lines and `%%` comments, `flowchart`
  or `graph`, spaces, then `LR` or `TD`, `TB` reading as `TD`: the flow's
  direction.
- **A node:** an id, ASCII letters, digits, `_` and `-`, ending before a
  `-->`, then optionally its label, `[label]` or `["label"]`, then
  optionally `:::strong`, which marks it.
  - *A label* is plain text, with no `]` unquoted and no `"` quoted, and
    no `<` but in `<br>` or `<br/>`, which splits it into the node's name
    and its caption, each with the spaces at its ends dropped and neither
    empty. A second split is past the subset.
  - *A node with no label* is named by its id.
  - *One label:* a node may be written many times and labelled in one of
    them; a second label is past the subset, the same words or not.
- **An edge:** `-->` between two nodes, with spaces or none, chained or
  not (`a --> b --> c`, or a line each). A line may hold one node alone.
- **Read and ignored:** a `classDef` line, which a forge needs to style a
  node, an `accDescr:` line, a `%%` comment line and a blank line.
- **Its name:** an `accTitle: words` line, at most one.
- **One path:** at least one edge, the edges making one path with every
  node on it: each node has at most one edge in and one out, and there is
  no cycle.
- **Past the subset:** anything else keeps the whole block fenced code: a
  branch, a merge, an edge's label, another arrow or shape, `subgraph`,
  `style`, `click`, another class after `:::`, a second `accTitle`.
- **What it makes,** where its data fits:
  - *a drawing* (`README.md`, 3.5), kind `flow::KIND`, its name the `accTitle`, or none;
  - *its data:* the direction, then each node in the path's order, its
    name, its caption or none, and whether it is strong;
  - *its fallback:* one paragraph holding one text node: each node's name,
    a comma, a space and its caption where it has one, joined by "; then ",
    ended by a full stop: "Design draft, you write it; then Goal, you
    start it; …". The words are the block's, not a phrasebook's: "then" is
    the one word the reader adds, as a table's reading adds none.
- **Bounded:** a block whose data passes `drawing_bytes`, or whose name or
  fallback passes `text_per_node`, stays fenced code, so limits with no
  `drawing_bytes` read every flow as code. A flow makes three nodes, and
  its name and fallback keep `markdown.md`'s bound of twice the input's bytes
  of text, since each edge's four bytes at least, a node's and its arrow's,
  add eight at most.
- **No cost for no flow:** the reader takes no scratch for a fence until
  its info string says `mermaid`, and none past `drawing_bytes`: each node
  takes at least four bytes of data, so a block with more nodes than a
  quarter of them stops being read and stays code.
- **Text still being written:** `read_growing` reads a closed flow as
  `read` does, and `end`'s nodes go inside its fallback's paragraph, which
  holds its last word.
