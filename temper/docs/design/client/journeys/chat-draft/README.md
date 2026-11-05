# chat-draft

On a fresh `temper box --fake-backends`, built with
`temper/scripts/bundle.sh --loopback --record`: set up Forgejo on this
machine with the offered empty `work/hello`, sign in to the fake ChatGPT
plan, and create Hello. Wait for Home's status before starting the
first chat. Home writes note 1; the chat's document is note 2.

Reload `/projects/1/settings` to start the recording at How work runs.
Mark the page with Alt+Shift+M, turn on "Chats can write drafts", and
mark it again. Open "+ New chat", send "Let us design Away mode." to
start the first chat, T1, and mark its settled reply. Press "+ Draft";
the fake model writes the 29-line Away mode document and Split opens.
Mark it, choose Chat, and send "Add what happens to a running goal."
The second document turn answers "Edited", "+7 −0". Mark the settled
Chat, choose Split and mark it, then Draft and mark it.

Press "Start goal…", mark the one sheet with its default `away-mode`
name and Task by task way, then press "Start goal". The fake goal, T2,
finishes with its local report. Mark its page and save the recording
with Alt+Shift+S. The first replay proves all 20 prints against the
live pages.

The required return through the goal's Draft button is blocked: the
goal page offers no Draft button. The second document turn also leaves
the displayed body at its first revision: "A running goal" is absent
in Split and Draft despite the successful edit line. The goal-start
request carries that old body, with its draft and chat provenance.
These steps are recorded as observed; the return is not recorded.

- `01.html`: How work runs with drafts Off, marked.
- `02.html`: the same page before toggling "Chats can write drafts".
- `03.html`: drafts On, marked.
- `04.html`: the same page before opening "+ New chat".
- `05.html`: the new chat with its opening words entered, before Send.
- `06.html`: T1 after its opening fake reply, marked.
- `07.html`: the settled chat before pressing "+ Draft".
- `08.html`: Split after the first document write, marked.
- `09.html`: the same Split before choosing Chat.
- `10.html`: Chat with the revision request entered, before Send.
- `11.html`: Chat after the second document turn, marked.
- `12.html`: the same Chat before choosing Split.
- `13.html`: Split after the second document turn, marked.
- `14.html`: the same Split before choosing Draft.
- `15.html`: Draft after the second document turn, marked.
- `16.html`: the same Draft before pressing "Start goal…".
- `17.html`: the Start goal sheet, marked.
- `18.html`: the same sheet before pressing "Start goal".
- `19.html`: the settled goal page, marked; no Draft button appears.
- `20.html`: the same goal page at the recording's end.

The walk used a throwaway Chromium CDP driver, accepting JavaScript
dialogs and using `Emulation.setDeviceMetricsOverride` for 420-pixel
renders. No driver is kept. Screenshots are outside the checkout.
