+++
id = "d004-rewrite"
part = "d004-rewrite"
kind = "docs"
needs = ["temper-root-shared", "service-identity", "composed-sender", "job-refused", "ops-protocol", "charter-parts"]
files = ["jig/docs/development/debt/D004-ops-root-jig-wiring.md", "jig/docs/design/domain/root.status.md", "jig/docs/design/domain/hosts.status.md", "jig/docs/design/examples.status.md"]
effort = "medium"
minutes = 25
+++

# jig: D004 says what is left, and the status files follow

## Goal

The last of part 3. D004 stays open for the test root, as the request says: its rebuilding is deferred, by the user. The
item and the status entries it names say what is true on the trunk once this plan's jig and temper tasks have landed.

Acceptance:

- The item's `Status` reads `open`; `python3 -I -B tools/debt/debt.py` lists it.
- `Paid when` names no delegate JSON, alert word, fixed refusal or Temper copy.
- `python3 -I -B tools/citations/citations.py --all` reads 0.
- The file keeps its name: `root.md`, `hosts.md` and three status files cite it.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `temper/docs/plan/debt-16/inputs/request.md`, part 3.
- `DESIGN.md`, 2.
- `jig/docs/design/domain/root.md`, 3, 5, 11 and 14; `jig/docs/design/domain/hosts.md`, 12.

## Owns and may change

This departs from the implementer's text: these are status files, and you write them.

- **Owns:** `jig/docs/development/debt/D004-ops-root-jig-wiring.md`.
- **May change, what's named below and each file's date:** `jig/docs/design/domain/root.status.md`,
  `jig/docs/design/domain/hosts.status.md`, `jig/docs/design/examples.status.md`.
- No code. If a claim below doesn't hold on the trunk, write what does and say so in the report.

## Steps

1. Check each of these on the trunk, then rewrite D004:
   - `Checked:` today, at the trunk's short hash. `Size` and `Value` stay.
   - `Paid when:` the user has decided the test root's store-answer queue, public reads and release order, and both roots
     follow the resulting shape for restart and the remaining `Now` and fleet hand-offs; shared message and document
     assembly lives once at its owners' boundaries.
   - `Measure:` the command, with the number it reads now.
   - `What`: the bullets on the test root's shape, restart, the `Now` and fleet hand-offs, the test root's `link.rs` and
     `client_documents.rs` stay. The bullets on `host_out` and `translate.rs` go. Temper's bullet becomes what it keeps,
     and why: `message`, `inbox_sender` and `inbox_words` render other words than `jig_protocol_charter::message`;
     `temper-service`'s `answered` waits on `hosts.md`, 12, which cites this item.
   - A new first bullet: rebuilding the test root is deferred by the user; this item stays open for it.
   - A new bullet: `ops`'s root calls `jig-ops-protocol-observability` in the step rather than handing its delegates out.
     A hand-out would put the connector's rows and the procedure step in two commits, and a crash between them may re-offer
     a step the connector has recorded. Whether that's wanted is the same user decision on order.
   - `What pays it back`: without the sentences on delegates, refusals and Temper's consumers.

2. `root.status.md`:
   - **Not yet:** the 3 and 5 clauses go, with "The code's side of both is ...". 2, 5, 6 stays.
   - **Departs:** 11, 12 becomes: `ops`'s root passes a triage's delegate batch through
     `jig-ops-protocol-observability` in the step, rather than handing it out, because a hand-out would commit the connector's
     rows and the procedure step apart (the code's side is the item).

3. `hosts.status.md`: **Not yet**'s "9: temper's ... `Party::sender`." goes; "The code's side of both" becomes "The code's
   side" for the test root's half, which stays.

4. `examples.status.md`: **Departs** 4 goes. **Not yet**'s `ops` standalone clause says "its connectors' protocols, but for
   the words and delegates of `jig-ops-protocol-observability`".

5. `python3 -I -B tools/debt/debt.py`; the citations check. Commit and push.

## Report

The common note's items, with the Measure's number, and any claim above that didn't hold.
