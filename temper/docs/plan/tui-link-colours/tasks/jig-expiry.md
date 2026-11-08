+++
id = "jig-expiry"
part = "jig-expiry"
kind = "interface"
needs = []
files = ["jig/crates/jig-client-protocol-link/src/sign_in.rs", "jig/crates/jig-client-protocol-link/src/component.rs", "jig/crates/jig-client-protocol-link/src/tests/sign_in.rs", "jig/crates/jig-client/src/client.rs", "jig/tests/client/tests/suite/authorize.rs", "jig/docs/design/client/domain/client.status.md"]
effort = "medium"
minutes = 60
+++

# tui-link-colours: the link names its sign-in's expiry

## Goal

jig-client's link view names the expiry of the sign-in it holds, as the last sign-in's
answer said it, so that temper's TUI can tell a sign-in that expired from one that was
signed out (`temper/docs/design/client/README.md`, 6.14, *Expired is not signed out*).
The link layer drops `SignedIn`'s `expires` today.

Acceptance:

- `jig_client_protocol_link`'s link gains `pub fn expires(&self) -> Option<Wall>`, beside
  `credential()` in `sign_in.rs`: set from the answer's `expires`
  (`skein_lib::Wall::from_nanos`) at both `SignedIn` arms (the link's sign-in, token or
  none, and the approval's take in `approval_signing_answer`); none after `Held`; none
  wherever the credential is dropped (`component.rs`, signed out). A field beside
  `credential`.
- `jig_client::LinkView` gains `pub expires: Option<Wall>`, filled in `Client::link()` from
  the link as `credential` is, none unlinked.
- `Signing::SignedIn { party }` keeps its shape: no reader of it changes.
- Rows, each a failing test first:
  - `jig-client-protocol-link/src/tests/sign_in.rs`: a link's sign-in answered with a
    token names the answer's expiry; a tokenless (kept) answer names its expiry too; an
    approval's take answered signed in names it; `Held` names none; a signed-out answer
    drops it. Add them to the existing tests where they already reach the state
    (`a_sign_in_answer_puts_its_credential_on_each_exchange`,
    `a_sign_in_answer_with_no_token_leaves_none_held`,
    `a_held_credential_is_carried_without_an_exchange`,
    `a_signed_out_answer_drops_the_credential`,
    `each_approval_take_answer_is_decoded_without_a_credential` or its neighbour that
    signs in), asserting on `h.link.expires()`; the fixture `signed_in(PARTY, ..)` encodes
    `expires: 10`, so the rows expect `Wall::from_nanos(10)`.
  - `jig/tests/client/tests/suite/authorize.rs`, beside its `client.link().credential`
    assertion: `client.link().expires` is the answer's expiry.
- `jig/docs/design/client/domain/client.status.md`: delete the **Not yet** entry "8.2: the
  link's `expires` ... names none." and nothing else (a stated departure from the
  implementer's text).

## Inputs

- `jig/docs/design/client/domain/client.md`, 8.2 (*Beside its state*, its last sentences)
  and 12 (*`SignIn { with }`*).
- `jig/docs/design/protocol/client.md`, section 8: a sign-in's answer carries its expiry.
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 1.

## Owns and may change

- Owns the files in `files`.
- Readers: `LinkView` is built only in `jig-client/src/client.rs`; a struct literal or
  exhaustive pattern of it elsewhere is a compiler-forced edit, allowed. Nothing else
  reads the new items until `host-local`.
- If `SignedIn`'s parts name the expiry other than `expires` (nanoseconds), use what the
  generated wire gives and say so. If the approval's take's answer carries no expiry,
  leave it none there and say so.

## Steps

1. Write the rows; run them; see them fail for the missing expiry.
2. Keep the expiry beside the credential; fill `LinkView.expires`.
3. `heavy cargo nextest run -p jig-client-protocol-link --lib -E 'test(/sign_in/)'`, then
   `heavy cargo nextest run -p jig-client-world -E 'test(/authorize/)'` (the fixture's
   expiry is `10` nanoseconds).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-client-protocol-link -p jig-client --all-targets -- -D warnings`; the focused tests
   of both packages and of jig's client world; `heavy cargo check -p
   temper-client-shell-tui -p temper-client-domain --tests`.
5. Commit, push.

## Report

As the implementer's text says; name the fixture's expiry and each row's failure seen.
