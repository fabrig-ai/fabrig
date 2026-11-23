# Admission refusals

Provisional, 2026-11-19. Why `Client::prepare` refuses a request before
anything is sent, and how each refusal names its cause (`vocabulary.md`,
section 1). A refusal starts no call, emits no terminal and supplies no
retry policy, as before; what changes is that it says which rule the
request broke, so its owner can say so too, rather than one line for
every cause.

## 1. In one page

- **Admission names its cause.** The error a refused request gets is

  ```rust
  pub enum Error {
      Invalid { cause: Invalid },          // the request breaks a rule of its dialect: which one
      Limit { which: Cap, bound: u64 },    // a configured local cap fired: which one, and its value
      Unsupported { cause: Unsupported },  // its dialect cannot carry what the request asks: what
  }
  ```

  `Invalid` and `Unsupported` are closed enums of causes, section 2. The
  classes stay as they were: what was refused `Invalid` is still, and
  what was `Unsupported` is still. A caller that reads only the class
  matches `Error::Invalid { .. }`, as the client's own translation of a
  completion does (a translation failure stays `Protocol`) and the fake
  peer's decoding (a malformed request stays `Malformed`).
- **Each cause has a name,** `Invalid::name` and `Unsupported::name`,
  `pub const fn name(self) -> &'static [u8]`: lowercase ASCII letters
  and hyphens, at most `CAUSE_BYTES` (24), and no name given twice
  across the two enums. An owner that reports a refusal reports it by
  this name (smith's `protocol/llm.md`, section 6). The name is fixed
  bytes of this crate's, never a provider's.
- **No catch-all.** Every refusal site gives the cause whose rule it
  checks; a site whose rule none names takes a new cause, named as these
  are. Sites that check one rule in several places give the same cause.
- **Documents too.** `document_error` keeps receiving overflow as
  `Error::Limit`, and gives malformed grammar, a missing field or a
  wrong type `Invalid { cause: Document }` (`vocabulary.md`, section 1).
- **Limits are unchanged:** `Limit { which, bound }` names its cap
  already (`vocabulary.md`, 1.5).

## 2. The causes

`Invalid`, a request malformed for its dialect:

| Cause | Name | Refused when |
| --- | --- | --- |
| `Model` | `model` | the model's name is empty |
| `NoMessages` | `no-messages` | an Anthropic request holds no message |
| `EmptyMessage` | `empty-message` | an Anthropic message holds no block |
| `Output` | `output` | the output ceiling is zero |
| `ToolSchema` | `tool-schema` | a tool's schema is not a JSON object |
| `Identifier` | `identifier` | a tool's name or a call's id is empty, too long for its dialect, or holds a byte its dialect refuses |
| `Role` | `role` | a block is in a role that cannot hold it: a call, a refusal or a provider's replay in a user message, a tool's result in an assistant's |
| `Outcome` | `outcome` | an oversize or cut call is replayed as a call (`vocabulary.md`, 1.4) |
| `Choice` | `choice` | `Only` names no tool, one not offered, or one twice (`vocabulary.md`, 1.2) |
| `Text` | `text` | text or a number JSON cannot spell, invalid UTF-8 among them |
| `Arguments` | `arguments` | a call's arguments are not a JSON object |
| `Replay` | `replay` | a provider's replay is malformed: not an object, a field or a value it cannot hold, cut short, or with bytes after it |
| `Reasoning` | `reasoning` | a reasoning item is malformed: no type or another block's, an empty signature or empty data |
| `Credential` | `credential` | the credential is empty, not a header's value, or not its provider's scheme |
| `Endpoint` | `endpoint` | the endpoint's authority or target is malformed |
| `Header` | `header` | an extra header is reserved, malformed or repeated |
| `Head` | `head` | the request's head is past what its length arithmetic counts |
| `Limits` | `limits` | the client's limits are inconsistent (`wire.md`, 1.3) |
| `Document` | `document` | a document is malformed, misses a field, or holds one of the wrong type |

`Unsupported`, what a dialect cannot carry:

| Cause | Name | Refused when |
| --- | --- | --- |
| `ReplayProvider` | `replay-provider` | a provider's replay or reasoning item belongs to another provider |
| `ReplayBlock` | `replay-block` | replay rides on a block its dialect carries none on |
| `ReplayForm` | `replay-form` | a replay's version or provider tag is unknown |
| `OutputCap` | `output-cap` | the request asks an output cap of a route that takes none |
| `Effort` | `effort` | a reasoning effort or thinking setting its dialect does not take |

An empty message is the case that matters most: Anthropic refuses one
on every attempt, so a history holding one could never be sent. smith
never builds one (smith's `protocol/llm.md`, section 2, *No empty
turn*); a refusal naming `empty-message` says another owner did.

## 3. Verification

The focused suites of `skein-llm` and `skein-llm-world` hold each cause
where a request can reach it: one case each, its error compared whole,
cause included. An assertion that compared `Error::Invalid` or
`Error::Unsupported` compares the cause its site gives. A test checks
that every name is lowercase ASCII letters and hyphens, at most
`CAUSE_BYTES`, and given once.

## 4. Open questions

None.
