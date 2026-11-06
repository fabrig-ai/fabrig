# debt-28: a file reader's move to `file_layer::step`

What the `file-readers-*` tasks share. `skein/docs/design/io.md`, 5, is the contract. `file_layer::step` and `FileIo::with_limits` are public today, and the thin forms work beside them on one `FileIo`, so each reader moves alone and in any order. The thin forms go last, in `file-thin-gone`: don't touch `skein-io/src/file_layer.rs`.

The move changes no behaviour: the thin forms build exactly this `Env`, and a file step reads `now` only in `down` and `expire`, and `wall` nowhere.

## The recipe

**A constructor.** `FileIo` gives its limits back to no one, so the reader keeps them.

```rust
// FileIo::new(files, read, entries, timeout): `whole` was `read`, so write it out.
let file_limits = file_layer::Limits { files, read, entries, whole: read, timeout };
let io = FileIo::with_limits(&file_limits);
// FileIo::with_whole_limit(files, read, entries, whole, timeout): the same, in that field order.
```

The reader holds `file_limits: file_layer::Limits` beside its `FileIo`, or in an env it already keeps. `file_layer::Limits` is `Copy`. Where the reader's own limits already hold the five fields, as `hearth-service-builder`'s `limits.files` does, build it from them rather than storing a copy of them twice.

`with_limits` asserts `file_layer::valid`, as `new` and `with_whole_limit` already do through it, so no refusal changes. `FileIo` is the same value either way: `with_effective_user` and the rest chain as before.

**A call.**

```rust
// file_layer::down(io, now, request, events, subs)
step::down(io, &Env { now, wall, limits: self.file_limits }, request, events, subs);
// file_layer::down_until(io, deadline, request, events, subs)
step::down_until(io, &env, deadline, request, events, subs);
// file_layer::up(io, complete, events, subs)
step::up(io, &env, complete, events, subs);
// file_layer::expire(io, now, subs)
step::expire(io, &env, subs);
```

`step` is `skein_io::file_layer::step`; `Env` is `skein_lib::Env`; `Time` and `Wall` are skein-lib's.

**The clock.** Give `now` and `wall` where the function has them. Where it has none, as seven `up` sites don't, give `Time::ZERO` and `Wall::EPOCH`, which is what the thin form gives. Don't add a parameter to a function to carry a clock it wouldn't read: `temper-client-shell-cli`'s `Directory::complete` is called from `host.rs`, another draft's file, and keeps its signature. Don't add a clock to a reader that keeps none.

**Where a reader already has an env,** such as `io_env` or `machine_env`, a file env beside it is one field set where that one is set. Where a function steps twice, one `let env` serves both. A one-line helper in the reader that builds the env is fine, and better than five literals.

**A comment** that names a thin form changes with it. A test keeps what it asserts: it changes only how it reaches the file layer. A literal `Time::ZERO` a test already passes stays.

## Size

A reader gains a field, and an env per function, so it can't end with fewer lines, and the request's aim doesn't hold for these tasks: your spec's cap does. Your report gives the net.

## What each task finishes on

Each tree's search finds nothing outside what its spec exempts, and every package you changed passes its focused tests:

```
rg -n 'file_layer::(down|down_until|up|expire)\b|FileIo::(new|with_whole_limit)\(' --type rust <the tree>
```

The item stays open until `file-thin-gone`, which deletes it.
