# skein: test environment

Generated from the tree's test table (`skein/docs/design/runtime/shell-table.md`, 1.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Test

### `SKEIN_TEST_FUZZY_CAMPAIGN`

- **Set:** test
- **Reads**
- **Form:** SEED
- **Absent:** With neither fuzzy variable set, each sweep runs its gate seeds.
- Run each adopted sweep's fresh cases for this campaign seed.
- **Notes:** A campaign sets it. Use sixteen hex digits with no prefix. A malformed value or both variables set fails the test before its first case. tools/tiers/one-process removes it.

### `SKEIN_TEST_FUZZY_CASE`

- **Set:** test
- **Reads**
- **Form:** SEED
- **Absent:** With neither fuzzy variable set, each sweep runs its gate seeds.
- Run one case with this seed, in the campaign's configuration.
- **Notes:** A campaign's replay sets it. Use sixteen hex digits with no prefix. A malformed value or both variables set fails the test before its first case. tools/tiers/one-process removes it.
