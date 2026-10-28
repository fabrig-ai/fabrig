# Real git transcripts

Run with git 2.41 or later, tested with 2.53:

```
TEMPER_FORGEJO=<absolute path to the pinned Forgejo> git-transcripts --git /usr/bin/git --scratch target/git-transcripts --out tools/git-transcripts
```

The tool starts and removes its own isolated Forgejo, with Actions off.
`VERSION` in `skein_forge::forgejo::pin` pins it.
It records 42 invocations using protocol/git.md's settings and whole
environment; credentials are Basic headers in the environment, scanned
before writing.
Each case holds `argv` (one argument per line), `exit`, `stdout`, and
`stderr`. A single map masks object ids, abbreviated ids, loopback ports
and the scratch path across all cases. `facts.txt` names Forgejo's version
and digest, git's version, the renamed repository's recorded git redirect,
and the protected push's porcelain rejection.
Fixture setup invocations are separate from the recorded operations.
