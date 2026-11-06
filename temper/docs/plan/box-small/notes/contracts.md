# box-small: the shapes that cross tasks

## The command's words (`printed-words`)

`ProgramEnvironmentAnswer.environment` stays one opaque string, and no wire field changes: the service and the domain carry it as they do. Only its two ends change.

- **The maker,** `temper/crates/temper/src/printed.rs` (new), with the rules of `temper/docs/design/client/README.md`, 4.6, *The words of a command for this machine*.
- **The drawer,** `temper-client-view`'s `pages/connect_model.rs`, `command`: an empty answer draws `temper program-login NAME`, as today; any other draws the answer, a space, `program-login NAME`.

### If `first-usable-model` is in your base

That draft is in the queue and adds `DeploymentAnswer.program_login`, `program_login::line` (`temper/crates/temper/src/program_login.rs`), `printed_command` and `command_line` (`temper-client-view`'s `pages/connect_model.rs`). If `program_login::line` is on your base, or whichever of the two lands second, the two meet like this:

- `line` keeps its flags and its bound, and takes this draft's words: the binary by `printed::running_word`, each directory by `printed::path_word`, in place of its whole-quoted forms.
- `printed_command`, in this order: the owner's answer when it has come (`program_environment_answered()`): empty draws `W::EnvironmentCommand` and NAME, any other draws the answer, a space, `program-login NAME`; else `program_login` and NAME; else the placeholders, `W::Command`. It never joins the answer to `program_login`: each says the binary and the directories by itself.
- `first-usable-model`'s tests that pin `'/box/bin/temper' program-login --config '/box/config' …` stay, those paths being unquoted now where they can be.
- `temper/docs/design/client/README.md`, 4.6, *The printed command* takes the same sentence: "each as a word (*The words of a command for this machine*)" in place of "each quoted for a POSIX shell".

## The clipboard (`copy-part`, read by `copy-button`)

`skein/crates/skein-ui-web/src/clipboard.rs`, naming no application:

```rust
//! The clipboard written, one effect's part (skein/docs/design/ui-web.md, section 11).
#[cfg(target_arch = "wasm32")]
pub mod web;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Done, Refused, Failed }

/// One write's one report, carrying its token (skein/docs/design/ui.md, section 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report { pub token: u32, pub outcome: Outcome }

/// What a rejected promise's error name says.
#[must_use]
pub fn rejected(name: Option<&str>) -> Outcome;
```

`clipboard/web.rs`, as each part is made (`skein/docs/design/ui-web.md`, 5.1):

```rust
pub struct Clipboard<R> { /* the inbox, the wrap */ }
impl<R: 'static> Clipboard<R> {
    #[must_use]
    pub fn new(inbox: Rc<Inbox<R>>, wrap: fn(Report) -> R) -> Self;
    /// Asked in the turn of a person's act; one report through the inbox, at once when refused.
    pub fn write(&self, token: u32, text: &[u8]);
}
```

## The Copy (`copy-button`)

`temper/crates/temper-client-domain/src/copy.rs`, new, `pub mod copy;`:

```rust
pub const SHOWN: Duration = Duration::from_secs(2);
/// What the shell's clipboard said.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Written { Done, Refused, Failed }
/// What the button reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Label { Copy, Copied, NotCopied }
```

- `boundary.rs`: `Action::Copy { words: Box<[u8]> }`, `Event::Copied { token: u32, written: copy::Written }`, `Request::Copy { token: u32, words: Box<[u8]> }`.
- `Domain` holds one `copy::Copy`, whose token counts up at each press, and reads `pub fn copy_label(&self) -> copy::Label`. A report for any but the last press's token is dropped. `Done` shows `Copied` until `now + SHOWN`; `Refused` and `Failed` show `NotCopied` until then. `next_deadline`, `fire` and `shown` each take it in, so that the view rebuilds (`temper-client-view`'s `root.rs` compares `shown`).
- `host::Report::Copied { token, written }`, `host::Duties::copy: Option<(u32, Box<[u8]>)>`. The press's report returns the duty.
- Two reserved names in `temper-client-view`'s `look.rs` become the block's: `temper-reserved-367` is `temper-welcome-copy` (`WELCOME_COPY`, token 367), `temper-reserved-121` is `temper-welcome-cmd-box` (`WELCOME_CMD_BOX`, token 121). No row is added at the table's tail, where other drafts are. If another draft has taken either on your base, take the next `temper-reserved-` row and say so.
