+++
id = "link-words"
part = "link"
kind = "docs"
needs = []
files = [
  "temper/docs/help/landing-rule.md",
  "temper/docs/help/configure.md",
  "temper/docs/help/party.md",
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-docs/src/tests/rules.rs",
  "temper/docs/design/domain/agent.status.md",
  "temper/docs/design/draft/forge-identity.status.md",
]
effort = "medium"
minutes = 45
+++

# The help and the configuring agent say what a box's person needs: nothing

## Goal

On a box, temper knows its person's account on its own Forgejo and on a GitHub App it made from their account (`temper/docs/design/domain/forge.md`, 4.6), so a rule asking for their approval needs no collaborator and no adopting again. The help and the configuring agent's instructions say so, in the person's words, with section 6's sentence, and keep today's collaborator route marked as for a deployment where no link vouches for a person: `temper serve` with several members, a managed Forgejo, a pasted App. The code that makes the link is other tasks'; these are words only.

This task departs from the implementer's text in two ways: `docs/help/` pages are yours to change as below, and two status entries are yours to edit (acceptance 5), because the design already says what this task builds.

Acceptance:

1. `temper/docs/help/landing-rule.md`, the paragraph beginning "An enforcing approval rule", says, in this order and the page's voice:
   - on a box, temper knows the person's account on the box's own Forgejo, and on GitHub when temper made the App from their own account: a rule asking for their approval names their role, owner (0) on a project they made, and needs nothing added;
   - the sentence, whole: "On this machine temper's agents and CI jobs run as you, so this stops their mistakes, not one trying to get past it." (`temper/docs/design/draft/forge-identity.md`, section 6, accepted by the user on 2026-11-07);
   - acceptance counts people on the forge of each repository the rule covers, and its refusal names the forge;
   - a rule covering the playground's forge is refused: no one can approve there;
   - then, marked "With several members (`temper serve`), a managed Forgejo or a pasted App", today's route: the human collaborator, temper's forge user's admin on Forgejo, the project-policy mapping, adopting again, `project show`, a fresh proposal; the stock mapping and the writer's sentence as they are.
   The sentences "A deployment owner login or a matching display name does not establish a forge identity." and "temper's own forge writer never counts, even with an existing role." stay.
2. `configure.md`'s *Human approvals* and `party.md`'s *Forge people for approvals* open with one sentence saying a box's person needs none of this (temper knows their account on the box's Forgejo and on a GitHub App it made from their account), and mark the rest as being for several members, a managed Forgejo or a pasted App. No fact of either is lost.
3. `temper/crates/temper-service/src/stock/configuring.txt`'s *Approvers* paragraph says, as `temper/docs/design/domain/agent.md`, 8.1 now has it: a person the inventory shows with an account on the repository's forge, as a box's person, needs nothing added: propose a rule naming their own role, owner (0) on a project they made, and say no more of forge identity; the collaborator, admin and adopting-again steps are for someone with no account the inventory shows on that forge. "Login/box ownership: no eligibility" becomes "A login alone is no eligibility". Every other instruction stays.
4. The tests reading these hold the new words: `approval_help_names_the_forge_and_project_policy_route` (`temper-docs/src/tests/rules.rs`) also requires the box's sentence and section 6's sentence on `landing-rule.md`, and the marking on all three pages; `configuring_txt_names_complete_approval_adoption_steps` (`temper-service/src/tests/stock.rs`, or whichever of its three `configuring_txt_*` tests reads *Approvers*) also requires the box's instruction. Each new requirement is a row or a phrase of the existing table or list there, and bites (remove the sentence; see it fail).
5. In `temper/docs/design/domain/agent.status.md`, the entry *8.1: the instructions (`stock/configuring.txt`) send every person who'd approve through a collaborator ...* is gone; in `temper/docs/design/draft/forge-identity.status.md`, the 9 entry's sentence "The help's `landing-rule.md`, `configure.md` and `party.md` speak for every deployment." is gone, the rest of the entry unchanged.
6. Each page stays within its bound, shortened if needed losing no fact. Focused tests pass in `temper-docs`, `temper-service` and `temper` (`--lib`, its help tests), and in `temper-engine-domain-world`, whose configuring stories deliver this help and these instructions.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 8.1, *Its instructions* (who approves; whenever someone is to approve or join) and section 10, *A person known by a login alone*.
- `temper/docs/design/domain/forge.md`, 4.6 and 8.9 (the refusal per forge; the playground's forge).
- `temper/docs/design/draft/forge-identity.md`, sections 2, 6 and 9 (the words of section 6; what `temper serve` keeps).

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:**

- `temper/tests/engine/domain/tests/suite/configuring.rs` (`approval_documentation`, `approval_context`) and `temper/crates/temper/src/tests/help.rs` read these pages: if a phrase they require moves, change the phrase there to the page's new words and nothing else, and name it in the report. The story there is `box-story`'s.
- `docs/help/reference/*` regenerated if a page's change makes it stale.

**Options, each with its default:** where the page's voice says "you" or "the person", keep its voice; section 6's sentence keeps "you" as the draft has it.

## Steps

1. The three pages, then the instructions, each in the person's register.
2. The tests' requirements (acceptance 4), and their bites.
3. The status entries.
4. `cargo fmt`; `heavy cargo nextest run -p temper-docs -p temper-service -p temper-engine-domain-world`; `heavy cargo nextest run -p temper --lib`; `heavy cargo clippy -p temper-docs -p temper-service --all-targets -- -D warnings`. Commit ("temper: the help and the configuring agent say a box's person needs nothing added to approve"), push.

## Report

As the implementer's text says, with each page's bytes against its bound.
