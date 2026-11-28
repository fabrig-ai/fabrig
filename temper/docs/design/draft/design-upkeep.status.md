# design-upkeep: status

2026-11-28. Where the design lags or departs from [design-upkeep.md](design-upkeep.md).

- **Design lags:**
  - 3.2: `tools/citations/README.md` and `status.py` still list a generated page among the long openings.
  - 4.2: `move.py` drops a moved literal's `allow.txt` exception, and the checker takes a citation to a section that holds only a "where it went" line.
- **Design departs:**
  - 3.2: a design document's status may also hold **Design departs** entries, where a goal with no draft records how its design departs from a request or a mockup, as fabrig's goal charter asks. The roll-up reads them under that label (`tools/citations/README.md`), where 3.2 gives a design document's labels as **Not yet** and **Departs**. Decided by the drafter, answering U2's designer (question 218192), 2026-11-21, so that no departure's reason is dropped and no text is pushed into a document over 48 KB before the split pass.
  - 4.2: a piece of a document that isn't its part's `README.md` goes beside it, as `<name>-<concept>.md`, the way `domain/landing-settings.md` and `protocol/git-sweep.md` went beside theirs, rather than into a directory named for the part with the document as its `README.md` (`DESIGN.md`, 5). A piece of a part's `README.md` goes in the part's directory. So the document left behind keeps its path as well as its numbers, and a goal in flight that cites or edits it still finds it (4.3). Decided by U3's designer, 2026-11-22.
  - 4.1 and 4.2: the CLI pages generated from their tables (`*/docs/design/cli/*-args.md` and `*-env.md`) keep their openings unnumbered, and the roll-up lists those past 8 KB. Each page's text is rendered byte for byte from its command's table by `skein-shell`'s reference renderer and held there by its sync test, so no goal adds an item to it by hand, and `move.py` can't number it without that test failing. Numbering them would be the renderer's change, for every tree's pages, which this rule doesn't need. Decided by U6's designer, 2026-11-24.
- **Open:** 7.1: when the split passes are automated, and with what cadence and approval, or what would keep documents in shape without them (2.6). The user decides.
