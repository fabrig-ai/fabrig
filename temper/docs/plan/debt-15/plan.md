# debt-15: carry out the user's answers

Three independent parts (`inputs/request.md`). This first release holds parts 1 and 2. Part 3, skein D011, is being
surveyed and follows.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `temper-wrong-kind` | `temper-wrong-kind` | | medium | 30 | a wrong kind's `Corrupt` stops the engine; the one behaviour change in temper |
| `github-media-type` | `github-media-type` | | medium | 30 | `Server`'s charset switch goes; the three approved test changes |
| `d013-kept` | `d013-kept` | | medium | 5 | hearth's D013 `kept` |
| `d001-kept` | `d001-jig` | | medium | 30 | `config::clients::invalid`, `link_owner`, the read check's test moves to the store |
| `d001-kept` | `d001-temper` | `d001-jig` | medium | 25 | temper-service takes them; D001 `kept` |

## Critical path

As it stands: `d001-jig`, `d001-temper`, 55 minutes. Part 3 will be the longest.

## Decisions

- **`listener-off-loopback` is done in this cut's push, not in a task.** Both of its edits are design text: the **Departs**
  entry of `temper/docs/design/protocol/configuration.status.md` now says the user chose the check, and section 11's open
  question goes. Section 3's table stays as it is, as the request records the gap rather than moving the section.
- **`WRONG_KIND` isn't `Token::new(PREFIX)`,** as the held spec sketched. A source's restore reads under its own number,
  so exchange 0 could be a real read's owner. Bit 59 is above every exchange jig's accounts issue (their highest is bit
  39) and under the `tag` assert.
- **A wrong kind on any `secret()` read stops the engine,** not only a restore's: jig's accounts give the owner one
  `InvalidSecret`, and jig's reference stops on all of them too.
- **D001's sizing test moves to `jig-protocol-store`,** the crate that holds `check_read_page`. It's the test of the
  check, not of either service's composition, so this is the right tier and no budget moves.
