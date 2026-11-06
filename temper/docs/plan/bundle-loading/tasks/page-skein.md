+++
id = "page-skein"
part = "page-skein"
kind = "feature"
needs = ["gzip-skein"]
files = [
  "skein/crates/skein-ui-bundle/**",
  "skein/examples/ui/shell-web/loading.html",
  "skein/examples/ui/shell-web/loading.css",
  "skein/scripts/browser.sh",
  "skein/tests/ui-web/**",
  "tools/tiers/run",
  "tools/tiers/test_run.py",
]
effort = "high"
minutes = 75
+++

## Goal

The page and the loader's generic half, in skein: given a placeholder's markup and stylesheet, the bundler writes a page whose first paint needs no script, and a `boot.js` that counts real bytes, starts the client and hands over. Given none, the page as today. skein names no temper: every word is the markup's.

Every commit of this task carries the trailer `Look-request: bundle-loading` as its last trailer, directly above `Co-Authored-By:` where the commit has one.

Acceptance:

- The page and `boot.js` are as `notes/contracts.md`, P1 to P3, say, and a bundle with no placeholder gives the page it gave (with `--title`, its title).
- In a real Chromium, under the example's strict policy, skein's browser tier passes with the example's bundle given a placeholder: `#boot` gives way to the client in every story, with no violation of the policy.
- A module not found says `data-answered`'s words and offers the button, in a browser.
- The browser tier stays within its budget (`tools/tiers/budgets.toml`, `browser-skein`, 145 CPU s).

## Inputs

- `temper/docs/plan/bundle-loading/notes/common.md`, and `notes/contracts.md`, P.
- `skein/docs/design/ui-web.md`, 3.3 (step 4), 4.2, 4.3 and 4.4. `skein/docs/design/examples.md`, 9.3. `skein/docs/design/ui-web.md`, 14.1 to 14.3: what the harness fails on, and `Setup`.
- The request's `boot.readable.js` and `loading.css`, by `git show` (P's opening).

## Owns and may change

- Owns: `files` above. `gzip-skein` has merged; the forms are there.
- Status, as `notes/common.md` says: `skein/docs/design/ui-web.status.md`'s 4.3 line, `examples.status.md`'s placeholder line.
- Readers, a literal's new fields only: `temper/tools/gallery/src/lib.rs` takes `title: None, placeholder: None`, and so do `skein/tests/ui-web/src/lib.rs`, `suite/serve.rs`, `skein-ui-bundle/src/main.rs` and its tests. `tools/tiers/run`'s `bundle`: skein's branch only, and `test_run.py`'s row for it.
- The example's placeholder: `skein/examples/ui/shell-web/loading.html` and `loading.css`, as small as will do: a `#boot` with P3's parts and words in plain English about notes, and a stylesheet. It's a fixture of the loader, not a look.
- Options: if a `Step` the stories need is missing from the fake person, use `Step::See` and `Step::Gone` as they are (`skein/testing/skein-fake-person/src/lib.rs`): they're there. If the harness counts the 404's console line as trouble, add it to the story's expected lines, as `stories::panic` does. If `#boot` trips a kit story that runs in `skein/testing/skein-fake-person-browser`, that's a finding of the design: fix the kit's wait there, and say so.
- Not yours: `temper/**` but the gallery's literal; `temper/scripts/bundle.sh`; `jig/**`.

## Steps

Tests first (`notes/common.md`):

1. `skein-ui-bundle/src/tests.rs`:
   - `a_placeholder_makes_the_page`: P2, whole, against a golden `src/tests/golden/placeholder.html`: no `link` but the placeholder's; `{noscript}` replaced; `#app` before the markup.
   - `the_loader_heads_with_each_files_sizes`: `boot.js`'s first three lines hold the names and the true sizes of the plain file and its `.gz`, and `GZIPPED` equals `PLAIN` with `gzip: false`; the skein-ui.css row is first and the placeholder's stylesheet has none.
   - `no_placeholder_keeps_the_page_and_the_starter`: today's golden `index.html` page and the two statements.
   - `a_title_names_the_page`, with or without a placeholder, escaped.
   - `a_placeholder_is_refused_where_it_cant_hold`: no marker; beside an own page; a stylesheet named `skein-ui.css`; a name holding a control character.
   - `the_page_and_the_loader_hold_nothing_inline`: no `style=` or `onclick=`-style attribute, no `<style`, no inline `<script>` (only the one with `src`) in the page made from the example's markup; no `eval` and no `new Function` in `loader.js`.
   - `main.rs`: the new arguments in the complete list; `--placeholder` alone and `--placeholder-stylesheet` alone in the usage table.
2. `skein/tests/ui-web/tests/browser_stories.rs`:
   - `a_placeholder_gives_way_to_the_client`: from the first page, `Step::See` of a thing of the client, then `Step::Gone` of `#boot`'s words `data-starting`, within a second or two.
   - `a_module_not_found_says_so_and_offers_to_try_again`: `Setup`'s `--bundle` names a copy of the bundle under a `skein_scratch::Scratch`, without `<name>_bg.wasm` and its `.gz`, each of which the example's server then answers 404; `Step::See` `The server answered 404.` and a button named by the markup's "Try again"; press it; the same words again.

Then the code:

3. `Inputs`, the binary's arguments, `Refusal`, `page.rs`, and `loader.js` (P3). The order in `bundle_with`: every other file, then their forms, then `boot.js` with the sizes they have, then its own form, then the digest.
4. The example's placeholder; `skein/scripts/browser.sh` and `tools/tiers/run`'s skein branch give it; `test_run.py`'s skein row.
5. Tier: `python3 -I -B tools/tiers/run skein browser`, once at the end. Report CPU and wall against the budget. Over budget: `notes/common.md`.
6. Once: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-bundle -p skein-ui-web-tests --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-ui-bundle -p skein-ui-web-tests`; `python3 -I -B -m unittest tools/tiers/test_run.py` from the root (or however it runs: `rg test_run .config tools`); `heavy cargo check -p temper-client-gallery --tests`.
7. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit, push.

## Report

What ran; each rule broken by hand and what failed; the tier's figures; `boot.js`'s size, plain and gzipped, against the draft's 2.1 KB; anything of the request's script that had to change besides P3's list.
