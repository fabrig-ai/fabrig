# live-smith: the API-key story is the owner's, out of the automatic live run (the user, 11-02)

smith's live pair (`live-smith` in the root `.config/nextest.toml`) runs after every landing with
logins lent from the shelf. Its story `smith::live an_anthropic_key_delivers_a_commit_in_place`
needs an Anthropic API key, which no shelf holds, so it is red on every run (at main da9027dbf5ae:
0.010 s, `live-smith-4.log` of that after-landing run).

The user's decision: leave it manual for now. Keep it out of the automatic live run, as temper's
person-needing live stories are: mark it `#[ignore]` with a reason that says it needs an API key and
is run by the owner, or leave it out of the `live-smith` profile's default filter; the owner's way to
run it stays (say how in smith's testing design, where the live tier is described). No other live
test changes. A test: the profile's selection without it (a nextest `--list` count), and the
design text that says what runs after landing and what stays the owner's.

Out of scope: the other live-smith reds (another plan), fab's after-landing code, the shelf.
