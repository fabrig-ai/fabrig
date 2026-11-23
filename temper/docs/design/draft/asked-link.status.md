# asked-link: status

2026-11-17. Where the design lags or departs from [asked-link.md](asked-link.md).

The design carries this draft. Section 2's asked link is jig's
`protocol/client/sign-in.md`, *The asked link*: asked by `LinkAsk` with
a person's credential, refused to a service's or none, enabled where
the printed link is; signing in the asker's party, held none, in a
sign-in people caps at the asker's saved expiry (jig's
`domain/people/README.md`, section 3, `until`); one live per sign-in,
asking again replacing it, the one replaced remembered expired; at
most 4 live in the deployment, an ask past them busy; its digest in
memory alone; answered as the printed link is. Its route is
`POST <prefix>/v1/link/ask` (jig's `protocol/client/default-http.md`,
1) and its records are on jig's client wire 28 (jig's
`protocol/client/wire.md`, 2.1). The client's `AskLink` is jig's
`client/domain/client/sign-in.md`, section 1. ctl's `link --out FILE`,
its file and its refusals, and the refusals naming both ways to a fresh
link, are `../client/ctl/command-line.md`, 1.1, and `../client/ctl/README.md`, 7, and `../client/ctl/host.md`;
temper's deployment is `../protocol/sign-in.md`, 2.3. The refusal of
an unknown link names both ways too, as the spent and expired ones do.

- **Design departs:**
  - 4: "none off loopback" is pinned in jig's engine clients suite as
    none where the printed link isn't enabled: the refusal of a link
    off loopback is the application's startup's (jig's
    `protocol/service.md`, section 3), already tested there, and the
    suite's kit listens on loopback alone. Decided by the plan's cut.
