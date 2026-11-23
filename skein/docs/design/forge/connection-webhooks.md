# The forge component: the webhooks' listener

## 1. The webhooks' listener

Deliveries cut polling's latency and decide nothing (README.md, section
6). Polling stands with no webhook (client.md, section 4) and
finds whatever a delivery lost, refused or never sent told of.

### 1.1 The listener, and each forge's path

- **The listener** is a socket of the component's own, on loopback:
  plaintext HTTP on skein-http's server, at the owner's address. A forge
  on another host polls alone until skein-tls has a server side (11).
- **One path per forge,** which the dialect gives (Forgejo's is
  `POST /hooks/<number>`, README.md, section 6), and which setup's hook
  names (setup/forgejo.md, section 1.5). A connection takes one request at
  a time and is kept alive.
- **A forge's secret** comes only with the listener. A forge with none
  has no path: it is answered as a path that names no forge.
- **A GitHub forge has no path:** its deliveries are read from the App's
  log (1.5), whose reads TLS and the JWT vouch for, so its access holds
  no secret.

### 1.2 Verified as it streams, answered first

The component computes the HMAC-SHA256 as the body arrives, with
`sha256::Hmac` keyed by the forge's secret, and decodes the body at the
same time into the few fields the hint reads. Nothing of it is believed
until the whole body is read and the signature holds, compared with
`sha256::equal`, in constant time (../runtime/lib-bytes.md, 1.2). The header, the events
and the fields are the dialect's (README.md, section 6).

Once the body is read and verified, the `204` is queued, and only then
does the hint go up, so the owner never makes the forge wait on its work
(http.md, 5.4). A verified delivery that hints nothing, one without the
dialect's event header (`X-Forgejo-Event`) among them, is answered `204`
too. A hint names the forge's number and the payload's repository id;
one for a repository the owner has not adopted goes up all the same.

### 1.3 What it answers

The head is checked in this order. A refusal is answered, its body
discarded, and the connection closed.

| The request | Answer |
|---|---|
| a head the server itself refuses | skein-http's own answer: a `4xx`, or `501` or `505` |
| a path that names no forge, or a forge with no secret | `404` |
| a method other than `POST` on a forge's path | `405` |
| no signature, or one that is not 64 lowercase hexadecimal digits | `403`, body unread |

A body is read within `hook_body_bytes`. A body too long (`413`), a
wrong signature (`403`) and a verified body the decoder finds malformed
(`400`) are answered as README.md, section 6, says; each closes the
connection and hints nothing. A `204` alone keeps it alive. A request
not read within `hook_deadline` closes with no answer.

### 1.4 Bounds, and a listener that fails

Connections (`hooks`), the head (`hook_head_bytes`), the body
(`hook_body_bytes`) and a request's time (`hook_deadline`) are capped; a
full listener accepts no more until one closes. A push payload's commit
list is skipped by the decoder, not held. The listener's bound address
goes up once as `listening`, so an owner that gave port 0 learns its
port. A listener that fails while the component runs is said once, as
`listener lost`, and it and its connections close. What the owner does
is its own.

### 1.5 A GitHub App's delivery log

A GitHub App's hook points at no listener (setup/github.md, section 1.5):
GitHub keeps each delivery it could not make in the App's log, and the
component reads the log instead. The reads are github.md's plans
(`github.md`, section 9.3), on the App's JWT and the forge's own connection, counted
in no call's cost, one at a time.

- **Each `log_poll`,** while the forge is ready, the puller reads the
  log's first page, newest first, conditionally: with the validator the
  last first page answered, kept for the forge (at most
  `VALIDATOR_BYTES`, README.md). A `304` says nothing is new, and the
  poll ends there.
- **It reads back to the newest it kept.** The puller keeps the guid of
  the newest delivery it read. A poll reads rows, and pages by the log's
  cursor, until it meets that guid, or a row delivered more than
  `log_catch_up` ago, or `log_pages` pages, and then keeps the first
  row's guid as the newest. A redelivery repeats its delivery's guid:
  the puller skips it, and never takes it as the newest.
- **With none kept,** at the component's first start of the forge, the
  first poll keeps the newest guid and hints nothing older: the client's
  reads afresh at its restart cover what came before (client.md,
  section 8). A later start of the forge, after a connect or a stop,
  reads back to the guid kept, within `log_catch_up` and `log_pages`.
- **A gap,** a poll that ends on `log_catch_up` or `log_pages` before it
  meets the guid kept, is said once as `Fact::Gap`, and the
  installations are read again (`connection-calls.md`, 1.4). The client's polling finds the
  rest (section 1).
- **A row's event decides what it costs:**
  - one of an installation's (github-app.md, section 1.3) reads the
    installations again, once for the poll (`connection-calls.md`, 1.4), and one saying an
    installation's repositories changed sends that installation's
    `Repositories` up;
  - one the dialect hints is read whole, `GET
    /app/hook/deliveries/<id>`, within `delivery_bytes`, and decoded as
    it streams into its hint, as a webhook's is (1.2). Its repository's
    path renames or drops that repository's row (`connection-calls.md`, 1.2). One past `delivery_bytes` hints
    nothing, and the client's polling finds it;
  - any other is passed over, unread.
- **A hint goes up** as 1.2's do: one held for the forge while the owner
  has no room, and the puller reads nothing more until it is taken.
- **What a poll is refused:** a `401` stops the forge, `KeyRefused`
  (`connection.md`, 3.3). Anything else ends the poll, which the next `log_poll` asks
  again; the guid kept stays.
