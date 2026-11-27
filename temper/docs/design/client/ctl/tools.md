# The tools

## 1. One table

- **A tool is a record of the table:** its name; a line saying what it
  does; its arguments, each a name, a type (a number, an amount, a word from a set,
  a text, a file, a duration) with its bound where it has one, whether
  it is required, and a line; its result, a typed record, and a line
  saying what it prints; whether it is keyed (`requests.md`, section 1); its area
  (`areas.md`); and its notes, at most 1,024 bytes, what this tool alone does
  beyond its line: a file's life, a refusal's words and what mends it,
  a step that follows.
- **Everything else is derived from it:** the parser and `temper ctl
  help [GROUP [VERB]]`, the JSON of arguments and results, and the tool
  names the pages may use (`testing.md`, 5). A test checks that each derivation
  names every tool. Its lines and notes are documentation, in the voice
  of `../../protocol/documentation.md`, section 2.3.
- **A call is a value:** a tool's name and its arguments, decoded from
  argv or from JSON and checked against the table before anything is
  sent. The runner takes a call and gives back its result; the host
  interprets neither.
- **A tool's name is its words:** a group, one of the map's nouns
  (`task`, `chat`, `goal`, `change`,
  `proposal`, `settings`, `revision`, `account`, `model`, `forge`,
  `repository`, `playground`, `hold`, `landing`, `finding`, `qa`,
  `role`, `need`, and
  area 3's `project`, `party`, `deployment` and `worker` (`workers.md`); `../../protocol/documentation.md`,
  2.4), then a verb of the closed vocabulary, `task show`; or
  one of the seven names of the client and the party, `me`, `inbox`,
  `sign-in`, `sign-out`, `link`, `pending` and `help`. A read of a part of the
  group's noun is the part's name: `task tree`, `task run`, `task
  activity`, `task turn`, `forge repositories`, `repository receipts`,
  `qa marks`, `qa passes`. `help GROUP` lists a
  group's rows; `--call`'s `tool` is the name's words, one space
  between (`command-line.md`, 1.1).
- **The table's vocabulary of placeholders** shows the arguments in
  the usage; written pages use `../../protocol/documentation.md`, 2.1.
  The table's `shown` names are `TASK`,
  `PROJECT`, `FORGE`, `INSTALLATION`, `ACCOUNT`, `PROVIDER`, `ADAPTER`,
  `SOURCE`, `POSITION`, `NAME` (a program source's), `HOLD` (a placed
  hold's number), `LANDING`, `FINDING`, `REVISION` (a
  settings revision), `REV` (a revision a read printed of the object
  acted on: a spec's, a hold's, a person task's), `PROPOSAL`, `QUESTION`,
  `CHOICE` (a person contract's code), `STATE` (its approval state), `NUMBER` (a
  setting's), `TEMPLATE`, `CHARTER`, `CURSOR`, `COUNT`, `PART`, `OFFSET`, `PAGE`,
  `TURN`, `DIGEST`, `CONDITION`, `DURATION`, `TIME`, `PRIORITY`,
  `MODEL`, `EFFORT`, `AMOUNT`, `OWNER`, `ORG`, `REPOSITORY`, `BRANCH`,
  `PREFIX`, `GATE`, `LABEL`, `IDS`, `APP`, `ROLE`, `FILE`, `TEXT`,
  `REASON`, `KEY`, `AREA` (an area's name, `../../domain/areas.md`, section
  2), `PARTY` (a party's number), and `WORKER`, `PLACES` and `SHA256` (`workers.md`, 1). FINDING is written `P/N`, its
  project's number or name, then its number in the project. A step test refuses any other `shown` name.
- **Projects by name or number;** a name is resolved through `me`. A
  name is never decimal digits alone, and no two projects share one
  (jig's `domain/model.md`, 6.1), so an argument is a number or a name,
  never both, and names one project.
- **Templates, charters and policies by name or number,** the same
  way: wherever a call takes a TEMPLATE, a CHARTER or a policy's or
  such a setting's NUMBER (`--template`, `--charter`, `settings show
  charter|template|policy|charter-words|template-goal`), it takes the
  item's name too (jig's `domain/engine/settings.md`, 1, Names), never
  decimal digits alone. A template's name is resolved through the
  project's templates in `me`, a charter's through `me`'s `charters`,
  and, for `settings show`, each through the settings' inventory; the
  request sends the number. A name nothing has is usage, exit 1,
  nothing sent, naming it and where names are listed: `temper ctl:
  no template is named lean-goal; temper ctl settings show lists them`.
  Where a call prints a template, a charter or a policy by number, in
  `settings show`'s inventory and `me`, it prints its name beside it.

## 2. Reads

Moved to [`tools-reads.md`](tools-reads.md), section 1.

## 3. Acts

Moved to [`tools-acts.md`](tools-acts.md), section 1.

## 4. Waiting

Moved to [`tools-waiting.md`](tools-waiting.md), section 1.

## 5. Open questions

None.
