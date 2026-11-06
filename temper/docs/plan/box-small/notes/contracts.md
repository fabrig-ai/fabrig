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
