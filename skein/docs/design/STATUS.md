# Status

Updated 2026-10-29. How far skein's code is from its design.

Each design document's status is in `<doc>.status.md` beside it:
- [`README.md`](README.status.md)
- [`lib.md`](lib.status.md)
- [`kernel.md`](kernel.status.md)
- [`io.md`](io.status.md)
- [`shell.md`](shell.status.md)
- [`simulator.md`](simulator.status.md)
- [`world.md`](world.status.md)
- [`fake-llm.md`](fake-llm.status.md)
- [`kv.md`](kv.status.md)
- [`fake-checkout.md`](fake-checkout.status.md)
- [`testing.md`](testing.status.md)
- [`examples.md`](examples.status.md)
- [`http.md`](http.status.md)
- [`json.md`](json.status.md)
- [`channel.md`](channel.status.md)
- [`codec.md`](codec.status.md)
- [`tls.md`](tls.status.md)
- [`llm.md`](llm.status.md)
- [`llm-connection.md`](llm-connection.status.md)
- [`oauth.md`](oauth.status.md)
- [`oauth-issuer.md`](oauth-issuer.status.md)
- [`forge.md`](forge.status.md)
- [`forge-github.md`](forge-github.status.md)
- [`forge-connection.md`](forge-connection.status.md)
- [`forge-client.md`](forge-client.status.md)
- [`forge-setup.md`](forge-setup.status.md)
- [`fake-forge.md`](fake-forge.status.md)
- [`browser.md`](browser.status.md)
- [`ui.md`](ui.status.md)
- [`ui-text.md`](ui-text.status.md)
- [`ui-web.md`](ui-web.status.md)
- [`lab-browser.md`](lab-browser.status.md)

Not single documents:

| Area | Built | Not yet |
|---|---|---|
| `../foundation/programming-model.md`, `../foundation/testing-strategy.md` | unknown (partly built); the world harness carries state digests and the echo supplies one. | Programming model 10.3: a rules checker. Testing strategy 6: other services' state digests, transition coverage and fuzz targets; checked ownership of heap shared with hosted programs. |
