# Request: a connected Forgejo, designed first

From status-gaps (its item 3, question `forgejo-connect`), answered unattended on 11-03 by
agents/loop at the director's word (the user asleep; the recommended option taken). The design is
`temper/docs/design/protocol/forge.md` 2.6; the gaps are `temper/docs/design/domain/forge.status.md`
2.2 and 2.6 and `temper/docs/design/setup.status.md` 4.

## The question, as asked

The connected Forgejo (temper forge.status.md 2.2 and 2.6, setup.status.md 4) cannot be built from the design as it stands; it leaves open:
1. The wire: a new connected-mode Forgejo needs a ForgeAsk arm (client-wire-v1.schema has github, owned and managed), and How::ForgejoTokens and Request::Replace are named only with values that travel carried, not as records.
2. The client: client/forge.md and client/tools.md mark the tokens' Connect "not offered" while offering Replace on a connected-mode Forgejo, which nothing then makes.
3. Where its login is kept: protocol/forge.md 2.x says the records hold its id and login (the User row), but User holds the id alone.
4. Connect again on a stopped connected Forgejo by another user: skein's Check refuses OtherUser; credentials.md 2.1 says connecting again overwrites.
5. The paste: credentials.md 2.2's bundle layout of several items against the code's one-item Pasted (temper-secrets 1).
Options: (a) recommended: temper's designer settles these first, then a later cut builds it (about 7 tasks: records, wire, engine domain, the service's Check meeting point, Replace, ctl, the web view). Suggested settlement: ForgeAsk gains connected: ManagedForge; How gains forgejo_tokens: ForgejoTokens {} and the requests replace: Replace { forge: u16 }, the values carried; Pasted holds a list of items in the codec's own form, credentials.md 2.2 saying so; the login kept in the forge record's bot; another user refused OtherUser; offered in ctl and on the web page; all in place at CLIENT_VERSION 11. (b) build only the engine and service side with (a)'s shapes, reached by tests alone, the client staying "not offered". (c) withdraw the connected Forgejo from the design.


## The answer: (a)

temper's designer settles the five open points first, in temper's designs (protocol/forge.md,
client/forge.md, client/tools.md, credentials.md 2.1 and 2.2, setup.md), taking the suggested
settlement as its start and departing only with a reason written in the design:

- the wire: ForgeAsk gains `connected: ManagedForge`; How gains `forgejo_tokens: ForgejoTokens {}`
  and the requests `replace: Replace { forge: u16 }`, the values carried;
- the paste: Pasted holds a list of items in the codec's own form, credentials.md 2.2 saying so;
- the login: kept in the forge record's bot;
- Connect again by another user on a stopped connected Forgejo: refused `OtherUser`;
- the client: Connect offered in ctl and on the web page, Replace with it;
- all in place at `CLIENT_VERSION` 11 (the versions stay at 11 until the first release).

Then the build, about seven tasks: records, wire, engine domain, the service's Check meeting point,
Replace, ctl, the web view. No stored format moves beyond what is in place at 11.
