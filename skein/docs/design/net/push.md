# Web Push

Provisional, 2026-11-11. The design of `skein-push`: a message a
service sends to a person's browser through the browser's push service,
sealed for the browser's subscription (RFC 8291) and signed with the
service's VAPID key (RFC 8292), posted to the subscription's endpoint
(RFC 8030). Step code: it makes the request's bytes and reads the
answer's status; connecting, TLS and the exchange are its owner's, over
`skein-link` and `skein-http` (`link.md`; `http/README.md`). Its
cryptography is skein-tls's (`tls.md`, section 4), so ring stays behind
one crate. Its fake is `../testing/fakes/push.md`.

## 1. In one page

- **A subscription** is what a browser's `PushManager.subscribe` gives:
  an endpoint, the push service's URL for this browser, and two keys,
  `p256dh` and `auth` (section 2).
- **The service's key** is a P-256 key pair, the VAPID key: its private
  half signs each request's token (section 3), its public half is the
  `applicationServerKey` a browser subscribes with, 65 bytes.
- **A push** is a payload of at most 3,993 bytes, with a time to live,
  an urgency and an optional topic. `seal` makes the request: a `POST`
  to the endpoint's target, its headers, and the payload sealed for the
  subscription (section 4).
- **An answer** is a status, classified: made, gone, too large, later,
  refused or failed (section 5).
- **Not deterministic,** as TLS is not: each seal draws a fresh key and
  salt, and each token's signature draws ring's randomness, from the
  kernel (`tls.md`, section 4). Their lengths are fixed, so a request's
  length is a function of its inputs.

## 2. The subscription

- **`Subscription { endpoint, p256dh, auth }`:** `endpoint`, 1 to
  `ENDPOINT_BYTES`, 1,024, bytes of an absolute URL; `p256dh`, 65 bytes,
  an uncompressed P-256 point (its first byte 4); `auth`, 16 bytes.
- **`Endpoint::parse(bytes) -> Option<Endpoint>`:** the scheme `https`
  or `http`, lower case; an authority, a host of 1 to 253 bytes of
  letters, digits, `-` and `.`, or an IPv4 address, and an optional port
  1 to 65,535; then a target, `/` and the rest, its bytes printable
  ASCII with no space; no user information and no fragment. None for
  anything else. `origin()` is the scheme, `://`, the host and, when
  given, `:` and the port, as written; `target()` the rest. An owner
  holds a list of the push services it sends to and matches an
  endpoint's origin against it exactly.
- **The keys are checked at the seal:** a `p256dh` ring refuses as a
  point refuses the seal (section 4).

## 3. The VAPID token

- **`Vapid`:** the service's key, read by `Vapid::from_pkcs8_pem(bytes)
  -> Option<Vapid>` from a PKCS#8 PEM document, `-----BEGIN PRIVATE
  KEY-----`, of a P-256 ECDSA key with its public half, as `openssl
  genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-256` writes it, at
  most `VAPID_PEM_BYTES`, 1,024; none for another document, another
  curve or a key ring refuses. `public()` gives its public half, 65
  bytes.
- **The token:** a JSON Web Token signed ES256, `base64url(header) "."
  base64url(claims) "." base64url(signature)`, base64url without
  padding: the header `{"typ":"JWT","alg":"ES256"}`; the claims
  `{"aud":ORIGIN,"exp":EXP,"sub":SUBJECT}` in that order, `aud` the
  endpoint's origin, `exp` the seal's wall in whole seconds plus 12
  hours, `sub` the subject; the signature skein-tls's `sign_es256` over
  the first two parts, 64 bytes, `r` then `s`.
- **The subject** says who to reach about the pushes: a `mailto:` or
  `https:` URI, 1 to `SUBJECT_BYTES`, 256, printable ASCII with no
  space and no `"` or `\`. Some push services refuse a request without
  one.
- **Its header:** `Authorization: vapid t=TOKEN, k=KEY`, `KEY` the
  public half in base64url without padding.

## 4. The request

`seal(vapid, subject, subscription, push, wall) -> Result<Sealed,
SealError>`, of a `Push { payload, ttl, urgency, topic }`:

- **`payload`,** 1 to `PAYLOAD_BYTES`, 3,993, bytes: what fits one
  record of 4,096 with the header and the tag;
- **`ttl`,** seconds the push service keeps it for an unreachable
  browser, 0 to 2,419,200 (28 days);
- **`urgency`,** `VeryLow`, `Low`, `Normal` or `High`, written
  `very-low`, `low`, `normal` and `high`;
- **`topic`,** none, or 1 to 32 bytes of the URL-safe base64 alphabet:
  a later push of the same topic replaces one the push service still
  holds undelivered.

It gives `Sealed { origin, target, headers, body }` for its owner to
send with skein-http's client: the method `POST`, the endpoint's target,
and these headers in this order: `TTL`, `Urgency`, `Topic` when given,
`Content-Encoding: aes128gcm`, `Content-Type:
application/octet-stream`, `Authorization` (section 3); the owner adds
`Host`, the origin's authority, and `Content-Length`, the body's
length. The body is skein-tls's `seal_aes128gcm` of the payload for the
subscription's keys (`tls.md`, section 4): RFC 8291's one record,
`salt` (16 bytes), `rs` 4,096 (4 bytes, big-endian), `idlen` 65, the
sender's fresh public key (65 bytes), then the payload and its
delimiter, 2, sealed with its 16-byte tag. So a body is 86 bytes, the
payload, 1 and 16.

`SealError`: `Payload` (empty or past its bound), `Topic`, `Ttl`,
`Subject`, `Keys` (ring refused the subscription's key), or
`Endpoint` (it does not parse).

## 5. The answer

`answer(status, retry_after) -> Answer`, of the response's status and
its `Retry-After` in whole seconds, when it gave one:

| Status | Answer | What its owner does |
|---|---|---|
| 2xx | `Made` | nothing more |
| 404, 410 | `Gone` | forgets the subscription: the browser no longer holds it |
| 413 | `TooLarge` | nothing: the payload's bound is the service's |
| 429, 5xx | `Later(retry_after)` | sends no more to that push service until then, if it waits at all |
| 400, 401, 403 | `Refused(status)` | reports it: the token or the request is wrong, the deployment's to repair |
| any other | `Failed(status)` | reports it |

A push is not idempotent and carries no key the push service
deduplicates by; an owner sends one at most once, and a request whose
answer was lost is not sent again.

## 6. Testing

- **Step tests** in `skein-push`: an endpoint's parse at its bounds and
  each refusal; the token's header and claims byte for byte at a fixed
  wall, its signature checked by `verify_es256`; each header and its
  order; each `SealError`; `answer` over each row of section 5; a
  sealed request taken by the fake push service's domain
  (`../testing/fakes/push.md`).
- **The seal's known answer** is skein-tls's (`tls.md`, section 5).
- **A fuzz target** for `Endpoint::parse` and `answer`: no panic, and
  an endpoint that parses gives back its bytes from its origin and
  target.

## 7. Open questions

- **Several records:** a payload past one record is refused; RFC 8188
  allows more, which no push service accepts. Default: one record.
- **Retrying a push:** not designed; a push whose answer was lost or
  came back later is not sent again. Default: none.
