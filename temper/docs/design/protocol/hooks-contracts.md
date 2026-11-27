# hooks: the time goal's contracts

2026-11-27. The shapes the changes of the goal that times hooks (`../draft/hooks.execution.md`, H3) share and no other area reads (`../draft/design-economy.md`, 4.3). They bind as `hooks.md` does; the goal's last change deletes this document once the code holds them.

## 1. The time file's reader

In `temper-worker-domain-tooling`, module `hooks`, beside `hook_path`. It reads `hooks.md`, 2.3's file, and nothing else: no I/O, no clock.

- `pub const CONFIG: &[u8] = b".temper/hooks/config.toml";` the file's path in its repository.
- `pub const CONFIG_BYTES: usize = 4096;` the file's bound.
- `pub struct Times { pub deliver: Option<u32>, pub finish: Option<u32>, pub merge: Option<u32>, pub close: Option<u32> }`: each moment's `timeout_seconds` as written, at least 1, `None` where the file has no such table or key.
- `pub struct Fault { pub line: u32, pub why: Box<[u8]> }`: the line from 1, 0 for the file as a whole; `why` the reason's words alone.
- `pub fn times(file: &[u8]) -> Result<Times, Fault>`: the whole file checked by 2.3's form and faults before any value is given; bytes past `CONFIG_BYTES` are `Fault { line: 0, .. }`.
- `impl Fault { pub fn text(&self) -> Box<[u8]> }`: the fault's line, `` .temper/hooks/config.toml:L: WHY ``.
- `pub fn cut_text(moment: &[u8], asked: u32, ceiling: u32) -> Box<[u8]>`: the line saying a value was cut, ``hook `.temper/hooks/MOMENT`: asks A s, past the project's ceiling: runs at C s``, `moment` one of `deliver`, `finish`, `merge`, `close`.

## 2. The figures

In `temper-service-core`'s `limits`, beside `MERGE_DEADLINE`:

- `pub const HOOK_TIME: Duration`, 600 seconds: a `merge` or `close` hook's time where its file is silent, and a fallback command's (`hooks.md`, 2.3).
- `pub const HOOK_TIMEOUT_MAX_DEFAULT: u32 = 3_600;` the ceiling's default, in seconds.
- `pub const HOOK_TIMEOUT_MAX_LIMIT: u32 = 86_400;` the ceiling's greatest, in seconds, and the merge and close kinds' declared deadline (`wires.md`, 8).

## 3. The wire's fields

`temper-protocol-worker-wire`'s `JobSpec.find` and `JobSpec.fallback`, and `Items.hook_timeout_max_seconds`, as `wires.md`, 8 names them, generated from its schema text: their Rust fields carry the same names, `fallback` of `command`'s type.
