# journeys: status

2026-11-06. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Departs:** 3.6: the API-key field carries a pasted trailing newline rather
  than trimming it. Redaction preserves that newline and replaces both the
  whole-value and trimmed-value digests, matching what the host carries so the
  replay holds.
