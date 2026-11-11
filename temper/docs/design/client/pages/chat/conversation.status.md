# conversation: status

2026-11-11. Where the tree's code lags or departs from [conversation.md](conversation.md).

- **Not yet:** 2: the long-message and composer manual proof remains `../../view.status.md`, 4.

- **Not yet:** 2: the whole-bound manual reading check remains `../../web/testing.status.md`, 3; the failed-head retry remains `skein/docs/design/ui/web/README.status.md`, 7.

Section 2's *Opening history* has no known implementation gap in the
domain: `Chat::accept_activity` (`pages/chat.rs`) reads on from a chat's
first page only while its held lines weigh under `OPENING_WEIGHT`, 2,880
bytes, and the domain's `a_chat_opens_to_its_opening_or_a_screen_and_a_half`
holds a short chat reaching its opening, a page one byte under the weight
reading on, and one at it stopping with its older cursor, which `Older`
then reads. The browser's story 14 (`../../web/testing.md`, 6) checks it after
landing; its later failures are `../../web/testing.status.md`'s, 3 and 6.

- **Departs:** 2, *While a read is out*: the failed-head retry gap remains `skein/docs/design/ui/web/README.status.md`, 7.
