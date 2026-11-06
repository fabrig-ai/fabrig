# ui-repo-link: the shapes that cross tasks

The run's binding calls. Each is the design's (`temper/docs/design/protocol/sign-in.md`, 9.9), in the form a task builds to.

## skein: the issuer (`provider-public`)

```rust
pub struct Client { pub id: Box<[u8]>, pub secret: Option<Secret>, pub redirect_uri: Box<[u8]> }
```

`None` is public. Nothing else of `skein_oauth_issuer` changes shape.

## jig: the configuration (`provider-public`)

```rust
pub struct ProviderClient { pub id: Box<[u8]>, pub secret: Option<Box<[u8]>>, pub redirect_uri: Box<[u8]> }
```

`"secret": null` in the document, never absent. `addresses` takes a provider of kind application or link.

## temper: the box's document (`box-sign-in`)

As `sign-in.md`, 9.9 shows it. `application.machine.owned.sign_in`: a client's id, or `null`; absent reads as `null`, so an older document and every fixture stay as they are. `OwnedMachine` gains `pub sign_in: Option<Box<[u8]>>`.

## temper: the box's line (`box-sign-in`)

`source FORGE added\n`, `source FORGE failed WHY\n` (`setup.md`, 7.1). What the `source` step itself says on its stdout: `source added\n`, `source failed WHY\n`.

## temper: the step's command (`box-sign-in`)

`temper forge source DIRS N`. It exits 0 whichever way it ends and says one line. Forgejo's commands take no secret.

## temper: the page's facts (`box-sign-in`)

`<meta name="temper-forge-sign-in" content="NUMBER CLIENT ROOT owned">`, 9.3's, once the source is there. `<meta name="temper-forge-sign-in-off" content="NUMBER">` while the box's owned forge is kept and has no key or no source. The first is what makes "Open Forgejo ↗" and a repository's link through temper appear (`repo-link` reads it through `Forges::number`).
