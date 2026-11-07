# ui-small-1: the plan

The request: what the user met on their own box, and a flake (`inputs/request.md`). Three tasks at most, focused and fuzzy alone, but for item 3's story by name.

## Parts and tasks

| Part | Task | Needs | Request item | What it does |
|---|---|---|---|---|
| small-words | `small-words` | | 2, 4 | the approver line, a chat's forge look, Home's repeat, a repair's blame |
| printed-link | `printed-link` | | 3 | a count of links, stderr in the panic, a fault fixed if found |
| link-why | `link-why` | | 1 | the engine answers why a link can't sign in, through the wire and the client's view, to the page's words |

## Critical path

None to speak of: the three run side by side, each alone.

## Decided

- **A repair of a semantic conflict** says "Repairing: CI failed at its updated head.", the request leaving the words to the cut. It has no hole: naming the base was the blame. The domain's word for the reason stays, since only the person's line misled.
- **Home's empty *Needs you*** loses its first sentence and keeps the rest; the headline keeps "Nothing needs you." This departs from the mock, which says it twice (`temper/docs/design/client/README.status.md`).
- **The approver sentence** stays where it is, in the paragraph on reading the inventory, with its condition added: a test reads its first half.

- **The engine tells the links apart.** The client couldn't: every secret not live was answered `refused`. The component now keeps the digests of its last 8 past links, in memory. Nothing else could tell a used link from an older box's without persisting something, and the words don't warrant a stored format.
- **Three answers, not two.** The user ruled on a used link and on one that isn't this box's. A link past its 2 hours is neither, and either word would be false for it, so it gets a third, in the same form: "This link has expired. A fresh one is in your terminal." These words are the cut's, not the user's.
- **A made-up link takes the same path.** A `secret` that doesn't decode is sent as an empty one and answered `link_unknown`, so no new client state is needed. It can now draw a fresh link where none is live, as any other unknown secret could, which makes "Use the one its terminal printed" true.
- **A restart forgets.** A used link opened after serve restarts reads "isn't from this temper". What it tells the person to do stays right, and it is what remembering in memory costs.
- **With the printed link not enabled,** the answer stays `refused`, so a deployment that signs in with Google keeps its sign-in page for a stray link.
- **The wire changes in place,** as jig's pre-release rule has it. This is a protocol change, with no compatibility, as the request allows.
- **`temper open` and `temper ctl` keep their words:** the ruling is about the page. They could say why later, now that they're told.

## Seen, left

- **A live cookie and a bad link.** A person whose cookie is live, opening a link that can't sign in, is shown the paragraph until they reload, since a link's sign-in doesn't fall back to the cookie. That is how a spent link behaves today. It now also holds for a made-up link, which used to sign in by the cookie. A fall back to the transport would fix both, and is worth its own request.
- **The second link's cause is unproved.** `printed-link` reads for it, and its spec lists what the cut read. If nothing is proved, the next red's panic says why.

## Overlaps

Could not be avoided, since the request names these files:

- `temper/crates/temper-service/src/stock/configuring.txt`: `ui-proposal-2/chat-revises` and `model-catalogue-2/configuring-learns`. One sentence here; whichever lands second catches up, as the request says.
- `temper/crates/temper-service/src/tests/stock.rs`: `ui-proposal-2/chat-revises` and `limits-fixtures/service-fixture`. A row and a pinned length.
- `temper/crates/temper-client-view/src/words.rs`: `limits-fixtures/view-words`, `look-round-13/forge-cards` and `ui-proposal-2/card-and-latest`. Single lines.
- `temper/tools/gallery/**`: several plans. One row, regenerated on a catch-up.
- `temper/crates/temper/tests/process.rs`: `limits-heavy/box-lifetime`. Two assertions.
- `temper/crates/temper/tests/support/**`: `model-catalogue-2/process-fixtures`. Three new functions beside `printed_link`.
- `jig/crates/jig-client-protocol-wire/schema/**`, `generated/**`, `golden/**`: `model-catalogue-2/config-models`, `model-act` and `model-surface`. One enum line; generated files are regenerated on a catch-up.
- `jig/crates/jig-protocol-engine-clients/src/component.rs`: `model-catalogue-2/catalogue-clients`. The link's arm only.
- `temper/crates/temper/tests/support/person.rs` and `managed_client.rs`: `model-catalogue-2/process-fixtures`. One arm each.
- `temper/docs/design/client/README.md`: `look-round-13/look-proof`. Two separate passages.
- `printed-link` and `link-why` meet in `component.rs` only if the second link's fault proves to be in the engine's link code. That spec says how.
