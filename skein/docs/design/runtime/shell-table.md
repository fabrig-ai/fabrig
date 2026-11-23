# The shell kit: a binary's table

## 1. A binary's table

What `../../foundation/cli.md`, 6 to 8 asks of skein: one table per binary,
whose types and rendering every tree's binaries share. A tree's rows are
its own; the binary's crate holds them as constants. It is portable
(`shell.md`, section 13): it takes bytes and returns text, and reads nothing.

### 1.1 Entries and sets

- **`Set`:** `Public`, `Internal` or `Test` (`../../foundation/cli.md`, 4).
- **`Table`:** the binary's name; its one line, what the binary is; its
  version and commit, as its tree's build identity words them; its
  commands; the settings every command takes, as groups (`shell-startup.md`, 1.5); the
  variables it reads or sets beside its settings' twins; and its last
  line, where to read more.
- **`Command`:** how it's reached, its set, its forms (each one's
  arguments as usage shows them after the name), its one line, its
  positional arguments, its settings' groups, its switches, its
  variables, and its notes. It's reached by its words after the
  binary's name, none for the bare binary's form, or by the name the
  binary runs as, for a delegated grammar
  (`../../foundation/cli.md`, 5).
- **`Argument`:** a positional argument as usage shows it, and its line.
- **`Switch`:** a flag with no value, `--name`: its name, set, line and
  notes. A setting (`shell-startup.md`, 1.5) is a flag with a value.
- **`Variable`:** its name, set, form, line, what holds when it's
  absent, notes, and whether the binary reads it or sets it for the
  programs it runs.
- **The host's variables** that the kit itself reads are its constants,
  public: `HOME` and the four XDG base directories, read by a setting's
  `Fallback::Base` (`shell-startup.md`, 1.5). A table that holds such a setting reads them,
  and its documents say so without the tree declaring them again.
- **`TestTable`:** a tree's test set: its variables, all `Test`, held in
  its test-support crate (`../../foundation/cli.md`, 8).
- **A lookup names a declared entry:** `Variable::value` answers a
  variable's bytes from the captured environment, the pairs
  `environment` returns, and an empty value is unset. No lookup takes a
  string.

### 1.2 Help

Every page is lines of at most 80 columns. A two-column line puts its
first column at column 2 and its second at column 37; a first column
of more than 33 characters takes the line's own row, with the second
column on the next. A second column, or a one-column line, wraps at
spaces to 80 columns, a two-column line's continuation at column 37 and
a one-column line's at its own indent.

- **The page,** for `-h`, `--help` and `help` alike, its parts
  separated by one blank line:
  1. `BINARY: LINE`;
  2. `usage:`, then `  BINARY FORM` for each form of a public bare
     command, and `  BINARY COMMAND [ARGUMENT...]` when a public command
     has words;
  3. `commands:`, then each public command with words: `WORDS FORM` and
     its line, the line beside the first form only;
  4. `flags:`, then each of the table's groups of settings that has a
     public one: its public flags, `--name VALUE` joined by `, `, and
     the group's line; then `-h, --help` with `Print this.` and
     `--version` with `Print the name, version and commit.`;
  5. `environment:`, when there is any: for each group of the table's
     or of a public command's, the twins of its public settings, joined
     by `, `, with `As --a, --b and --c.`; then each public variable
     that the table or a public command reads, once, with its line;
  6. the table's last line.
- **A command's page,** for `COMMAND -h` or `--help`: an internal
  command's first line is `internal: not part of BINARY's public
  command line; it may change without notice.` Then `usage:` with its
  forms; its line; `arguments:`; `flags:`, each setting with its line,
  `Default: DEFAULT.` or `Required.`, and `Environment: TWIN.` where it
  has one, then each switch with its line; `environment:` with the
  variables it reads; and its notes. A public command's page lists its
  public entries; an internal command's lists all of its own. A part
  with nothing in it is left out.
- **The version line:** `BINARY VERSION (COMMIT)`, as temper's
  `release.md`, section 3 words it for every binary of temper's.

### 1.3 The reference documents

Rendered for `../../foundation/cli.md`, 8, each a Markdown document whose
first line is its title and whose second paragraph says it's generated:

- **`<binary>-args.md`:** `# BINARY: arguments`, then `## Public` and
  `## Internal`, each holding the commands then the flags of that set.
  A command's heading is `` ### `BINARY WORDS` `` (`` `BINARY` `` for
  the bare form, `` run as `NAME` `` for a delegated grammar), followed
  by a list: `**Set:**`, `**Forms:**`, its line, `**Arguments:**`,
  `**Flags:**` and `**Notes:**`, each item only where it has one. A
  flag's heading is `` ### `--name VALUE` `` or `` ### `--name` ``,
  with `**Set:**`, `**Default:**` (or `**Required.**`), `**Twin:**`,
  its line and `**Notes:**`.
- **`<binary>-env.md`:** `# BINARY: environment`, then `## Public` and
  `## Internal`, holding the variables the binary alone reads or sets,
  its settings' twins among them. A variable's entry is its heading,
  `` ### `NAME` ``, then a list whose first item is `- **Set:** public`,
  `internal` or `test`, then `**Reads**` or `**Sets**`, `**Form:**`,
  `**Absent:**`, `**Twin of:**` for a twin, its line and `**Notes:**`.
