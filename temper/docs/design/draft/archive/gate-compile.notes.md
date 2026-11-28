# gate-compile: the reasons behind it

This holds the reasons for `gate-compile.md`, moved from the shared notes file.

The reasons behind `gate-compile.md`, from the user, 2026-11-13.

- **What they asked for:** after `crate-graph.md`'s milestones landed, "you can take measurements yourself, just look at what landed in the last 8 hours via the CI landing job ... and measure build times (or triangulate them if they are mixed with test times)"; then "come up with a follow up draft for improving the situation against your observation", the shared compiler cache's operational problems aside. Then "okay, I like it, let's go for all the changes you propose except the weekly ranking (for now)" (`gate-compile.md`, 6 and 11).
- **The draft's readings,** not the user's words: the gate's build step as the measure, since it builds every test target; units no milestone touched as a rough yardstick for the ones the milestones split, since the toolchain and the builder's load changed too; a split judged by everything that replaced the old units, not by its largest piece; compile per build as what's left to cut; generated tests as integration tests, an exception to step tests; one version's generated code as skein's codec design already says; the first split measured before the next; and a weekly ranking of the next cuts, proposed and left for later.
