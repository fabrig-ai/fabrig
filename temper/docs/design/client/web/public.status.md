# public: status

2026-11-10. Where the tree's code lags or departs from [public.md](public.md).

- **Not yet:** 4: the browser kit returns the first content quad of each text
  node, so the public geometry story cannot check every wrapped fragment.
  Its result needs a browser testing interface decision before it can retain those
  fragments. Missing rectangles remain failures.
- **Not yet:** 4: the intermittent missing rectangle's cause remains
  unestablished. The story retains one terminal boxed query per page and
  width, checks its truncation count, and fails on missing geometry with
  the text and a preceding measurement-stage line. Batching narrows a
  changing page's query window without making it atomic; the request
  `ui-public-story`'s timer-driven explanation remains unproved (`README.md`, section 3).
- **Departs:** 4: `browser_public` accepts the 401 resource-load console
  text throughout the walk, including its public reads, because a stream
  around sign-out can correctly answer for an ended session. The request
  `ui-public-story` permits this expectation if the sign-out flake recurs;
  it is matched by text, without a stage or resource constraint, so an
  unexpected public-read 401 with those words is accepted too (`README.md`, section 3).
