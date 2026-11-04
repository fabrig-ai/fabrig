# macos-box-start: the shapes that cross the two tasks

Binding for both; `skein/docs/design/shell.md`, 6.6 is their design.

In `skein-shell`, exported from `lib.rs` beside `ListenersRefusal`:

```rust
/// The call of a listener's check that failed (shell.md, 6.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListenerCheck { Stat, Kind, Listening, Address }

impl ListenerCheck {
    /// The call, for a refusal's words.
    pub const fn name(self) -> &'static str;
    // Stat "fstat", Kind "getsockopt SO_TYPE",
    // Listening "getsockopt SO_ACCEPTCONN", Address "getsockname"
}

pub enum ListenersRefusal {
    // every variant as it stands, and:
    /// A descriptor check's call failed with this errno.
    Check { fd: Fd, check: ListenerCheck, errno: i32 },
}
```

- **Additive:** no variant changes its fields. `Kernel { fd, errno }` stays, and from here on holds the
  close-on-exec update alone; a check's failure is `Check`. `EBADF` is still `NotOpen`, and
  `EAFNOSUPPORT` still `NotIp`, from any of the four calls.
- **Readers:** the only match on the enum outside `skein-shell` is `listeners_refusal`,
  `temper/crates/temper/src/lib.rs`. hearth, smith and jig name neither the enum nor its functions.

temper's words, built by `held-refusal`:

| A refusal of | Reads |
|---|---|
| an inherited descriptor (`inherited_listeners`, and `forge run`'s fd 3) | `LISTEN_FDS: fd 3 is not a socket`, as today |
| a descriptor the box holds itself | `held listener: fd 4 is not a socket` |
| a failed call | `<prefix>: fd 4: getsockopt SO_ACCEPTCONN: errno 42` |
| a failed close-on-exec update | `<prefix>: fd 4: close-on-exec: errno 42` |
