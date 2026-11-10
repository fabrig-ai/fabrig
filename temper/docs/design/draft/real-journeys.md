# Journeys on real models

Draft, 2026-11-05, from a session with the user that day. Journeys are
recorded on fake backends alone (`../client/web/journeys.md`, 3.5), because
a recording keeps everything the page was given. The user: "we need to
fix this, because a few things are hard to show without real models.
We can do this: email gets redacted (I don't think it matters), for
repository names I'll use public ones anyway." This draft lets a person
record a journey on a box with real accounts. `add` redacts what must
not be committed before anything is printed. The user asked for it to
be built (2026-11-05).

## 1. In one page

- **A real box takes the recording bundle:** `temper box --bundle
  BUNDLE`, as `--fake-backends` takes one today (section 2).
- **`add` redacts, then proves** (section 3). Before the first replay's
  proof (`../client/web/journeys.md`, 4.3) it replaces every secret the
  recording holds, every person's email address, and each text the
  person names. Each original gets one placeholder of the same length,
  everywhere it appears, so that the replay still holds.
- **A last scan refuses** a redacted recording that still holds a
  secret's shape (section 4). Nothing is written then.
- **A real journey is recorded by a person:** `again` walks fake boxes
  alone. A stale real journey is recorded again by hand, and the
  journeys' table (`../client/web/journeys.md`, 9) says which are real
  (section 5).

## 2. The real box's bundle

- **`temper box --bundle BUNDLE`** on real backends serves BUNDLE in place
  of the binary's own, as `--fake-backends --bundle` does: the recording
  bundle (`scripts/bundle.sh --loopback --record`) for a person who
  records.
- **Nothing else changes:** the box, its accounts and its forge are real.
  The terminal says once, under the sign-in block, that the page records
  nothing until Alt+Shift+S saves it.

## 3. Redaction

`add NAME FILE` redacts before it proves. It reads the recording as its
lines (`../client/web/journeys.md`, 3.2), and within each one every quoted
field. A `heard piece` is a stream's `data:` line, so its payload is
redacted decoded and encoded back.

- **What it replaces:**
  - *the recording's secrets:* the `secret` record's value, each
    `stored` record's value, and each value of an act whose event shows
    a secret's length (`Secret { len: N }`), such as a typed key or a
    pasted sign-in;
  - *email addresses:* every address but those at `example.com`,
    `example.org`, `example.net` and `localhost`;
  - *the person's own:* each `--redact TEXT`, given as often as
    wanted, for a name, a host or anything else.
- **How:** each original gets one placeholder, the same bytes wherever
  it appears and of its length, so that every frame, length prefix and
  base64 run keeps its size. An address becomes one at `example.com`,
  `x`s padding its local part, or a shorter domain when it is too short
  for that. Any other text becomes `x`s.
- **Then the proof:** the first replay runs on the redacted recording,
  against its redacted marks. The host derives what it asks from what it
  heard, so a consistent replacement replays as the original did. A mark
  that differs is refused as today, and is a fault in the redaction.
- **What `add` prints:** the count of each kind it replaced, never an
  original.

## 4. The last scan

Before writing, `add` scans the redacted recording, its decoded pieces
included, for what must never be committed:

- any original it replaced;
- an email address outside the example domains;
- a key's or token's shape: `sk-` and an OpenAI key's run, a JWT's
  three base64url parts beginning `eyJ`, a `Bearer ` value, an OAuth
  `code=` or `refresh_token` value.

A match refuses the recording, naming the record and its kind, never
the bytes. The person adds a `--redact` and runs `add` again.

## 5. Real journeys

- **Recorded by hand,** on a fresh box with real accounts. The person
  uses public repositories and writes nothing private into the chats:
  the model's words are kept as they were given.
- **Not walked by `again`** (`../client/web/journeys.md`, 8.1), which starts
  fake boxes alone. A stale real journey is recorded again by its person.
- **Named as real** in the journeys' table (`../client/web/journeys.md`,
  9), beside the fake journeys.
- **Section 3.5's rule** becomes: fake backends, or real ones redacted
  by `add`.

## 6. Open questions

- **The stream's stored credential:** redacted with the rest. It is a
  loopback session of a box the person stops; whether `add` also asks
  the person to stop that box.
- **Usage and spend figures** the page was given stay as they were.