- **`<tree>/docs/design/cli/common-env.md`:** `# TREE: common environment`, the variables two
  or more of the tree's tables read, the host's among them, as
  `<binary>-env.md` holds them, with `**Read by:**` naming the binaries.
- **`<tree>/docs/design/cli/test-env.md`:** `# TREE: test environment`, the tree's test table,
  as `<binary>-env.md` holds a variable.

Entries keep their table's order; a variable read by several commands
appears once. The checks read these headings and `**Set:**` items
(`../../foundation/cli.md`, 9, check 5), so their form is the contract.

### 1.4 Reaching a command

`Table::reach` answers what a binary's arguments reach, so that every
binary's dispatch, help and refusals agree
(`../../foundation/cli.md`, 5 and 7). It takes the name the binary was
run by, argv's first word, and the words after it, as the binary's
capture holds them, and reads nothing else:

```rust
pub enum Reached<'a> {
    /// `-h`, `--help` or `help`, alone: the page (14.2).
    Help,
    /// `--version`, alone: the version line (14.2).
    Version,
    /// A command's words, then `-h` or `--help` alone: its page.
    CommandHelp(&'static Command),
    /// A command and the arguments after its words.
    Command(&'static Command, &'a [Box<[u8]>]),
    /// No command: the refusal's line, newline included.
    Unknown(String),
}

impl Table {
    pub fn reach<'a>(&self, run_as: &[u8], arguments: &'a [Box<[u8]>]) -> Reached<'a>;
    pub fn unknown(&self, arguments: &[Box<[u8]>]) -> String;
}
```

In this order:

1. **Run as another program:** a command reached by `Reach::RunAs(NAME)`
   whose NAME is `run_as`'s last part, after its last `/`, is reached
   with every argument, its grammar delegated: `--help` and `--version`
   are its own.
2. **The binary's page and version:** exactly `-h`, `--help` or `help`
   is `Help`; exactly `--version` is `Version`.
3. **By its words:** the command reached by `Reach::Words` whose words
   are the longest prefix of the arguments, the bare form's none being
   a prefix of any. When the arguments after its words are exactly
   `-h` or `--help`, `CommandHelp`; otherwise `Command`, with those
   arguments.
4. **None:** `Unknown`, the line `unknown` gives.

`unknown` is `BINARY: no command WORDS; BINARY --help lists them` and a
newline, WORDS the arguments up to and including the first one no
command's words continue with, joined by spaces, and
`BINARY: no command; BINARY --help lists them` for no arguments. A
binary prints it on stderr and exits 2 (`../../foundation/cli.md`, 5).
A bare form that takes no positional argument answers a word it
doesn't take with the same line, so a mistyped command reads the same
whether the binary has a bare form or not.

### 1.5 The table's own words

What every binary answers from its table before any command of its own
runs: its page, its version line, a command's page and the refusal of a
command it doesn't have (`../../foundation/cli.md`, 5 and 7). One code
answers them for every binary, so their words and exit codes are the
same everywhere. Like the rest of the table it is portable: it takes the
captured arguments and returns text, and the binary writes it.

- **`Table::front(&self, arguments: &[Box<[u8]>]) -> Front<'_>`** reads
  the arguments after the binary's name, as `arguments` returns them
  (`shell-startup.md`, 1.5), and answers the first of these that holds:
  1. the first argument is `-h` or `--help`: `Print` of the page
     (1.2), whatever follows;
  2. the first argument is `--version`: `Print` of the version line,
     whatever follows;
  3. the only argument is `help` and no command is reached by the words
     `help`: `Print` of the page;
  4. the words of some command reached by one or more words are a
     prefix of the arguments: the command with the longest such words.
     When the argument right after its words is `-h` or `--help`,
     `Print` of its page, whatever follows; otherwise `Run` of it with
     the arguments after its words;
  5. the table has a bare command, reached by no words: `Run` of it with
     every argument, or with none when there is none. For a bare
     command, `-h` and `--help` are the page's (1), so its own page is
     printed only within a refusal (below);
  6. otherwise `Refuse`, of `BINARY: no command WORD; BINARY --help says
     more` and a newline, `WORD` the first argument as UTF-8 with each
     invalid sequence replaced, or of `BINARY: no command; BINARY --help
     says more` and a newline when there is no argument.
- **`Front<'a>`:**
  - `Print(String)`: the binary writes it to stdout and exits 0;
  - `Run { command: &'a Command, rest: &'a [Box<[u8]>] }`: the
    binary runs the command, one of its table's, reading `rest` as that
    command's own arguments, with `resolve` (`shell-startup.md`, 1.5) for its settings;
  - `Refuse(String)`: the binary writes it to stderr and exits 2.
- **`Table::refusal(&self, command: &Command, what: &str) -> String`:**
  the words a binary refuses one of its commands' arguments with, before
  it exits 2: `BINARY WORDS: WHAT` (`BINARY: WHAT` for the bare
  command) and a newline, then the `usage:` part of the command's page,
  as 1.2 renders it. For example, a bare command whose form is `OUT`,
  given none:

  ```
  demo: OUT is not given
  usage:
    demo OUT
  ```

- **What it leaves to the binary:** a delegated grammar's rows
  (`Reach::RunAs`), which the binary matches by the name it runs as
  before it reads its table, and every word of a command's own arguments,
  a `--` among them.
