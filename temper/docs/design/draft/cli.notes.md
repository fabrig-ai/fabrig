# cli: the reasons behind it

This holds the reasons for `cli.md`, moved from the shared notes file.

The reasons behind `cli.md`, from the user, 2026-11-08.

- **What they asked for:** "a fresh look to fabrig's CLI interfaces, in terms of environment variables, command line arguments and build flags". Every convention is "repo-wide", documented under `skein/docs/foundation/` "and enforced by code wherever possible (ideally everywhere)".
- **No features:** "We should have no feature flags, "cargo build" from a fabrig repo checkout builds everything." Asked whether that grows the binaries ("ballpark, no exact measures needed"), the draft's answer is not significantly: a feature that becomes a crate leaves a binary linking what it did, and the recorder and the browser's switches, built always, are small and measured as they go.
- **Two sets:** "user facing (public, stable) and internal (hidden, unstable)", the public ones "advertized via the cli (<binary> {-h,--help,help}) in standardize way", and all of them documented "with standardized format and location (proposed: <tree>/docs/design/cli/<binary>-{env,args}.md) and always be in sync with actual code", and "Env vars common across several binaries go into <tree>/docs/design/cli/common-env.md".
- **Layout:** "All binaries are built from <tree>/bin/<binary>/main.rs", "main.rs is just a few lines of plumbing code (you tell me a reasonable size, I think 100 lines at most)", and "all actual logic lives under <tree>/crates".
- **The public set's quality** can't be checked by code: "elegant, orthogonal and ergonomical".
- **Scope:** "let's limit the scope to temper binaries for this round, other trees can wait", and "this should [be] in the v0.1.0 roadmap".
- **What's internal:** "from a quick look all ctl should be internal (for now), all fakes, and probably "temper forge", not sure if something else."
- **The draft's first look, agreed whole** ("everything you said sounds good"): a marker in internal variables' names, a third set for what tests alone read, `main.rs` at 30 lines, one table per binary rendering the reference documents.
- **The names:** the current single binary "temper" should become "temper-box", and `temper` is the TUI. On delivering the box, the user weighed serving `temper-box` under the name `temper` against `temper box` downloading `temper-box` at its first run, and chose the second: "Let's go for "temper" as TUI only and temper-box downloaded when not found around (~/.cache/temper)."
- **What `temper box` does:** "start the TUI normally, as a regular "temper" run, but also print some message saying it's downloading temper-box and then start temper-box as sub-process" (the user). The draft keeps today's behaviour around it: the box's log in its file, and quitting the TUI stopping the box.
- **The download's time,** the draft's estimate for the box's 24 MB: about 1 to 3 seconds on fiber, cable or a cloud machine, which most of temper's audience has; 12 to 24 seconds on ADSL; 40 seconds to a minute and a half on a weak mobile hotspot. The download line pipes through `gunzip`, since curl has no short form of `--compressed`.
- **The draft's readings,** not the user's words: `ctl` stays in `temper`, internal; `temper box` doesn't look on PATH, where another commit's `temper-box` may be; `serve` and `program-login` are public on `temper-box`, for one person's deployment on one machine; `TEMPER_INTERNAL_BUNDLE=none` stays for development builds' speed.
