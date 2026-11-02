# sign-in-wait

agents/creds, 11-02, at the director's word.

On 11-02 ~10:10Z the user's first `smith login chatgpt` for the live suite's root failed. Signing
in and approving in the browser took more than 2 minutes. smith's sign-in deadline is 120 s
(`sign_in_time: Duration::from_secs(120)` in
`smith/crates/smith-local-shell/src/credentials/product.rs`). So smith printed `account chatgpt:
not signed in`, exited and closed its loopback listener, and the browser's redirect through
`ssh -L` ended in ERR_CONNECTION_RESET. Nothing told the user there was a 2-minute limit. The
retry worked once the user signed in to ChatGPT first.

Wanted:
- the sign-in waits long enough for a person who must sign in, with two-factor authentication,
  on another machine: 10 minutes;
- `smith login` says how long it waits, beside the `Sign in:` address;
- when the wait runs out, the failure says it was the time, not a bare `not signed in`.
