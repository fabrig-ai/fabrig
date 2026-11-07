# hearth-apt-304: retirement

2026-11-07. The recordings part of `hearth/docs/plan/hearth-apt-304/`
is landed.

The seven mmdebstrap 1.5.7 / apt 3.0.4 cases contain all 50 supplied
capture files, byte for byte. Their seven expectations equal apt 3.0.3's
byte for byte. The plan leaves apt 3.0.3's recordings and expectations
unchanged. Between the two capture versions, only five stderr files differ,
as the request describes.

No design or status file needs changing: the plan's decision preserves
`hearth/docs/design/domain/testing.md`, section 4.5, whose versioned
recording contract already covers both apt versions. Its status records
M2's missing tool fakes and captures, not a gap in these resolution
recordings. Those M2 gaps and the separate gap in automatic conformance
runs remain outside this plan. Section 11's open questions remain current;
this plan leaves no deferred work to add there.

The supplied change to `temper/docs/design/STATUS.md` indexes a TUI draft
and is unrelated to this plan. It needs no retirement edit here. No design
contradiction was found in the recording contract.

The checks above are file comparisons, not decoder test runs.
