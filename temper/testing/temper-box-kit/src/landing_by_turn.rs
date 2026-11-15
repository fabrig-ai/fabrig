//! The dispatch-only gate and funding share for owned Forgejo stories.
pub const WORKFLOW: &str = r#"on:
  workflow_dispatch:
    inputs:
      temper_commit: { required: true }
      temper_key: { required: true }
jobs:
  test:
    runs-on: temper-host
    steps:
      - run: |
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 -w0)
          git -c "http.extraHeader=Authorization: basic $auth" clone "${{ github.server_url }}/${{ github.repository }}.git" .
          git checkout "${{ inputs.temper_commit }}" && sh check.sh
"#;

/// Leave 7K of the stock 48K period available for landing tasks.
pub const ROLE_PERIOD_IN_K: u64 = 41;
