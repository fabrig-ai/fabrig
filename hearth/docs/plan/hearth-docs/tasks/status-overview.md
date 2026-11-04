+++
id = "status-overview"
part = "status"
kind = "docs"
needs = ["citations-docs"]
files = ["hearth/docs/design/README.md", "hearth/docs/design/README.status.md", "hearth/docs/design/client/README.md", "hearth/docs/design/client/README.status.md"]
effort = "high"
minutes = 40
+++

# Status for the overview and the client

## Goal

`hearth/docs/design/README.md` and `client/README.md` get status files checked against the code, close with `Open questions`, and lose
their build state. The overview's status also records what `citations-code` and `citations-docs` left as they were.

Acceptance: `notes/status.md`, its section "Acceptance, in every status task". Beyond it, `README.status.md` holds the **Departs** of
step 3.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/status.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 2.2 and 3
- `DESIGN.md`, 1 and 2

## Owns and may change

- **Owns:** the four files under `files`. Section 6's conventions are `citations-docs`' and stay; `hearth/docs/design/README.md`, 3's
  milestones stay.
- **Not to touch:** `hearth/docs/design/STATUS.md`, code.
- **Options:** `notes/status.md` settles what's a gap. `hearth/docs/design/README.md`, 8's asks are design and stay: an ask skein doesn't
  meet yet, and hearth's code would need, is a **Not yet** on 8.1, by name, and an ask met isn't listed.

## Steps

1. **Seed, `README.md`:** built: 4.2: render's libraries, the builder root, the protocol layer and the service. Not yet: 8.1: remaining
   crates and support. Compare section 4.2's crates to `hearth/crates`, and name those not built. `hearth-domain-artifacts` is M6's, marked,
   and needs no entry if the document says so.
2. **Seed, `client/README.md`:** built: 3.1 to 3.3, 3.5: in-process render and diff. Not yet: 3.5: the proxy; the remaining client, retarget
   and development commands. Code: `hearth`, `hearth-client-protocol-records`, `hearth-service-builder`.
3. In `README.status.md`, a **Departs** entry on 6: the 32 errors of `hearth/crates/hearth-client-protocol-records/src/errors/reserved.rs`
   and `errors/repository.rs` print a document as `protocol/format.md, 9.1` does, and the `doc` strings of task JSON the same way, because
   a printed error and a digest's input are hearth's behaviour.
4. Follow `notes/status.md`, 1 to 4, for each document.
5. Commit (`hearth: overview and client's status`), run the checks in `notes/common.md`, push.

## Report

As `notes/status.md` says.
