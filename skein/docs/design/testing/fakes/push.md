# The fake push service

Provisional, 2026-11-11. The kit's fake of a browser's push service
(`../../net/push.md`): it takes a Web Push request as Mozilla's, Google's
or Apple's push service would, checks what can be checked of it, records
it, and answers by a script. A world points its service's push
configuration at it, and a device's endpoint names it.

## 1. In one page

- **The domain** (`crates/skein-fake-push`, on skein-lib, skein-json and
  skein-tls) is a step crate: a request in, an answer and a record out
  (section 2).
- **It checks** the request's form, its VAPID token's signature and
  claims, and the sealed body's header; it cannot open the body, as
  ring keeps a private key for one agreement alone (`../../net/tls.md`,
  section 4), so a payload's words are its sender's to test
  (section 3).
- **The push peer** (in `crates/skein-fake-peers`, beside the forge's,
  the LLM's and OAuth's peers) hosts the domain on loopback or the
  simulator's sockets, in plaintext HTTP/1.1, and gives its records to
  the world (section 4).

## 2. The domain

- **`Service::new(origin, key)`:** the origin it answers as, `aud`'s
  value, such as `http://push.test:4000`, and the VAPID public key it
  expects, 65 bytes; a world gives it its service's key's public half.
- **`take(request, wall) -> Taken`:** of a `Request { method, target,
  headers, body }`, as skein-http's server read it. It gives the status
  and headers to answer with, and, for a push it accepted, its
  `Received` record.
- **Endpoints** are the service's own targets: any target under
  `/push/` is one browser's, its rest the subscription's name, as
  `http://push.test:4000/push/alice-phone`. A target outside it is
  `404`.
- **`Received { target, topic, ttl, urgency, length, at }`:** the
  target, the `Topic` or none, the `TTL`, the `Urgency` or `normal`, the
  body's length and the wall it came at. At most `RECEIVED`, 256, kept
  in order; past them each push is answered `507` and the domain says it
  is full, so a world fails before a record is lost.
- **A script** per target, set by the world at any time: `Accept`, the
  default, `201`; `Gone`, `410` for every push from then on; `Later {
  seconds }`, `429` with `Retry-After`; `Fail { status }`, that status.
  A push answered other than `201` leaves no record.

## 3. What it checks

Each failed check answers as a push service does, and records nothing:

- **The form:** `POST`, else `405`; a `TTL` of 0 to 2,419,200, an
  `Urgency`, when given, one of `push.md`'s four, a `Topic`, when given,
  1 to 32 bytes of the URL-safe base64 alphabet, and `Content-Encoding:
  aes128gcm`, else `400`.
- **The token:** an `Authorization: vapid t=TOKEN, k=KEY`, else `401`;
  `KEY` the service's key, else `403`; `TOKEN` three base64url parts,
  the header `{"typ":"JWT","alg":"ES256"}`, the signature checked by
  skein-tls's `verify_es256` over the first two, `aud` the service's
  origin, `exp` after the wall and at most 24 hours past it, `sub` a
  `mailto:` or `https:` URI, else `403`.
- **The body:** at least 86 bytes and the 17 a record needs; `rs` at
  least 18; `idlen` 65; the key after it a P-256 point's uncompressed
  form, its first byte 4; and no longer than 4,096 bytes past the
  header, else `400`; past 4,096 bytes in all, `413`.

## 4. The push peer

- **`push::Peer`,** in `skein-fake-peers`, as the forge peer is
  (`forge.md`, section 5): it listens where its owner says, serves
  skein-http's server over plaintext, reads each request's body whole
  within 8,192 bytes, hands it to the domain at the step's wall, and
  writes the answer. It implements the peers' `Listening`.
- **Its records** are the world's to read, `Peer::received()`, and its
  script the world's to set, `Peer::script(target, script)`.
- **Its faults** are the connection's: a reset before the answer, an
  answer dropped, drawn from the world's seed as the forge peer draws
  its own.

## 5. Testing

- **Step tests** in `skein-fake-push`: each check of section 3 refusing
  one fault at a time, with the status it answers; a request sealed by
  `skein-push` accepted and recorded; the script's answers; the bound of
  records.
- **The peer** in `tests/fake-peers`: one push taken over the
  simulator's sockets and recorded, and a scripted `Gone` answered
  `410`.

## 6. Open questions

- **A real browser's push service** is never reached by a test; a
  person's walk on a deployment does that. Default: the fake alone.
