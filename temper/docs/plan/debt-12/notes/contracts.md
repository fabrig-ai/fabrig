# debt-12: the link's contracts

Binding for the tasks of the parts `link`, `forge` and `owners`. The face is
`skein/docs/design/link.md`, 3.1 and 3.4. `link-surface` carries the additions; an owner's task reads them
this way, and no other.

## 1. The face

```rust
impl Link {
    /// `next_deadline` has come: `fire` at `now` aborts.
    #[must_use]
    pub fn due(&self, now: Time) -> bool;
}
impl Upload {
    /// A request's optional body.
    #[must_use]
    pub fn of(body: Option<Box<[u8]>>) -> Option<Upload>;
}
```

`Link::socket` goes in `link-ending`, once no crate reads it. Nothing else of the face changes: `Upload::new`,
`next_deadline` and the rest keep their signatures.

## 2. How an owner reads it

- **Is the close due:** `c.link.due(env.now)`, where a `match` on `next_deadline` says it. `next_deadline` stays
  where a deadline is wanted.
- **The link's limits:** the function stays, one expression, built with `Env::with`:

  ```rust
  fn link_env(env: &Env<Limits>) -> Env<skein_link::Limits> {
      env.with(skein_link::Limits { tls: env.limits.tls, close_timeout: env.limits.io.close_timeout })
  }
  ```

  The forge connection's reads `self.shared.io.close_timeout`, as it does now.
- **An optional body:** `Upload::of(body)`, where a `match` on the body says it. An `#[expect(clippy::manual_map)]`
  that spoke of that `match` goes with it.
- **The end of an upload:** `next == Down::Finish`, on `room`'s second value, with no arm for `Send`. What each
  owner does at the end and in which order stays as it is.

## 3. What the review's findings came to, read against the code

The review was read, not run; so was this. Each task still reports what it saw run.

| Finding | Reading | Task |
|---|---|---|
| 1 | Right: `link.fire` and `next_deadline` in the forge's component have no test | `forge-close` |
| 2 | Right. `Moved` and `Event` also derive `Debug` over `Up`'s bytes, and the review doesn't ask for them: they aren't kept, and stay | `link-surface` |
| 3 | Right: only `pump_call` and its like pop `heard`, each with its own wake, so a closing connection's `heard` needs none | `forge-close` |
| 4 | Right: `held` is what `ending` says while `socket` is `None`, and it closes `link.status.md`'s 3.3 | `link-ending` |
| 5 | Right for three of the four sites. oauth's already reads `due(web.next_deadline(), now)`, through `Web`, which hides its link and has other deadlines to test with its `due`: it stays, and only `link_env` changes there | `llm-tidy`, `oauth-tidy`, `setup-tidy` |
| 6 | Right, but `Link::socket` has two readers more, in the link world, and an integration test can't reach `#[cfg(test)]`: it is removed, not made test-only | `forge-close`, `llm-tidy`, `link-ending` |
| 7 | Left alone, by the request. `link.status.md` records it under section 1 | none |
| 8 | Right: it restores the order the connection had before it stood on the link, but for a closing connection, whose link must drain | `forge-close` |
| 9 | Right | `forge-tidy`, `setup-tidy` |
| 10 | Right. The llm world hands a `Stream` after `Close` to the wire, which is why that test can't fail on order | `llm-tidy`, `setup-tidy`, `link-ending` |
