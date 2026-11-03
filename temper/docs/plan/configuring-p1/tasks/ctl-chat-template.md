+++
id = "ctl-chat-template"
part = "chat-template"
kind = "feature"
needs = ["jig-chat-from"]
files = ["temper/crates/temper-client-tools/src/jig/rows.rs", "temper/crates/temper-client-tools/src/jig/argv.rs", "temper/crates/temper-client-tools/src/jig/runner/acts.rs", "temper/crates/temper-client-tools/src/tests/jig.rs", "temper/crates/temper-client-tools/src/tests/jig/chat_template.rs", "temper/crates/temper-client-tools/src/tests/jig/choices.rs", "temper/crates/temper-client-tools/src/tests/help.rs", "temper/crates/temper-client-tools/src/tests/golden/**", "temper/crates/temper-client-tools/src/tests/fixtures/**", "temper/docs/help/reference/ctl-chat.md", "temper/docs/help/chat.md", "temper/docs/help/template.md", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 60
+++

# temper: `chat start --template`, any template a chat's

## Goal

`temper ctl chat start --project PROJECT [--template TEMPLATE] TEXT` starts a chat from
TEMPLATE, the project's chat template (template 1) when none is named; a template that is no
chat's, or not offered, is refused naming the chat templates `me` offers the project
(`temper/docs/design/client/tools.md`, 5.3, its first row). It is the first row of
`configuring.md`'s 5.4 ("Tell temper how you work"); template 3, the configuring chat, is not
this plan's: build it for any template.

Acceptance:

- The `chat start` row (`rows.rs`, `CHAT_CREATE`) gains, right after its `--project` flag,
  `flag(b"template", b"TEMPLATE", Kind::Number, Optional, b"A chat's template by its number; the project's chat template when absent")`,
  so its usage reads
  `chat start --project PROJECT [--template TEMPLATE] TEXT [--model MODEL] [--effort EFFORT] --key KEY`
  (and `tests/help.rs`'s `USAGES` says so). Its notes become exactly:

  ```
  The chat's template chooses its model unless `--model` names one;
  `--effort` needs `--model`, and a model that declares efforts needs
  `--effort`. A chat past what your party may start, but within what
  it may propose, is proposed (exit 0). `chat not offered` and `chat
  exceeds offered bounds` (exit 1): your party may not start it.
  `--template` names one of the project's chat templates; one that is
  no chat's, or not offered, fails naming those `me` offers (exit 1).
  Then task show TASK --until replied.
  ```

  (each line ending in `\n`, as the row's notes do today).
- `runner/acts.rs`'s `chat`: with `--template N`, the ask is
  `jig_client::shapes::chat_from(me, project, N, words)` (jig's task `jig-chat-from`); without
  it, `shapes::chat(me, project, words)` as today. Every later step (the LLM choice, the
  creating bounds, `fall`, the proposal) is unchanged. The values' indices move by one past
  `--project`: the words, `--model` and `--effort` are read at their new indices.
- `shapes::chat_from` giving `NotOffered::Template` for a named template fails, exit 1, with
  `Failed::Detail` whose text is exactly
  `template N is no chat template of this project; its chat templates: LIST`, LIST being each
  template `me` offers the project whose `flags.conversation` is true, in `me`'s order, as
  `NUMBER NAME` joined by `, ` (`1 Chat, 3 Configuring`), or `none`. Any other `NotOffered`
  keeps today's `chat not offered`.
- Step tests, a new module `src/tests/jig/chat_template.rs` (declared in `src/tests/jig.rs`),
  with today's fixtures (`f::chat_me`, `f::choice_root`, `root.submit`, `f::act_ask`), a `me`
  given a third template, number 3, a conversation's, run by charter 3 where a story needs one:
  - `chat_start_names_a_chat_template`: `--template 3` sends the ask
    `shapes::chat_from(&me, PROJECT, 3, words)` gives, its task's executor charter 3;
  - `chat_start_without_a_template_takes_the_chat_template`: the ask equals `shapes::chat`'s
    (the existing `chat_keeps_the_template_choice` may stand for it if it still holds);
  - `chat_start_refuses_a_goal_template`: `--template 2` (no conversation) prints the refusal
    above naming `1 ...` and `3 ...`, exit 1, nothing sent;
  - `chat_start_refuses_a_template_not_offered`: `--template 9`, the same refusal;
  - `chat_start_names_a_template_and_a_model`: `--template 3 --model M` keeps the named model,
    as `chat_uses_the_named_model` does for template 1.
  `heavy cargo nextest run -p temper-client-tools --lib -E 'test(/chat_template::/)'` passes,
  its count in the report; then the crate's lib tests, the help goldens regenerated
  (`SKEIN_CODEGEN_WRITE=1`) and read.
- The help pages below hold the text given, and `temper-docs`'s tests pass
  (`heavy cargo nextest run -p temper-docs`).
- The status entry this builds leaves `temper/docs/design/client/tools.status.md`: in its
  **Not yet** line, delete "`chat start`'s `--template`; " from the 5.3 entry, and nothing
  else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md`.
- `temper/docs/design/client/tools.md`, 5.3 (the `chat start` row) and section 6.3 (exit
  status); jig's `jig/docs/design/client/domain/client.md`, 19 (`shapes::chat_from`).

## Owns and may change

- Owns the files in the header. Its fixtures (`src/tests/fixtures/acts.rs`,
  `fixtures/choices.rs`) may change only for the moved indices (`call_words(CREATE_CHAT_CALL,
  1)` becomes index 2) and a third template where a story needs one.
- `src/jig/argv.rs` and `src/tests/jig/choices.rs` may change only for the moved indices:
  `check_form`'s `check_choice(call, 2, 3, limits)` for `chat start` becomes
  `check_choice(call, 3, 4, limits)`, and `choices.rs`'s text index and the model and effort
  argument indices in its expected `Usage` values move by one.
- Read only: `temper-client-domain` and `temper-client-view`, whose chat pages keep
  `shapes::chat`; `temper/tests/client` (no joined story is this task's).
- Pre-approved: if `Kind::Number` renders the flag's line otherwise than
  `--template TEMPLATE: a number; optional. ...`, keep the renderer's line and put that line in
  `ctl-chat.md`; if a golden other than `help` holds `chat start`'s usage, regenerate it too and
  list it.

### Help text, placed as given

`temper/docs/help/reference/ctl-chat.md`: the `## chat start` block's usage line becomes the
usage above; after its `--project PROJECT: ...` line comes
`  --template TEMPLATE: a number; optional. A chat's template by its number; the project's chat template when absent`;
its notes become the notes above. Nothing else of the page changes.

`temper/docs/help/chat.md`, under `## Limits`, the bullet

```
- temper does not let ctl choose a chat's template: `chat start` uses the
  project's chat template.
```

becomes

```
- `chat start` uses the project's chat template unless `--template`
  names another of its chat templates; a template that is no chat's
  starts none.
```

`temper/docs/help/template.md`: under `## Acting on it`, "`chat start` starts a chat from\n  the
project's chat template." becomes "`chat start` starts a chat from\n  the project's chat
template, or from the chat template `--template`\n  names." (rewrap the bullet at the page's
width); under `## Limits`, the bullet

```
- temper does not let ctl pick a template: `chat start` fills the chat
  template, and no ctl tool starts a goal.
```

becomes

```
- ctl starts a chat from a chat's template alone: `chat start
  --template` refuses one that is no chat's, and no ctl tool starts a
  goal.
```

## Steps

1. The row's flag and notes; `acts.rs`'s `chat` with its indices and the refusal's text.
2. The step tests; run them alone, then `heavy cargo nextest run -p temper-client-tools --lib`,
   regenerating the help goldens once the row is final.
3. The help pages as given; `heavy cargo nextest run -p temper-docs`.
4. The status line as above.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`;
   `heavy cargo check -p temper-client-shell-cli --tests`; the citations check.
6. Commit (why: ctl starts a chat from any of the project's chat templates); push your branch.

## Report

As the implementer's text says, and also: the goldens regenerated, the rendered flag line, and
the tests' count.

## Answers

2026-11-02. May the task change `src/jig/argv.rs` and `src/tests/jig/choices.rs` for the
shifted chat argument indices? Yes: both are now in the header's files, for the moved indices
alone (see "Owns and may change"). `--template` after `--project` moves the words, `--model` and
`--effort` from 1, 2, 3 to 2, 3, 4, so `check_form`'s `chat start` check becomes
`check_choice(call, 3, 4, limits)`, and `choices.rs`'s `call_words(CREATE_CHAT_CALL, 1)` becomes
index 2 and its expected `Usage` argument indices move by one. Nothing else of either file
changes.
