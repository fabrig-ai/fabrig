# The client's binary

This document holds the client binary's interface, split from [`binaries.md`](binaries.md).

## 1. The client's binary

- **The client, `temper`** (`bin/temper`, its code
  `crates/temper-client-shell`; `../draft/cli.md`, 4.1;
  `../draft/deployment-binaries.md`, section 4 and section 9's first
  item), a binary of its own for a person's machine or an automation's,
  and the box's front. Its table, its sets, its help and its exits are
  `../cli/README.md`'s (7.4):

  ```
  public         temper [ADDRESS]
                 temper box [ARGUMENT...]
                 temper help [NAME | pages | --search WORDS...]
  internal       temper ctl DIRECTORY GROUP VERB [ARGUMENT...] [--json]
                 temper ctl DIRECTORY --call FILE [--json]
                 temper ctl help [GROUP [VERB]]
  spawned        temper internal fetch-box [--cache DIR]
  ```

  - **`ctl` is one row both tables hold,** run by the same code
    (`components.md`); its own help and its lines (`temper ctl: `) name
    `temper`, whichever binary runs it.
  - **`temper --version`** prints one line, `temper VERSION (COMMIT)`, as
    `../release.md`, section 3, gives every binary's: VERSION and COMMIT
    `temper-version`'s (1).
  - **A first word that is no row of its table** is the bare form's
    address, refused, exit 2, when it isn't one (`../cli/README.md`,
    7.4): `serve`, `open` and `init` among them, which are the box's.
  - **It links** the client's crates but the web shell and the bundle,
    jig's client crates, skein's, `temper-shell` (`components.md`), which
    every binary of temper's starts from, and the fetch of `temper-box`,
    `temper-fetch` and `temper-fetch-shell` (`../client/box-binary.md`,
    6); none of the engine's, the worker's, smith's agent, the box or the
    forge's setup, its row in 4 holding it with its exceptions. Of the forges' records it links the
    codecs alone, `temper-protocol-records`; their reading at start is
    `temper-protocol-records-forge`'s, which it never links
    (`components.md`).
  - **Its file** is the Cargo target `temper`, the name the site serves
    it by (`../draft/cli.md`, 5.3).
  - **Its story,** in the process tier: `temper ctl` signs in to a
    `temper-engine serve` on an engine alone's document, with the link
    it printed, and reads `me` (`../draft/deployment-binaries.md`,
    section 10).

## 2. Open questions

None.
