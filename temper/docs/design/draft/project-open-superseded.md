# A new project opens only if the person hasn't moved on

Draft, 2026-11-17, on the user's word. temper-box's wiring story (`temper/bin/temper-box/tests/suite/wiring.rs`, `a_scratch_binary_answers_one_chat_and_drains`) failed two landing gates, on 11-16 and 11-17, and passed alone each time. It was a known red on 11-12 too. At its fifth step, where the key form should be, the client showed the new project's Home.

## 1. What happens

- **Making a project opens it:** "+ New project" opens the new project's Home (`temper/docs/design/client/view.md`, 3). The switcher keeps the request pending and, when its answer comes, shows Home (`temper/crates/temper-client-domain-pages/src/pages/switcher.rs`, `told`; `temper/crates/temper-client-domain-pages/src/pages.rs`, `switcher_chosen`).
- **A late answer overrides a later navigation:** navigating elsewhere meanwhile only closes the rename form. The creation stays pending with its opening armed, so an answer that arrives after the person has moved on shows Home over the page they went to. A person who makes a project and goes straight to model setup can hit this.
- **The story's person goes on too early:** the kit takes a project's button in the tree as the choice made, even while the creation is pending (`temper/testing/temper-box-kit/src/person.rs`, `select_project`). Membership can arrive before the creation's answer (`temper/crates/temper-client-domain/src/tests/pages/switcher.rs`), so the script navigates, and the answer's Home lands between two steps.
- **Nothing records the navigation:** the native client exposes readiness counts and a view revision, but not the address it applied (`temper/crates/temper-client-shell-terminal/src/client.rs`).

## 2. Wanted

- **A later navigation supersedes a pending opening:** a navigation or a project choice made after the creation was asked cancels its opening. The creation still finishes, and its project shows in the switcher. Home opens on the answer only if the person is still where they asked it. This holds in both orders of membership and answer.
- **The kit waits for the applied navigation:** the native client exposes, read-only, the address it last applied. The kit's project choice is made once the creation's own navigation is applied, not when a button appears.
- Revise `view.md`, 3, with its status file.

## 3. Out

- The story's patience and every other figure.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The pages domain's switcher tests (`temper/crates/temper-client-domain/src/tests/pages/switcher.rs`):
  - leaving before the creation finishes opens no Home, in both orders of membership and answer;
  - a later project choice supersedes the pending opening, and the creation still succeeds;
  - staying put still opens Home.
- The native client's tests (`temper/crates/temper-client-shell-terminal/src/client/tests.rs`): the applied address follows a Show and a `go`, and a late answer doesn't replace a later page.
- temper-box's support tests (`temper/bin/temper-box/tests/suite/support_tests.rs`): the person's script stays blocked until the creation's navigation is applied.
- The wiring story passes alone, by name.

Run only the new and affected tests, by name.
