# design-upkeep: status

2026-11-23. Where the design lags or departs from [design-upkeep.md](design-upkeep.md).

- **Design departs:**
  - 3.2: a status file with nothing after its opening reads as done, as the close command leaves one when it settles the file's last entry (`tools/close/README.md`), where the draft names only "No gap.". Decided by U2's designer, 2026-11-21, so that each goal's close doesn't leave files the roll-up calls unread.
  - 3.2: a design document's status may also hold **Design departs** entries, where a goal with no draft records how its design departs from a request or a mockup, as fabrig's goal charter asks. The roll-up reads them under that label (`tools/citations/README.md`), where 3.2 gives a design document's labels as **Not yet** and **Departs**. Decided by the drafter, answering U2's designer (question 218192), 2026-11-21, so that no departure's reason is dropped and no text is pushed into a document over 48 KB before the split pass.
  - 4.2: a piece of a document that isn't its part's `README.md` goes beside it, as `<name>-<concept>.md`, the way `domain/landing-settings.md` and `protocol/git-sweep.md` went beside theirs, rather than into a directory named for the part with the document as its `README.md` (`DESIGN.md`, 5). A piece of a part's `README.md` goes in the part's directory. So the document left behind keeps its path as well as its numbers, and a goal in flight that cites or edits it still finds it (4.3). `draft/notes.md` is left behind the same way: each of its sections goes beside its draft, its heading staying with the line that names where, and it takes no reasons (`DESIGN.md`, 4) until a later pass removes it. Decided by U3's designer, 2026-11-22.
- **Open:** none.
