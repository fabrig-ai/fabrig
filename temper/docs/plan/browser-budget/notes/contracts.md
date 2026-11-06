# browser-budget: the shapes that cross tasks

These are the run's binding calls. `temper/docs/design/client/browser.md`, 4.3, and `temper/docs/design/client/README.md`, 5.15, say why.

## The window

The tier can't set a page's window today: `document_bytes` is a constant in `temper-client-shell-web`'s `limits::client()`. Lowering `document_bytes` itself would bound every document of all seventeen stories, thirteen of which can't be run here, so the seam narrows the conversation's window alone.

```rust
// temper-client-domain/src/limits.rs
pub struct Limits {
    pub forge: forge::Limits,
    pub client: jig_client::Limits,
    pub row_uses: u32,
    /// A conversation's window narrower than `document_bytes` (client/README.md, 5.15).
    /// `None` everywhere but the browser tier's bundle.
    pub window_bytes: Option<u64>,
}

// temper-client-domain/src/pages/chat.rs
pub fn thread_limits(limits: &Limits) -> jig_client::thread::Limits;
// bytes: window_bytes, or document_bytes where it is None.
// lines: those bytes / c::LINE_ROOM, as today.
```

`valid` refuses a named window below `long_text` or above `document_bytes`.

```toml
# temper-client-shell-web/Cargo.toml
small-window = []
```

With the feature, `limits::client()` names `Some(131_072)`; without it, `None`. The tier's bundle turns it on only in `window-story`, in one commit with the stories that fit it: until then, every bundle holds the whole bound.

## A chat's history through ctl

A new module, `temper/tests/client-browser/src/history.rs`, `mod history;` in that crate's `lib.rs`:

```rust
impl Tier {
    /// One `temper ctl` call on this deployment's client directory, `Stdin::Closed`, and its exit 0.
    #[must_use]
    pub fn ctl(&self, words: &[&str]) -> [Step; 2];
    /// The playground running, chat T1 started with `first` and answered,
    /// then each of `then` sent and answered, in order.
    #[must_use]
    pub fn ctl_chat(&self, first: &str, then: &[&str]) -> Vec<Step>;
    /// The first face's script: ctl signs in with the printed link, plays `calls`,
    /// then opens the spent link, so that the server prints the reader's.
    #[must_use]
    pub fn ctl_face(&self, calls: Vec<Step>) -> Script;
    /// `setup()` with the printed link held for ctl (`Sent::Command`).
    #[must_use]
    pub fn held_setup(&self) -> Setup<'_>;
}
/// What the reader's face does first: opens the link the spent one made the server print,
/// then goes to `address`. The harness opens this face once that link is printed, so a page
/// opens only after `ctl_face`'s calls have ended.
#[must_use]
pub fn reader_opens(address: &[u8]) -> Vec<Step>;
```

A story runs `Harness::new(tier.held_setup()).run(&mut [&mut ctl, &mut reader])`. The client directory is `github_connect`'s, `self.scratch.path().join("client")`. `temper/tests/client-browser/tests/browser_all/browser_public.rs`, `sign_in`, `open_printed` and its `ctl` script, shows the steps working in this tier; `temper/crates/temper/tests/fake_backends/walk/chat_document.rs` shows the calls.
