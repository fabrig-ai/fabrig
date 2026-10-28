# Building with local jig

temper uses jig from the Git revision recorded in `Cargo.lock`. To work
on both repositories at once, patch jig's Git source with a local
checkout. From the temper worktree, this command writes every package
of jig's workspace into an untracked `.cargo/config.toml`:

```sh
jig_checkout=~/src/rust/jig
mkdir -p .cargo
(cd "$jig_checkout" && cargo metadata --no-deps --format-version 1) |
  python3 -c '
import json
import pathlib
import sys

metadata = json.load(sys.stdin)
members = set(metadata["workspace_members"])
print("[patch.\"https://github.com/fabrig-dev/jig.git\"]")
for package in sorted(metadata["packages"], key=lambda package: package["name"]):
    if package["id"] in members:
        path = str(pathlib.Path(package["manifest_path"]).parent)
        print(package["name"] + " = { path = " + json.dumps(path) + " }")
' > .cargo/config.toml
cargo metadata --offline --format-version 1 > /dev/null
```

`.gitignore` keeps `/.cargo/config.toml` out of commits. While the patch
is in place, Cargo writes jig's packages into `Cargo.lock` as paths:
**never commit that lock.** Save any intentional lock changes before
trying the patch. When local work is done, remove the patch and let
Cargo restore the Git sources. Confirm jig's intended commit and the
pin rule below:

```sh
rm .cargo/config.toml
cargo metadata --format-version 1 > /dev/null
```

## The pin rule

A repin goes skein, then smith, then jig, then temper. The manifests
name their Git URLs without `rev`, `branch` or `tag`; `Cargo.lock` holds
the commits. Keep the URLs spelled exactly as they are in the manifests,
so Cargo treats shared dependencies as the same source.

temper must pin a jig commit whose `Cargo.lock` names the same skein and
smith revisions as temper's. From the temper worktree, compare the
distinct source lines against jig's lock at the pinned commit (Bash):

```bash
jig_checkout=~/src/rust/jig
jig_commit=$(sed -nE 's|^source = "git\+https://github\.com/fabrig-dev/jig\.git#([0-9a-f]+)"$|\1|p' Cargo.lock | sort -u)
diff -u \
  <(sed -nE '/^source = "git\+https:\/\/github\.com\/fabrig-dev\/(skein|smith)\.git#[0-9a-f]+"$/p' Cargo.lock | sort -u) \
  <(git -C "$jig_checkout" show "$jig_commit:Cargo.lock" |
    sed -nE '/^source = "git\+https:\/\/github\.com\/fabrig-dev\/(skein|smith)\.git#[0-9a-f]+"$/p' | sort -u)
```

No diff means the revisions match. Check without the local patch, when
temper's lock contains jig's Git commit.

The stage 1 startup admission integration has an explicitly approved pin
exception: jig `4be07c1fb71dccfd433c253b5e6abd90c965405e`, while temper
retains smith `f1d8d29bf88ce2724d363c724cc4dd9fcbdc377e` and skein
`9d34791ec9bae7253917270ccee56668b0f4115d`. That jig commit's own lock uses
smith `3ffd88c34fbbd3f9d622db40ec73c7189c41999f` and skein
`7218a31f7b73e6c27723466a8a59fc5161ba9cfe`; the comparison above therefore
reports this known skew. The exception authorizes no further upstream
movement; a later coordinated repin must restore the matching-lock rule.
