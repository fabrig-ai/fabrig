//! temper's style tokens and their names (client/browser.md, 2.5), as jig's `jig-client-shell-web` `look.rs`.
use crate::frame::owner;
use skein_ui::{Style, look::Names};
/// The goal column (client/browser.md, 2.5).
pub const GOAL: Style = Style { owner: owner(), token: 377 };
/// The goal heading (client/browser.md, 2.5).
pub const GOAL_HEAD: Style = Style { owner: owner(), token: 378 };
/// The goal metadata (client/browser.md, 2.5).
pub const GOAL_META: Style = Style { owner: owner(), token: 379 };
/// The goal byline (client/browser.md, 2.5).
pub const GOAL_BYLINE: Style = Style { owner: owner(), token: 380 };
/// The goal actions (client/browser.md, 2.5).
pub const GOAL_ACTS: Style = Style { owner: owner(), token: 381 };
/// A goal button (client/browser.md, 2.5).
pub const GOAL_BTN: Style = Style { owner: owner(), token: 382 };
/// A small goal button (client/browser.md, 2.5).
pub const GOAL_SMALL: Style = Style { owner: owner(), token: 383 };
/// A quiet goal button (client/browser.md, 2.5).
pub const GOAL_QUIET: Style = Style { owner: owner(), token: 384 };
/// A primary goal button or its recommendation (client/browser.md, 2.5).
pub const GOAL_PRIMARY: Style = Style { owner: owner(), token: 385 };
/// The goal needs band (client/browser.md, 2.5).
pub const GOAL_BAND: Style = Style { owner: owner(), token: 386 };
/// The goal needs band tag (client/browser.md, 2.5).
pub const GOAL_BAND_TAG: Style = Style { owner: owner(), token: 387 };
/// The goal needs choices (client/browser.md, 2.5).
pub const GOAL_CHOICES: Style = Style { owner: owner(), token: 388 };
/// The goal report and side columns (client/browser.md, 2.5).
pub const GOAL_TWO: Style = Style { owner: owner(), token: 389 };
/// The goal report (client/browser.md, 2.5).
pub const GOAL_REPORT: Style = Style { owner: owner(), token: 390 };
/// A goal section title (client/browser.md, 2.5).
pub const GOAL_TITLE: Style = Style { owner: owner(), token: 391 };
/// The goal typing indicator (client/browser.md, 2.5).
pub const GOAL_TYPING: Style = Style { owner: owner(), token: 392 };
/// The goal side column (client/browser.md, 2.5).
pub const GOAL_SIDE: Style = Style { owner: owner(), token: 393 };
/// A goal side section (client/browser.md, 2.5).
pub const GOAL_SECTION: Style = Style { owner: owner(), token: 394 };
/// The goal facts (client/browser.md, 2.5).
pub const GOAL_FACTS: Style = Style { owner: owner(), token: 395 };
/// The goal task progress bar (client/browser.md, 2.5).
pub const GOAL_BAR: Style = Style { owner: owner(), token: 396 };
/// A landed goal task segment (client/browser.md, 2.5).
pub const GOAL_BAR_LANDED: Style = Style { owner: owner(), token: 397 };
/// A running goal task segment (client/browser.md, 2.5).
pub const GOAL_BAR_RUNNING: Style = Style { owner: owner(), token: 398 };
/// A goal task line (client/browser.md, 2.5).
pub const GOAL_LINE: Style = Style { owner: owner(), token: 399 };
/// A running goal task line (client/browser.md, 2.5).
pub const GOAL_LINE_RUNNING: Style = Style { owner: owner(), token: 400 };
/// A goal task needing the person (client/browser.md, 2.5).
pub const GOAL_LINE_NEEDS: Style = Style { owner: owner(), token: 401 };
/// The landed goal tasks fold (client/browser.md, 2.5).
pub const GOAL_LANDED_FOLD: Style = Style { owner: owner(), token: 402 };
/// The goal messages (client/browser.md, 2.5).
pub const GOAL_SENT: Style = Style { owner: owner(), token: 403 };
/// A message from the person (client/browser.md, 2.5).
pub const GOAL_YOU: Style = Style { owner: owner(), token: 404 };
/// The goal composer box (client/browser.md, 2.5).
pub const GOAL_COMPOSER_BOX: Style = Style { owner: owner(), token: 405 };
/// The goal composer (client/browser.md, 2.5).
pub const GOAL_COMPOSER: Style = Style { owner: owner(), token: 406 };
/// The goal composer hint (client/browser.md, 2.5).
pub const GOAL_HINT: Style = Style { owner: owner(), token: 407 };
/// A goal sheet field (client/browser.md, 2.5).
pub const GOAL_FIELD: Style = Style { owner: owner(), token: 408 };
/// A goal sheet text input (client/browser.md, 2.5).
pub const GOAL_TEXT: Style = Style { owner: owner(), token: 409 };
/// The start goal sheet (client/browser.md, 2.5).
pub const SHEET: Style = Style { owner: owner(), token: 410 };
/// The sheet muted words (client/browser.md, 2.5).
pub const SHEET_MUTED: Style = Style { owner: owner(), token: 411 };
/// The sheet ways fieldset (client/browser.md, 2.5).
pub const SHEET_FIELDSET: Style = Style { owner: owner(), token: 412 };
/// A sheet way option (client/browser.md, 2.5).
pub const SHEET_OPTION: Style = Style { owner: owner(), token: 413 };
/// The sheet ways link (client/browser.md, 2.5).
pub const SHEET_LINK: Style = Style { owner: owner(), token: 414 };
/// The sheet actions (client/browser.md, 2.5).
pub const SHEET_FOOT: Style = Style { owner: owner(), token: 415 };
/// The sheet error (client/browser.md, 2.5).
pub const SHEET_ERROR: Style = Style { owner: owner(), token: 416 };

/// The session toast (client/README.md, 3.5).
pub const TOAST: Style = Style { owner: owner(), token: 300 };
/// The Goals column (client/README.md, 3.5).
pub const GOALS: Style = Style { owner: owner(), token: 270 };
/// Goals’ intro (client/README.md, 3.5).
pub const GOALS_INTRO: Style = Style { owner: owner(), token: 271 };
/// Goals’ project and page name (client/README.md, 3.5).
pub const GOALS_KICKER: Style = Style { owner: owner(), token: 272 };
/// Goals’ count headline (client/README.md, 3.5).
pub const GOALS_HEADLINE: Style = Style { owner: owner(), token: 273 };
/// Goals’ ways of working (client/README.md, 3.5).
pub const GOALS_LEDE: Style = Style { owner: owner(), token: 274 };
/// Goals’ byline (client/README.md, 3.5).
pub const GOALS_BYLINE: Style = Style { owner: owner(), token: 275 };
/// A Goals section (client/README.md, 3.5).
pub const GOALS_SECTION: Style = Style { owner: owner(), token: 276 };
/// A Goals section’s heading and actions (client/README.md, 3.5).
pub const GOALS_SECTION_HEAD: Style = Style { owner: owner(), token: 277 };
/// A Goals section’s title (client/README.md, 3.5).
pub const GOALS_SECTION_TITLE: Style = Style { owner: owner(), token: 278 };
/// A Goals section’s signal colour (client/README.md, 3.5).
pub const GOALS_SIGNAL: Style = Style { owner: owner(), token: 279 };
/// An empty Goals list (client/README.md, 3.5).
pub const GOALS_EMPTY: Style = Style { owner: owner(), token: 280 };
/// Goals’ facts (client/README.md, 3.5).
pub const GOALS_FACTS: Style = Style { owner: owner(), token: 281 };
/// The Goals rows (client/README.md, 3.5).
pub const GOALS_ROWS: Style = Style { owner: owner(), token: 282 };
/// A goal’s row link (client/README.md, 3.5).
pub const GOALS_ROW: Style = Style { owner: owner(), token: 283 };
/// An ended goal’s row (client/README.md, 3.5).
pub const GOALS_ROW_ENDED: Style = Style { owner: owner(), token: 284 };
/// A goal’s title, counts and state (client/README.md, 3.5).
pub const GOALS_TOP: Style = Style { owner: owner(), token: 285 };
/// A goal’s report headline (client/README.md, 3.5).
pub const GOALS_LINE: Style = Style { owner: owner(), token: 286 };
/// Goals’ filter buttons (client/README.md, 3.5).
pub const GOALS_FILTERS: Style = Style { owner: owner(), token: 287 };
/// The button to read more goals (client/README.md, 3.5).
pub const GOALS_MORE: Style = Style { owner: owner(), token: 288 };
/// A goal needing the person (client/README.md, 3.5).
pub const GOALS_STATE_NEEDS: Style = Style { owner: owner(), token: 289 };
/// A running goal (client/README.md, 3.5).
pub const GOALS_STATE_RUNNING: Style = Style { owner: owner(), token: 290 };
/// A planning goal (client/README.md, 3.5).
pub const GOALS_STATE_PLANNING: Style = Style { owner: owner(), token: 291 };
/// A goal paused by the person (client/README.md, 3.5).
pub const GOALS_STATE_PAUSED: Style = Style { owner: owner(), token: 292 };
/// A landed goal (client/README.md, 3.5).
pub const GOALS_STATE_LANDED: Style = Style { owner: owner(), token: 293 };
/// A closed or failed goal (client/README.md, 3.5).
pub const GOALS_STATE_CLOSED: Style = Style { owner: owner(), token: 294 };
/// Changes page parts (temper/docs/design/client/README.md, 3.5).
pub const CHANGES: Style = Style { owner: owner(), token: 180 };
/// Changes page parts (temper/docs/design/client/README.md, 3.5).
pub const CHANGES_MORE: Style = Style { owner: owner(), token: 181 };
/// Changes page parts (temper/docs/design/client/README.md, 3.5).
pub const CHANGES_MONO: Style = Style { owner: owner(), token: 182 };
/// Changes page parts (temper/docs/design/client/README.md, 3.5).
pub const CHANGES_RUNNING: Style = Style { owner: owner(), token: 183 };
/// Changes page parts (temper/docs/design/client/README.md, 3.5).
pub const CHANGES_RED: Style = Style { owner: owner(), token: 184 };

/// The Chats page and its parts (client/README.md, 3.5).
pub const CHATS_PAGE: Style = Style { owner: owner(), token: 254 };
pub const CHATS_INTRO: Style = Style { owner: owner(), token: 255 };
pub const CHATS_KICKER: Style = Style { owner: owner(), token: 256 };
pub const CHATS_LEDE: Style = Style { owner: owner(), token: 257 };
pub const CHATS_BAND: Style = Style { owner: owner(), token: 258 };
pub const CHATS_TAG: Style = Style { owner: owner(), token: 259 };
pub const CHATS_SECTION: Style = Style { owner: owner(), token: 260 };
pub const CHATS_SECTION_HEAD: Style = Style { owner: owner(), token: 261 };
pub const CHATS_ACT: Style = Style { owner: owner(), token: 262 };
pub const CHATS_MORE: Style = Style { owner: owner(), token: 263 };
pub const CHATS_ROWS: Style = Style { owner: owner(), token: 264 };
pub const CHATS_ROW: Style = Style { owner: owner(), token: 265 };
pub const CHATS_ROW_ENDED: Style = Style { owner: owner(), token: 266 };
pub const CHATS_TOP: Style = Style { owner: owner(), token: 267 };
pub const CHATS_LINE: Style = Style { owner: owner(), token: 268 };
pub const STATE_NEEDS: Style = Style { owner: owner(), token: 269 };
/// A session's byline, status, spend and controls (client/README.md, 3.5).
pub const SESSION_RIGHT: Style = Style { owner: owner(), token: 246 };
/// A chat's scrolling column and working indicator (client/README.md, 3.5).
pub const THREAD_INNER: Style = Style { owner: owner(), token: 321 };
pub const TYPING: Style = Style { owner: owner(), token: 322 };
pub const TURN_TOOL: Style = Style { owner: owner(), token: 324 };
pub const TOOL_FILE: Style = Style { owner: owner(), token: 325 };
pub const BY_DELEGATE: Style = Style { owner: owner(), token: 323 };
/// A new chat's centred repository context (client/README.md, 3.5).
pub const META_LINE: Style = Style { owner: owner(), token: 253 };
/// A session’s main. (client/README.md, 3.5).
pub const SESSION: Style = Style { owner: owner(), token: 234 };
/// The session’s path and runs’ line. (client/README.md, 3.5).
pub const SESSION_TOP: Style = Style { owner: owner(), token: 235 };
/// The session’s composer form. (client/README.md, 3.5).
pub const SESSION_FOOT: Style = Style { owner: owner(), token: 236 };
/// The session’s text area and Send. (client/README.md, 3.5).
pub const SESSION_COMPOSER: Style = Style { owner: owner(), token: 237 };
/// The session’s keyboard hint. (client/README.md, 3.5).
pub const SESSION_HINT: Style = Style { owner: owner(), token: 238 };
/// The session’s conversation region. (client/README.md, 3.5).
pub const CONVO: Style = Style { owner: owner(), token: 239 };
/// The session’s Where navigation. (client/README.md, 3.5).
pub const CRUMBS: Style = Style { owner: owner(), token: 240 };
/// The phone’s link back to Chats or Settings. (client/README.md, 3.5).
pub const BACK: Style = Style { owner: owner(), token: 241 };
/// The wide page’s path through Home and Chats or Settings. (client/README.md, 3.5).
pub const TRAIL: Style = Style { owner: owner(), token: 242 };
/// The session’s account and model line. (client/README.md, 3.5).
pub const BYLINE: Style = Style { owner: owner(), token: 243 };
/// The configuring agent’s introduction and starters. (client/README.md, 3.5).
pub const STARTER: Style = Style { owner: owner(), token: 244 };
/// The configuring session’s suggested first messages. (client/README.md, 3.5).
pub const STARTERS: Style = Style { owner: owner(), token: 245 };
/// The configuring opening’s lede (client/README.md, 3.5).
pub const STARTER_LEDE: Style = Style { owner: owner(), token: 248 };
/// The configuring opening’s sets (client/README.md, 3.5).
pub const STARTER_SETS: Style = Style { owner: owner(), token: 249 };
/// The configuring opening’s note (client/README.md, 3.5).
pub const STARTER_NOTE: Style = Style { owner: owner(), token: 250 };
/// The configuring opening’s ways (client/README.md, 3.5).
pub const STARTER_WAYS: Style = Style { owner: owner(), token: 251 };
/// The configuring opening’s unsure (client/README.md, 3.5).
pub const STARTER_UNSURE: Style = Style { owner: owner(), token: 252 };
/// The rail switcher's panel.
pub const PROJECTS_MENU: Style = Style { owner: owner(), token: 190 };
/// The rail switcher's actions.
pub const PROJECTS_ACTS: Style = Style { owner: owner(), token: 191 };
/// A switcher row's repository words.
pub const PROJECTS_REPO: Style = Style { owner: owner(), token: 192 };
/// The additional repository count.
pub const PROJECTS_MORE: Style = Style { owner: owner(), token: 193 };
/// A switcher row: the name and its small words.
pub const PROJECTS_ROW: Style = Style { owner: owner(), token: 194 };
/// A repository link in the look of its surrounding words.
pub const REPOSITORY_LINK: Style = Style { owner: owner(), token: 195 };
/// Home's home (client/README.md, 3.5).
pub const HOME: Style = Style { owner: owner(), token: 200 };
/// Home's intro (client/README.md, 3.5).
pub const HOME_INTRO: Style = Style { owner: owner(), token: 201 };
/// Home's kicker (client/README.md, 3.5).
pub const HOME_KICKER: Style = Style { owner: owner(), token: 202 };
/// Home's lede (client/README.md, 3.5).
pub const HOME_LEDE: Style = Style { owner: owner(), token: 203 };
/// Home's begin (client/README.md, 3.5).
pub const HOME_BEGIN: Style = Style { owner: owner(), token: 204 };
/// Home's steps (client/README.md, 3.5).
pub const HOME_STEPS: Style = Style { owner: owner(), token: 205 };
/// Home's step (client/README.md, 3.5).
pub const HOME_STEP: Style = Style { owner: owner(), token: 206 };
/// Home's acts (client/README.md, 3.5).
pub const HOME_ACTS: Style = Style { owner: owner(), token: 207 };
/// Home's primary (client/README.md, 3.5).
pub const HOME_PRIMARY: Style = Style { owner: owner(), token: 208 };
/// Home's quiet (client/README.md, 3.5).
pub const HOME_QUIET: Style = Style { owner: owner(), token: 209 };
/// Home's done (client/README.md, 3.5).
pub const HOME_DONE: Style = Style { owner: owner(), token: 213 };
/// Home's Needs you region (client/README.md, 3.5).
pub const HOME_NEEDS: Style = Style { owner: owner(), token: 214 };
/// A Home section's heading and link (client/README.md, 3.5).
pub const HOME_HEAD: Style = Style { owner: owner(), token: 215 };
/// The Needs you heading while something needs the person (client/README.md, 3.5).
pub const HOME_SIGNAL: Style = Style { owner: owner(), token: 216 };
/// Home's wide-page Answer in a row link (client/README.md, 3.5).
pub const HOME_IN_A_ROW: Style = Style { owner: owner(), token: 217 };
/// Home's linked needs or chat rows (client/README.md, 3.5).
pub const HOME_ROWS: Style = Style { owner: owner(), token: 218 };
/// A Home row's kind or state and age (client/README.md, 3.5).
pub const HOME_WHEN: Style = Style { owner: owner(), token: 219 };
/// A Home section's empty words (client/README.md, 3.5).
pub const HOME_EMPTY: Style = Style { owner: owner(), token: 220 };
/// Home's Chats region (client/README.md, 3.5).
pub const HOME_CHATS: Style = Style { owner: owner(), token: 221 };
/// Home's phone question and Answer link (client/README.md, 3.5).
pub const HOME_CTA: Style = Style { owner: owner(), token: 222 };
/// Home's phone Answer control (client/README.md, 3.5).
pub const HOME_CTA_GO: Style = Style { owner: owner(), token: 223 };
/// To begin's optional tag (client/README.md, 3.5).
pub const HOME_TAG: Style = Style { owner: owner(), token: 229 };
/// To begin's Skip button (client/README.md, 3.5).
pub const HOME_SKIP: Style = Style { owner: owner(), token: 231 };
/// Home’s lede and Accounts link (client/README.md, 3.5).
pub const HOME_LEDES: Style = Style { owner: owner(), token: 233 };
/// Home’s small repository actions (client/README.md, 3.5).
pub const HOME_SMALL: Style = Style { owner: owner(), token: 372 };
/// A repository’s progress line (client/README.md, 3.5).
pub const HOME_COPY: Style = Style { owner: owner(), token: 371 };
/// To begin’s remaining steps note (client/README.md, 3.5).
pub const HOME_NOTE: Style = Style { owner: owner(), token: 232 };
/// Home's first needs card (client/README.md, 3.5).
pub const ASK: Style = Style { owner: owner(), token: 224 };
/// The needs card's task link, kind and age (client/README.md, 3.5).
pub const ASK_META: Style = Style { owner: owner(), token: 225 };
/// The needs card's words (client/README.md, 3.5).
pub const ASK_QUESTION: Style = Style { owner: owner(), token: 226 };
/// The needs card's actions (client/README.md, 3.5).
pub const ASK_CHOICES: Style = Style { owner: owner(), token: 227 };
/// A quiet action on the needs card (client/README.md, 3.5).
pub const ASK_QUIET: Style = Style { owner: owner(), token: 228 };
/// The frame's banner.
pub const BANNER: Style = Style { owner: owner(), token: 0 };
/// The frame's navigation.
pub const NAVIGATION: Style = Style { owner: owner(), token: 1 };
/// Every page's main.
pub const MAIN: Style = Style { owner: owner(), token: 2 };
/// A chat's cards and departures.
pub const CARD: Style = Style { owner: owner(), token: 3 };
/// Each line of a chat's conversation.
pub const TURN: Style = Style { owner: owner(), token: 4 };
/// The chats' and a chat's composers.
pub const COMPOSER: Style = Style { owner: owner(), token: 5 };
/// The banner's wordmark.
pub const BRAND: Style = Style { owner: owner(), token: 6 };
/// The live link status.
pub const LIVE: Style = Style { owner: owner(), token: 7 };
/// The connecting link status.
pub const CONNECTING: Style = Style { owner: owner(), token: 8 };
/// The offline link status.
pub const OFFLINE: Style = Style { owner: owner(), token: 9 };
/// The link region's behind status.
pub const BEHIND: Style = Style { owner: owner(), token: 10 };
/// A chats section's list.
pub const ROWS: Style = Style { owner: owner(), token: 11 };
/// A chats row.
pub const ROW: Style = Style { owner: owner(), token: 12 };
/// The chats' composer box.
pub const ENTRY: Style = Style { owner: owner(), token: 13 };
/// A chat waiting for the reader.
pub const STATUS_YOURS: Style = Style { owner: owner(), token: 14 };
/// A held chat.
pub const STATUS_HELD: Style = Style { owner: owner(), token: 15 };
/// A retrying chat.
pub const STATUS_RETRYING: Style = Style { owner: owner(), token: 16 };
/// A chat starting or doing work.
pub const STATUS_ACTIVE: Style = Style { owner: owner(), token: 17 };
/// A chat waiting, parked, closing or ended.
pub const STATUS_QUIET: Style = Style { owner: owner(), token: 18 };
/// A chats row's absolute UTC time.
pub const TIME: Style = Style { owner: owner(), token: 19 };
/// The start composer's keyboard hint.
pub const HINT: Style = Style { owner: owner(), token: 20 };
/// The banner's corner: link state, person and sign out.
pub const CORNER: Style = Style { owner: owner(), token: 21 };
/// The accounts' banner: no usable account, its words and acts.
pub const ACCOUNTS: Style = Style { owner: owner(), token: 22 };
/// The composer's account and model line.
pub const ACCOUNT_LINE: Style = Style { owner: owner(), token: 23 };
/// The agent task page main.
pub const AGENT_PAGE: Style = Style { owner: owner(), token: 24 };
/// The agent task identity and controls.
pub const TASKBAR: Style = Style { owner: owner(), token: 25 };
/// The conversation and its tail.
pub const THREAD: Style = Style { owner: owner(), token: 26 };
/// One message in a conversation turn.
pub const MESSAGE: Style = Style { owner: owner(), token: 27 };
/// A message spoken by a person.
pub const BY_PERSON: Style = Style { owner: owner(), token: 28 };
/// A message spoken by the agent.
pub const BY_AGENT: Style = Style { owner: owner(), token: 29 };
/// A message spoken by a tool.
pub const BY_TOOL: Style = Style { owner: owner(), token: 30 };
/// A message containing instructions.
pub const BY_SYSTEM: Style = Style { owner: owner(), token: 31 };
/// The speaker and message state.
pub const WHO: Style = Style { owner: owner(), token: 32 };
/// The contents of a message.
pub const SAYS: Style = Style { owner: owner(), token: 33 };
/// A tool call or result line.
pub const WORK: Style = Style { owner: owner(), token: 34 };
/// A run or omission marker.
pub const MARKER: Style = Style { owner: owner(), token: 35 };
/// The chat proposal and departure list.
pub const CARDS: Style = Style { owner: owner(), token: 36 };
/// The chat composer and account area.
pub const COMPOSER_AREA: Style = Style { owner: owner(), token: 37 };
/// A list page main.
pub const PAGE: Style = Style { owner: owner(), token: 38 };
/// The page breadcrumb bar.
pub const PAGEBAR: Style = Style { owner: owner(), token: 39 };
/// A section heading row.
pub const HEADING: Style = Style { owner: owner(), token: 40 };
/// The heading row filters.
pub const FILTERS: Style = Style { owner: owner(), token: 41 };
/// The inbox updates list.
pub const UPDATES: Style = Style { owner: owner(), token: 42 };
/// A card kind and path.
pub const META: Style = Style { owner: owner(), token: 43 };
/// A card title.
pub const TITLE: Style = Style { owner: owner(), token: 44 };
/// A card supplied facts.
pub const DETAILS: Style = Style { owner: owner(), token: 45 };
/// An inbox proposal.
pub const KIND_PROPOSAL: Style = Style { owner: owner(), token: 46 };
/// An inbox question.
pub const KIND_QUESTION: Style = Style { owner: owner(), token: 47 };
/// An inbox held task.
pub const KIND_HELD: Style = Style { owner: owner(), token: 48 };
/// An inbox person task.
pub const KIND_PERSON_TASK: Style = Style { owner: owner(), token: 49 };
/// An inbox result.
pub const KIND_RESULT: Style = Style { owner: owner(), token: 50 };
/// An inbox reply.
pub const KIND_REPLY: Style = Style { owner: owner(), token: 51 };
/// The task page header region.
pub const TASK_HEADING: Style = Style { owner: owner(), token: 52 };
/// A task page card region.
pub const TASK_CARD: Style = Style { owner: owner(), token: 53 };
/// The plan tree.
pub const PLAN: Style = Style { owner: owner(), token: 54 };
/// One plan row's own parts.
pub const PLAN_ROW: Style = Style { owner: owner(), token: 55 };
/// The list around a held row card.
pub const PLAN_HOLD: Style = Style { owner: owner(), token: 56 };
/// A done task phase or a settled account outcome.
pub const STATUS_DONE: Style = Style { owner: owner(), token: 58 };
/// The next-run model choice block.
pub const CHOICE: Style = Style { owner: owner(), token: 59 };
/// The accounts page body, separate from the banner notice.
pub const ACCOUNTS_PAGE: Style = Style { owner: owner(), token: 60 };
/// The repository adoption page body.
pub const ADOPTION_PAGE: Style = Style { owner: owner(), token: 61 };
/// A supplied refusal, still announced as a status.
pub const REFUSAL: Style = Style { owner: owner(), token: 63 };
/// A change's steps on its delegation card.
pub const STEPS: Style = Style { owner: owner(), token: 64 };
/// One step on a change's delegation card.
pub const STEP: Style = Style { owner: owner(), token: 65 };
/// A completed step on a change's delegation card.
pub const STEP_DONE: Style = Style { owner: owner(), token: 66 };
/// The playground's project, repository and signed-in role.
pub const CONTEXT: Style = Style { owner: owner(), token: 67 };
/// The existing guidance when the application is unavailable.
pub const UNAVAILABLE: Style = Style { owner: owner(), token: 68 };
/// A plan row's spend.
pub const PLAN_SPEND: Style = Style { owner: owner(), token: 71 };
/// A plan row's dependency line.
pub const DEPENDENCIES: Style = Style { owner: owner(), token: 72 };
/// A task waiting on its dependencies, delegates, an answer or news.
pub const STATUS_WAITING: Style = Style { owner: owner(), token: 73 };
/// An agent task's parent and own identity.
pub const AGENT_LINEAGE: Style = Style { owner: owner(), token: 74 };
/// An agent task's specification, requester and dependencies.
pub const AGENT_BRIEF: Style = Style { owner: owner(), token: 75 };
/// An agent task's current or last run heading.
pub const AGENT_RUN: Style = Style { owner: owner(), token: 76 };
/// An ended agent task's result.
pub const AGENT_RESULT: Style = Style { owner: owner(), token: 77 };
/// The playground context line's repository.
pub const REPOSITORY: Style = Style { owner: owner(), token: 78 };
/// A change's offered acts (client/README.md, 5.10).
pub const TASK_ACTS: Style = Style { owner: owner(), token: 79 };
/// The agent task composer's steering note (client/README.md, 5.11).
pub const AGENT_STEER: Style = Style { owner: owner(), token: 80 };
/// The task page's amendment editor (client/README.md, 5.11).
pub const TASK_AMEND: Style = Style { owner: owner(), token: 82 };
/// A chat's end line (client/README.md, 3.5).
pub const ENDED: Style = Style { owner: owner(), token: 83 };
/// The day before a chat's turns (client/README.md, 5.12).
pub const DAY: Style = Style { owner: owner(), token: 84 };
/// The banner's menu, folded by the browser at phone width.
pub const MENU: Style = Style { owner: owner(), token: 85 };
/// A change's card on its task page (client/README.md, 5.10).
pub const CHANGE: Style = Style { owner: owner(), token: 86 };
/// The tag naming a change in its task page's header (5.10).
pub const CHANGE_TAG: Style = Style { owner: owner(), token: 87 };
/// The step a change is at (5.10).
pub const STEP_CURRENT: Style = Style { owner: owner(), token: 88 };
/// The notice of a held change (5.10).
pub const CHANGE_HOLD: Style = Style { owner: owner(), token: 89 };
/// One of a change's gates, CI's among them (5.10).
pub const GATE: Style = Style { owner: owner(), token: 90 };
/// An ended plan row (5.8).
pub const PLAN_DONE: Style = Style { owner: owner(), token: 91 };
/// The navigation inbox count.
pub const NAV_COUNT: Style = Style { owner: owner(), token: 92 };
/// Labelled facts in a proposal card.
pub const PROPOSAL_FACTS: Style = Style { owner: owner(), token: 93 };
/// An empty list's words, in a dashed box.
pub const EMPTY: Style = Style { owner: owner(), token: 94 };
/// The proposal type above its title.
pub const PROPOSAL_KICKER: Style = Style { owner: owner(), token: 95 };
/// Inbox presentation (client/README.md, 3.5).
pub const INBOX_TAG: Style = Style { owner: owner(), token: 96 };
/// Inbox presentation (client/README.md, 3.5).
pub const SUBTLE_COUNT: Style = Style { owner: owner(), token: 97 };
/// Inbox presentation (client/README.md, 3.5).
pub const KIND_MENU: Style = Style { owner: owner(), token: 98 };
/// Inbox presentation (client/README.md, 3.5).
pub const WAIT: Style = Style { owner: owner(), token: 99 };
/// Inbox presentation (client/README.md, 3.5).
pub const UPDATE_TIME: Style = Style { owner: owner(), token: 100 };
/// Centred first-run column.
pub const WELCOME: Style = Style { owner: owner(), token: 101 };
/// Four wizard steps.
pub const WELCOME_STEPS: Style = Style { owner: owner(), token: 102 };
/// One wizard step, marked when current.
pub const WELCOME_STEP: Style = Style { owner: owner(), token: 103 };
/// The playground and setup cards.
pub const WELCOME_CHOICES: Style = Style { owner: owner(), token: 104 };
/// One first-run choice.
pub const WELCOME_CARD: Style = Style { owner: owner(), token: 105 };
/// The step’s introduction.
pub const WELCOME_LEAD: Style = Style { owner: owner(), token: 106 };
/// A choice’s cost and time.
pub const WELCOME_MUTED: Style = Style { owner: owner(), token: 107 };
/// The closing invitation to connect real backends.
pub const WELCOME_AFTER: Style = Style { owner: owner(), token: 108 };
/// Back, Later and the playground alternative.
pub const WELCOME_FOOT: Style = Style { owner: owner(), token: 109 };
/// Shared flow presentation (client/README.md, 5.2).
pub const FLOW_STEPS: Style = Style { owner: owner(), token: 110 };
/// Shared flow presentation (client/README.md, 5.2).
pub const FLOW_STEP: Style = Style { owner: owner(), token: 111 };
/// Shared flow presentation (client/README.md, 5.2).
pub const FLOW_DONE: Style = Style { owner: owner(), token: 112 };
/// Shared flow presentation (client/README.md, 5.2).
pub const FLOW_CARD: Style = Style { owner: owner(), token: 113 };
/// Shared flow presentation (client/README.md, 5.2).
pub const FLOW_MUTED: Style = Style { owner: owner(), token: 114 };
/// Shared flow presentation (client/README.md, 5.2).
pub const FLOW_PROGRESS: Style = Style { owner: owner(), token: 115 };
/// A navigation entry needs the person.
pub const NAV_NEEDS: Style = Style { owner: owner(), token: 116 };
/// A navigation entry is limited.
pub const NAV_LIMITED: Style = Style { owner: owner(), token: 117 };
/// The forge connection flow's main.
pub const CONNECT_FORGE: Style = Style { owner: owner(), token: 118 };
/// A configuration entry or terminal command.
pub const FLOW_CODE: Style = Style { owner: owner(), token: 119 };
/// The Forges page (client/forge.md, 5.4).
pub const FORGES: Style = Style { owner: owner(), token: 120 };
/// The setup strip (client/README.md, 5.1, 5.6).
pub const SETUP_STRIP: Style = Style { owner: owner(), token: 122 };
/// Browser sign-in page (protocol/sign-in.md, 7).
pub const SIGN_IN: Style = Style { owner: owner(), token: 123 };
/// A connection flow's filled primary control.
pub const FLOW_PRIMARY: Style = Style { owner: owner(), token: 125 };
/// A failed line in a connection flow's progress.
pub const FLOW_FAILED: Style = Style { owner: owner(), token: 126 };
/// Organization and repository fields on one responsive row.
pub const FLOW_PAIR: Style = Style { owner: owner(), token: 127 };
/// The opening welcome keeps its introductory heading scale.
pub const WELCOME_INTRO: Style = Style { owner: owner(), token: 128 };
/// The navigation's project label when no project remains.
pub const PROJECT_EMPTY: Style = Style { owner: owner(), token: 129 };
/// A neutral small action.
pub const FLOW_SECONDARY: Style = Style { owner: owner(), token: 130 };
/// A quiet terracotta text link.
pub const FLOW_QUIET: Style = Style { owner: owner(), token: 131 };
/// The forge choices, three across on desktop.
pub const FLOW_CHOICES: Style = Style { owner: owner(), token: 132 };
/// A choice card with bottom-aligned actions.
pub const FLOW_CHOICE: Style = Style { owner: owner(), token: 133 };
/// A stacked choice action group.
pub const FLOW_ACTIONS: Style = Style { owner: owner(), token: 134 };
/// A small muted setup field label.
pub const FLOW_FIELD: Style = Style { owner: owner(), token: 135 };
/// The setup notes as muted bullet points.
pub const FLOW_FACTS: Style = Style { owner: owner(), token: 136 };
/// The model choices, two across on desktop.
pub const MODEL_CHOICES: Style = Style { owner: owner(), token: 137 };
/// A connection details card, separate from Model’s outer Choose card.
pub const FLOW_DETAILS: Style = Style { owner: owner(), token: 138 };
/// The API key card limits its contents to the reference's 420 px maximum.
pub const MODEL_KEY: Style = Style { owner: owner(), token: 139 };
/// The configured admin's first repository group.
pub const ADMIN_FIRST: Style = Style { owner: owner(), token: 140 };
/// The first repository group's legend.
pub const ADMIN_LEGEND: Style = Style { owner: owner(), token: 141 };
/// The Model ending’s definition list, scoped away from Accounts and Forges.
pub const MODEL_FACTS: Style = Style { owner: owner(), token: 142 };
/// The Model ending’s wrapping row of actions.
pub const MODEL_ACTIONS: Style = Style { owner: owner(), token: 143 };
/// A muted repository line tight beneath its setup heading.
pub const FLOW_SUBTITLE: Style = Style { owner: owner(), token: 144 };
/// The muted description of a Model way’s Details.
pub const MODEL_DESCRIPTION: Style = Style { owner: owner(), token: 145 };
/// Native terms and definitions inside the Model ending’s list.
pub const MODEL_FACT: Style = Style { owner: owner(), token: 146 };
/// The Model sign-in’s external address and folded paste.
pub const MODEL_SIGN_IN: Style = Style { owner: owner(), token: 147 };
/// Ready’s sign-in ending below Model’s line.
pub const WELCOME_END: Style = Style { owner: owner(), token: 149 };
/// The small note below Ready’s disabled action.
pub const WELCOME_TINY: Style = Style { owner: owner(), token: 150 };
/// Model’s forge status beside its choices.
pub const WELCOME_STATUS: Style = Style { owner: owner(), token: 151 };
/// A private repository's tag in the first-run picker.
pub const REPOSITORY_PRIVATE: Style = Style { owner: owner(), token: 152 };
/// A repository radio, its native label and optional privacy tag on one row.
pub const REPOSITORY_CHOICE: Style = Style { owner: owner(), token: 153 };
/// Ready's project button and optional Forgejo link.
pub const WELCOME_ACTIONS: Style = Style { owner: owner(), token: 154 };
/// The selected repository row in the first-run picker.
pub const REPOSITORY_SELECTED: Style = Style { owner: owner(), token: 155 };
/// The setup card's forge and model lines in one compact block.
pub const WELCOME_SETUP_DESCRIPTION: Style = Style { owner: owner(), token: 156 };
/// The opening welcome’s software factory tagline.
pub const WELCOME_TAGLINE: Style = Style { owner: owner(), token: 157 };
/// The banner's project switcher (temper/docs/design/client/README.md, 5.1).
pub const PROJECTS: Style = Style { owner: owner(), token: 159 };
/// The banner's way out of the playground (temper/docs/design/client/README.md, 5.6).
pub const LEAVE_PLAYGROUND: Style = Style { owner: owner(), token: 160 };
/// The switcher's Rename, at the end of the open project's row (temper/docs/design/client/README.md, 5.3).
pub const PROJECT_RENAME: Style = Style { owner: owner(), token: 161 };
/// The switcher's new project, its list's last row (temper/docs/design/client/README.md, 5.3).
pub const PROJECT_NEW: Style = Style { owner: owner(), token: 162 };
/// The open project's row while it is renamed (temper/docs/design/client/README.md, 5.3).
pub const PROJECT_RENAME_FORM: Style = Style { owner: owner(), token: 163 };
/// The welcome's temper box line, in its top bar (temper/docs/design/client/README.md, 3.5).
pub const WELCOME_BOX_LINE: Style = Style { owner: owner(), token: 164 };
/// The welcome's sheet for Forgejo on this machine, a modal dialog over the
/// Forge step (temper/docs/design/client/README.md, 3.5).
pub const WELCOME_SHEET: Style = Style { owner: owner(), token: 165 };
/// The sheet's heading and its line under it.
pub const WELCOME_SHEET_HEAD: Style = Style { owner: owner(), token: 166 };
/// What Forgejo starts with: the sheet's choices, a fieldset.
pub const WELCOME_START: Style = Style { owner: owner(), token: 167 };
/// The choices' legend.
pub const WELCOME_LEGEND: Style = Style { owner: owner(), token: 168 };
/// One choice as a card: its button, its name and its line.
pub const WELCOME_OPTION: Style = Style { owner: owner(), token: 169 };
/// The chosen one's own fields, under it.
pub const WELCOME_OPTION_MORE: Style = Style { owner: owner(), token: 170 };
/// The sheet's foot: Cancel, then Run on this machine.
pub const WELCOME_SHEET_FOOT: Style = Style { owner: owner(), token: 171 };
/// Ready's first project: its name's field, Create, and "Rename it
/// whenever you like.".
pub const WELCOME_PROJECT: Style = Style { owner: owner(), token: 172 };
/// The welcome forge's indeterminate setup bar (temper/docs/design/client/README.md, 3.5).
pub const FLOW_BUSY: Style = Style { owner: owner(), token: 173 };
/// The welcome forge's fetch meter (temper/docs/design/draft/fetch-progress.md, 5 and 6).
/// No node carries it yet.
pub const FLOW_METER: Style = Style { owner: owner(), token: 174 };
/// The project settings (temper/docs/design/client/README.md, 3.5).
pub const SETTINGS: Style = Style { owner: owner(), token: 230 };
/// Native browser elements for the admin group, Model’s definition list and
/// the welcome sheet's choices.
pub const ELEMENTS: &[(skein_ui::Owner, skein_ui::Role, Style, &str)] = &[
    (owner(), skein_ui::Role::Text, SETTINGS_DETAIL, "small"),
    (owner(), skein_ui::Role::Region, SETTINGS_PROPOSAL, "article"),
    (owner(), skein_ui::Role::ChoiceGroup, SHEET_FIELDSET, "fieldset"),
    (owner(), skein_ui::Role::Heading { level: 3 }, SHEET_FIELDSET, "legend"),
    (owner(), skein_ui::Role::Text, SHEET_MUTED, "small"),
    (owner(), skein_ui::Role::Region, ADMIN_FIRST, "fieldset"),
    (owner(), skein_ui::Role::Slot, WELCOME_SPIN, "span"),
    (owner(), skein_ui::Role::Slot, WELCOME_OK, "span"),
    (owner(), skein_ui::Role::ChoiceGroup, WELCOME_START, "fieldset"),
    (owner(), skein_ui::Role::Heading { level: 3 }, WELCOME_LEGEND, "legend"),
    (owner(), skein_ui::Role::Slot, MODEL_FACTS, "dl"),
    (owner(), skein_ui::Role::Term, MODEL_FACT, "dt"),
    (owner(), skein_ui::Role::Description, MODEL_FACT, "dd"),
    (owner(), skein_ui::Role::Heading { level: 3 }, ADMIN_LEGEND, "legend"),
];
/// The project's navigation frame (client/README.md, 5.3).
pub const RAIL: Style = Style { owner: owner(), token: 175 };
/// The rail's three page entries.
pub const RAIL_NAV: Style = Style { owner: owner(), token: 176 };
/// The rail's new chat link and menu.
pub const RAIL_NEW: Style = Style { owner: owner(), token: 177 };
/// All ways to start work in the rail menu (client/README.md, 5.3).
pub const RAIL_NEW_ALL: Style = Style { owner: owner(), token: 437 };
/// The rail's status, settings and person.
pub const RAIL_FOOT: Style = Style { owner: owner(), token: 178 };
/// The browser's phone navigation.
pub const TABBAR: Style = Style { owner: owner(), token: 179 };
/// Forge status in the welcome's column (client/README.md, 3.5).
pub const WELCOME_FORGE_STATUS: Style = Style { owner: owner(), token: 373 };
/// A Model way’s panel (client/README.md, 3.5).
pub const WELCOME_PANEL: Style = Style { owner: owner(), token: 365 };
/// A Model sign-in’s note (client/README.md, 3.5).
pub const WELCOME_AWAY: Style = Style { owner: owner(), token: 366 };
/// The waiting note’s external link (client/README.md, 3.5).
pub const WELCOME_AGAIN: Style = Style { owner: owner(), token: 460 };
/// The waiting page’s folded paste (client/README.md, 3.5).
pub const WELCOME_PASTE: Style = Style { owner: owner(), token: 461 };
/// A Model panel’s full-width secret field (client/README.md, 3.5).
pub const WELCOME_FIELD: Style = Style { owner: owner(), token: 368 };
/// The welcome’s command wrapper (temper/docs/design/client/README.md, 3.5).
pub const WELCOME_CMD_BOX: Style = Style { owner: owner(), token: 121 };
/// The welcome’s clipboard button (temper/docs/design/client/README.md, 3.5).
pub const WELCOME_COPY: Style = Style { owner: owner(), token: 367 };
/// The welcome’s terminal command (client/README.md, 3.5).
pub const WELCOME_CMD: Style = Style { owner: owner(), token: 369 };
/// The forge's spinning status marker.
pub const WELCOME_SPIN: Style = Style { owner: owner(), token: 361 };
/// The welcome's primary continuation.
pub const WELCOME_ACTS: Style = Style { owner: owner(), token: 362 };
/// The forge's connected tick.
pub const WELCOME_OK: Style = Style { owner: owner(), token: 363 };
/// The separator between the forge's name and its status.
pub const WELCOME_SEP: Style = Style { owner: owner(), token: 364 };
/// The look's names (skein's ui.md, 8): styles by token, regions by number.
pub const NAMES: Names<'static> = Names {
    owner: owner(),
    prefix: b"temper",
    styles: &[
        b"temper-banner",
        b"temper-navigation",
        b"temper-main",
        b"temper-card",
        b"temper-turn",
        b"temper-composer",
        b"temper-brand",
        b"temper-live",
        b"temper-connecting",
        b"temper-offline",
        b"temper-behind",
        b"temper-rows",
        b"temper-row",
        b"temper-entry",
        b"temper-status-yours",
        b"temper-status-held",
        b"temper-status-retrying",
        b"temper-status-active",
        b"temper-status-quiet",
        b"temper-time",
        b"temper-hint",
        b"temper-corner",
        b"temper-accounts",
        b"temper-account-line",
        b"temper-agent-page",
        b"temper-taskbar",
        b"temper-thread",
        b"temper-message",
        b"temper-by-person",
        b"temper-by-agent",
        b"temper-by-tool",
        b"temper-by-system",
        b"temper-who",
        b"temper-says",
        b"temper-work",
        b"temper-marker",
        b"temper-cards",
        b"temper-composer-area",
        b"temper-page",
        b"temper-pagebar",
        b"temper-heading",
        b"temper-filters",
        b"temper-updates",
        b"temper-meta",
        b"temper-title",
        b"temper-details",
        b"temper-kind-proposal",
        b"temper-kind-question",
        b"temper-kind-held",
        b"temper-kind-person-task",
        b"temper-kind-result",
        b"temper-kind-reply",
        b"temper-task-heading",
        b"temper-task-card",
        b"temper-plan",
        b"temper-plan-row",
        b"temper-plan-hold",
        b"temper-retired-events",
        b"temper-status-done",
        b"temper-choice",
        b"temper-accounts-page",
        b"temper-adoption-page",
        b"temper-reserved-62",
        b"temper-refusal",
        b"temper-steps",
        b"temper-step",
        b"temper-step-done",
        b"temper-context",
        b"temper-unavailable",
        b"temper-retired-task-contract",
        b"temper-retired-task-budget",
        b"temper-plan-spend",
        b"temper-dependencies",
        b"temper-status-waiting",
        b"temper-agent-lineage",
        b"temper-agent-brief",
        b"temper-agent-run",
        b"temper-agent-result",
        b"temper-repository",
        b"temper-task-acts",
        b"temper-agent-steer",
        b"temper-retired-task-danger",
        b"temper-task-amend",
        b"temper-ended",
        b"temper-day",
        b"temper-menu",
        b"temper-change",
        b"temper-change-tag",
        b"temper-step-current",
        b"temper-change-hold",
        b"temper-gate",
        b"temper-plan-done",
        b"temper-nav-count",
        b"temper-proposal-facts",
        b"temper-empty",
        b"temper-proposal-kicker",
        b"temper-inbox-tag",
        b"temper-subtle-count",
        b"temper-kind-menu",
        b"temper-wait",
        b"temper-update-time",
        b"temper-welcome",
        b"temper-welcome-steps",
        b"temper-welcome-step",
        b"temper-welcome-choices",
        b"temper-welcome-card",
        b"temper-welcome-lead",
        b"temper-welcome-muted",
        b"temper-welcome-after",
        b"temper-welcome-foot",
        b"temper-flow-steps",
        b"temper-flow-step",
        b"temper-flow-done",
        b"temper-flow-card",
        b"temper-flow-muted",
        b"temper-flow-progress",
        b"temper-nav-needs",
        b"temper-nav-limited",
        b"temper-connect-forge",
        b"temper-flow-code",
        b"temper-forges",
        b"temper-welcome-cmd-box",
        b"temper-setup-strip",
        b"temper-sign-in",
        b"temper-reserved-124",
        b"temper-flow-primary",
        b"temper-flow-failed",
        b"temper-flow-pair",
        b"temper-welcome-intro",
        b"temper-project-empty",
        b"temper-flow-secondary",
        b"temper-flow-quiet",
        b"temper-flow-choices",
        b"temper-flow-choice",
        b"temper-flow-actions",
        b"temper-flow-field",
        b"temper-flow-facts",
        b"temper-model-choices",
        b"temper-flow-details",
        b"temper-model-key",
        b"temper-admin-first",
        b"temper-admin-legend",
        b"temper-model-facts",
        b"temper-model-actions",
        b"temper-flow-subtitle",
        b"temper-model-description",
        b"temper-model-fact",
        b"temper-model-sign-in",
        b"temper-reserved-148",
        b"temper-welcome-end",
        b"temper-welcome-tiny",
        b"temper-welcome-status",
        b"temper-repository-private",
        b"temper-repository-choice",
        b"temper-welcome-actions",
        b"temper-repository-selected",
        b"temper-welcome-setup-description",
        b"temper-welcome-tagline",
        b"temper-reserved-158",
        b"temper-projects",
        b"temper-leave-playground",
        b"temper-project-rename",
        b"temper-project-new",
        b"temper-project-rename-form",
        b"temper-welcome-box-line",
        b"temper-welcome-sheet",
        b"temper-welcome-sheet-head",
        b"temper-welcome-start",
        b"temper-welcome-legend",
        b"temper-welcome-option",
        b"temper-welcome-option-more",
        b"temper-welcome-sheet-foot",
        b"temper-welcome-project",
        b"temper-flow-busy",
        b"temper-flow-meter",
        b"temper-rail",
        b"temper-rail-nav",
        b"temper-rail-new",
        b"temper-rail-foot",
        b"temper-tabbar",
        b"temper-changes",
        b"temper-changes-more",
        b"temper-changes-mono",
        b"temper-changes-running",
        b"temper-changes-red",
        b"temper-reserved-185",
        b"temper-reserved-186",
        b"temper-reserved-187",
        b"temper-reserved-188",
        b"temper-reserved-189",
        b"temper-projects-menu",
        b"temper-projects-acts",
        b"temper-projects-repo",
        b"temper-projects-more",
        b"temper-projects-row",
        b"temper-repository-link",
        b"temper-reserved-196",
        b"temper-reserved-197",
        b"temper-reserved-198",
        b"temper-reserved-199",
        b"temper-home",
        b"temper-home-intro",
        b"temper-home-kicker",
        b"temper-home-lede",
        b"temper-home-begin",
        b"temper-home-steps",
        b"temper-home-step",
        b"temper-home-acts",
        b"temper-home-primary",
        b"temper-home-quiet",
        b"temper-reserved-210",
        b"temper-reserved-211",
        b"temper-reserved-212",
        b"temper-home-done",
        b"temper-home-needs",
        b"temper-home-head",
        b"temper-home-signal",
        b"temper-home-in-a-row",
        b"temper-home-rows",
        b"temper-home-when",
        b"temper-home-empty",
        b"temper-home-chats",
        b"temper-home-cta",
        b"temper-home-cta-go",
        b"temper-ask",
        b"temper-ask-meta",
        b"temper-ask-question",
        b"temper-ask-choices",
        b"temper-ask-quiet",
        b"temper-home-tag",
        b"temper-settings",
        b"temper-home-skip",
        b"temper-home-note",
        b"temper-home-ledes",
        b"temper-session",
        b"temper-session-top",
        b"temper-session-foot",
        b"temper-session-composer",
        b"temper-session-hint",
        b"temper-convo",
        b"temper-crumbs",
        b"temper-back",
        b"temper-trail",
        b"temper-byline",
        b"temper-starter",
        b"temper-starters",
        b"temper-session-right",
        b"temper-reserved-247",
        b"temper-starter-lede",
        b"temper-starter-sets",
        b"temper-starter-note",
        b"temper-starter-ways",
        b"temper-starter-unsure",
        b"temper-meta-line",
        b"temper-chats-page",
        b"temper-chats-intro",
        b"temper-chats-kicker",
        b"temper-chats-lede",
        b"temper-chats-band",
        b"temper-chats-tag",
        b"temper-chats-section",
        b"temper-chats-section-head",
        b"temper-chats-act",
        b"temper-chats-more",
        b"temper-chats-rows",
        b"temper-chats-row",
        b"temper-chats-row-ended",
        b"temper-chats-top",
        b"temper-chats-line",
        b"temper-state-needs",
        b"temper-goals",
        b"temper-goals-intro",
        b"temper-goals-kicker",
        b"temper-goals-headline",
        b"temper-goals-lede",
        b"temper-goals-byline",
        b"temper-goals-section",
        b"temper-goals-section-head",
        b"temper-goals-section-title",
        b"temper-goals-signal",
        b"temper-goals-empty",
        b"temper-goals-facts",
        b"temper-goals-rows",
        b"temper-goals-row",
        b"temper-goals-row-ended",
        b"temper-goals-top",
        b"temper-goals-line",
        b"temper-goals-filters",
        b"temper-goals-more",
        b"temper-goals-state-needs",
        b"temper-goals-state-running",
        b"temper-goals-state-planning",
        b"temper-goals-state-paused",
        b"temper-goals-state-landed",
        b"temper-goals-state-closed",
        b"temper-reserved-295",
        b"temper-reserved-296",
        b"temper-reserved-297",
        b"temper-reserved-298",
        b"temper-reserved-299",
        b"temper-toast",
        b"temper-needs",
        b"temper-flow-head",
        b"temper-dots",
        b"temper-dot",
        b"temper-dot-done",
        b"temper-flow-q",
        b"temper-ask-context",
        b"temper-ask-saw",
        b"temper-recommended",
        b"temper-rec",
        b"temper-pair",
        b"temper-ask-reply",
        b"temper-ask-reply-acts",
        b"temper-answered",
        b"temper-answered-label",
        b"temper-answered-undo",
        b"temper-next-up",
        b"temper-answered-section",
        b"temper-needs-home",
        b"temper-needs-intro",
        b"temper-thread-inner",
        b"temper-typing",
        b"temper-by-delegate",
        b"temper-turn-tool",
        b"temper-tool-file",
        b"temper-settings-proposal",
        b"temper-settings-proposal-title",
        b"temper-settings-proposal-state",
        b"temper-settings-proposal-superseded",
        b"temper-settings-proposal-ok",
        b"temper-settings-proposal-items",
        b"temper-settings-proposal-reason",
        b"temper-settings-proposal-acts",
        b"temper-settings-col",
        b"temper-settings-intro",
        b"temper-settings-kicker",
        b"temper-settings-headline",
        b"temper-settings-lede",
        b"temper-settings-tabs",
        b"temper-settings-body",
        b"temper-settings-setting",
        b"temper-settings-section",
        b"temper-settings-section-head",
        b"temper-settings-section-title",
        b"temper-settings-lines",
        b"temper-settings-when",
        b"temper-settings-band",
        b"temper-settings-quiet-band",
        b"temper-settings-tag",
        b"temper-settings-btn",
        b"temper-settings-primary",
        b"temper-settings-small",
        b"temper-settings-detail",
        b"temper-settings-end",
        b"temper-settings-checks",
        b"temper-settings-switch",
        b"temper-settings-toggle-row",
        b"temper-settings-text",
        b"temper-settings-facts",
        b"temper-settings-refusal",
        b"temper-welcome-spin",
        b"temper-welcome-acts",
        b"temper-welcome-ok",
        b"temper-welcome-sep",
        b"temper-welcome-panel",
        b"temper-welcome-away",
        b"temper-welcome-copy",
        b"temper-welcome-field",
        b"temper-welcome-cmd",
        b"temper-reserved-370",
        b"temper-home-copy",
        b"temper-home-small",
        b"temper-welcome-forge-status",
        b"temper-settings-row",
        b"temper-settings-path",
        b"temper-settings-empty",
        b"temper-goal",
        b"temper-goal-head",
        b"temper-goal-meta",
        b"temper-goal-byline",
        b"temper-goal-acts",
        b"temper-goal-btn",
        b"temper-goal-small",
        b"temper-goal-quiet",
        b"temper-goal-primary",
        b"temper-goal-band",
        b"temper-goal-band-tag",
        b"temper-goal-choices",
        b"temper-goal-two",
        b"temper-goal-report",
        b"temper-goal-title",
        b"temper-goal-typing",
        b"temper-goal-side",
        b"temper-goal-section",
        b"temper-goal-facts",
        b"temper-goal-bar",
        b"temper-goal-bar-landed",
        b"temper-goal-bar-running",
        b"temper-goal-line",
        b"temper-goal-line-running",
        b"temper-goal-line-needs",
        b"temper-goal-landed-fold",
        b"temper-goal-sent",
        b"temper-goal-you",
        b"temper-goal-composer-box",
        b"temper-goal-composer",
        b"temper-goal-hint",
        b"temper-goal-field",
        b"temper-goal-text",
        b"temper-sheet",
        b"temper-sheet-muted",
        b"temper-sheet-fieldset",
        b"temper-sheet-option",
        b"temper-sheet-link",
        b"temper-sheet-foot",
        b"temper-sheet-error",
        b"temper-mode-chat",
        b"temper-mode-split",
        b"temper-mode-draft",
        b"temper-doc",
        b"temper-doc-inner",
        b"temper-doc-path",
        b"temper-doc-sec",
        b"temper-doc-changed",
        b"temper-doc-note",
        b"temper-doc-muted",
        b"temper-seg",
        b"temper-seg-split",
        b"temper-rail-unfold",
        b"temper-start-over",
        b"temper-doc-flash",
        b"temper-doc-empty",
        b"temper-add-draft",
        b"temper-home-facts",
        b"temper-home-byline",
        b"temper-chat-goal",
        b"temper-rail-new-all",
        b"temper-away",
        b"temper-away-chip",
        b"temper-away-track",
        b"temper-decided",
        b"temper-decision",
        b"temper-decision-took",
        b"temper-decision-why",
        b"temper-decision-late",
        b"temper-decision-overturned",
        b"temper-starter-head",
        b"temper-starter-own",
        b"temper-starter-own-text",
        b"temper-starter-own-title",
        b"temper-starter-own-steps",
        b"temper-starter-own-acts",
        b"temper-doc-sub",
        b"temper-doc-flow",
        b"temper-reserved-455",
        b"temper-reserved-456",
        b"temper-reserved-457",
        b"temper-reserved-458",
        b"temper-reserved-459",
        b"temper-welcome-again",
        b"temper-welcome-paste",
        b"temper-rail-person",
        b"temper-settings-sign-out",
        b"temper-confirm",
        b"temper-confirm-fates",
        b"temper-confirm-acts",
        b"temper-confirm-do",
        b"temper-confirm-choices",
        b"temper-writing-end",
        b"temper-work-run",
        b"temper-work-running",
        b"temper-work-failed",
        b"temper-work-calls",
        b"temper-work-out",
        b"temper-latest",
        b"temper-thread-loading",
        b"temper-reserved-477",
        b"temper-reserved-478",
        b"temper-reserved-479",
        b"temper-reserved-480",
        b"temper-reserved-481",
        b"temper-reserved-482",
        b"temper-reserved-483",
        b"temper-reserved-484",
        b"temper-reserved-485",
        b"temper-reserved-486",
        b"temper-reserved-487",
        b"temper-reserved-488",
        b"temper-reserved-489",
        b"temper-reserved-490",
        b"temper-reserved-491",
        b"temper-reserved-492",
        b"temper-reserved-493",
        b"temper-reserved-494",
        b"temper-reserved-495",
        b"temper-reserved-496",
        b"temper-reserved-497",
        b"temper-reserved-498",
        b"temper-reserved-499",
        b"temper-reserved-500",
        b"temper-reserved-501",
        b"temper-reserved-502",
        b"temper-reserved-503",
        b"temper-reserved-504",
        b"temper-reserved-505",
        b"temper-reserved-506",
        b"temper-reserved-507",
        b"temper-reserved-508",
        b"temper-reserved-509",
        b"temper-reserved-510",
        b"temper-reserved-511",
        b"temper-reserved-512",
        b"temper-reserved-513",
        b"temper-reserved-514",
        b"temper-reserved-515",
        b"temper-welcome-status-why",
        b"temper-welcome-status-acts",
    ],
    regions: &[b"temper-link"],
};

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const NEEDS: Style = Style { owner: owner(), token: 301 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const FLOW_HEAD: Style = Style { owner: owner(), token: 302 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const DOTS: Style = Style { owner: owner(), token: 303 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const DOT: Style = Style { owner: owner(), token: 304 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const DOT_DONE: Style = Style { owner: owner(), token: 305 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const FLOW_Q: Style = Style { owner: owner(), token: 306 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ASK_CONTEXT: Style = Style { owner: owner(), token: 307 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ASK_SAW: Style = Style { owner: owner(), token: 308 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const RECOMMENDED: Style = Style { owner: owner(), token: 309 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const REC: Style = Style { owner: owner(), token: 310 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const PAIR: Style = Style { owner: owner(), token: 311 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ASK_REPLY: Style = Style { owner: owner(), token: 312 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ASK_REPLY_ACTS: Style = Style { owner: owner(), token: 313 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ANSWERED: Style = Style { owner: owner(), token: 314 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ANSWERED_LABEL: Style = Style { owner: owner(), token: 315 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ANSWERED_UNDO: Style = Style { owner: owner(), token: 316 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const NEXT_UP: Style = Style { owner: owner(), token: 317 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const ANSWERED_SECTION: Style = Style { owner: owner(), token: 318 };

/// Needs you (temper/docs/design/client/README.md, 3.5).
pub const NEEDS_HOME: Style = Style { owner: owner(), token: 319 };

/// Done’s intro (temper/docs/design/client/README.md, 3.5).
pub const NEEDS_INTRO: Style = Style { owner: owner(), token: 320 };

/// Settings proposal cards (temper/docs/design/client/README.md, 5.13).
pub const SETTINGS_PROPOSAL: Style = Style { owner: owner(), token: 326 };
pub const SETTINGS_PROPOSAL_TITLE: Style = Style { owner: owner(), token: 327 };
pub const SETTINGS_PROPOSAL_STATE: Style = Style { owner: owner(), token: 328 };
pub const SETTINGS_PROPOSAL_SUPERSEDED: Style = Style { owner: owner(), token: 329 };
pub const SETTINGS_PROPOSAL_OK: Style = Style { owner: owner(), token: 330 };
pub const SETTINGS_PROPOSAL_ITEMS: Style = Style { owner: owner(), token: 331 };
pub const SETTINGS_PROPOSAL_REASON: Style = Style { owner: owner(), token: 332 };
pub const SETTINGS_PROPOSAL_ACTS: Style = Style { owner: owner(), token: 333 };
/// Settings frame and rows (temper/docs/design/client/README.md, 3.5, Settings).
pub const SETTINGS_COL: Style = Style { owner: owner(), token: 334 };
pub const SETTINGS_INTRO: Style = Style { owner: owner(), token: 335 };
pub const SETTINGS_KICKER: Style = Style { owner: owner(), token: 336 };
pub const SETTINGS_HEADLINE: Style = Style { owner: owner(), token: 337 };
pub const SETTINGS_LEDE: Style = Style { owner: owner(), token: 338 };
pub const SETTINGS_TABS: Style = Style { owner: owner(), token: 339 };
pub const SETTINGS_BODY: Style = Style { owner: owner(), token: 340 };
pub const SETTINGS_SETTING: Style = Style { owner: owner(), token: 341 };
pub const SETTINGS_SECTION: Style = Style { owner: owner(), token: 342 };
pub const SETTINGS_SECTION_HEAD: Style = Style { owner: owner(), token: 343 };
pub const SETTINGS_SECTION_TITLE: Style = Style { owner: owner(), token: 344 };
pub const SETTINGS_LINES: Style = Style { owner: owner(), token: 345 };
pub const SETTINGS_WHEN: Style = Style { owner: owner(), token: 346 };
pub const SETTINGS_BAND: Style = Style { owner: owner(), token: 347 };
pub const SETTINGS_QUIET_BAND: Style = Style { owner: owner(), token: 348 };
pub const SETTINGS_TAG: Style = Style { owner: owner(), token: 349 };
pub const SETTINGS_BTN: Style = Style { owner: owner(), token: 350 };
pub const SETTINGS_PRIMARY: Style = Style { owner: owner(), token: 351 };
pub const SETTINGS_SMALL: Style = Style { owner: owner(), token: 352 };
pub const SETTINGS_DETAIL: Style = Style { owner: owner(), token: 353 };
pub const SETTINGS_END: Style = Style { owner: owner(), token: 354 };
pub const SETTINGS_CHECKS: Style = Style { owner: owner(), token: 355 };
pub const SETTINGS_SWITCH: Style = Style { owner: owner(), token: 356 };
pub const SETTINGS_TOGGLE_ROW: Style = Style { owner: owner(), token: 357 };
pub const SETTINGS_TEXT: Style = Style { owner: owner(), token: 358 };
pub const SETTINGS_FACTS: Style = Style { owner: owner(), token: 359 };
pub const SETTINGS_REFUSAL: Style = Style { owner: owner(), token: 360 };
pub const SETTINGS_ROW: Style = Style { owner: owner(), token: 374 };
pub const SETTINGS_PATH: Style = Style { owner: owner(), token: 375 };
pub const SETTINGS_EMPTY: Style = Style { owner: owner(), token: 376 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const MODE_CHAT: Style = Style { owner: owner(), token: 417 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const MODE_SPLIT: Style = Style { owner: owner(), token: 418 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const MODE_DRAFT: Style = Style { owner: owner(), token: 419 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const DOC: Style = Style { owner: owner(), token: 420 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const DOC_INNER: Style = Style { owner: owner(), token: 421 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const DOC_PATH: Style = Style { owner: owner(), token: 422 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const DOC_SEC: Style = Style { owner: owner(), token: 423 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const DOC_CHANGED: Style = Style { owner: owner(), token: 424 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const DOC_NOTE: Style = Style { owner: owner(), token: 425 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const DOC_MUTED: Style = Style { owner: owner(), token: 426 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const SEG: Style = Style { owner: owner(), token: 427 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const SEG_SPLIT: Style = Style { owner: owner(), token: 428 };
/// A chat's document (temper/docs/design/client/README.md, 3.5).
pub const RAIL_UNFOLD: Style = Style { owner: owner(), token: 429 };

/// A configuring chat’s fresh start (temper/docs/design/client/README.md, 3.5).
pub const START_OVER: Style = Style { owner: owner(), token: 430 };

/// A shown section (temper/docs/design/client/README.md, 3.5).
pub const DOC_FLASH: Style = Style { owner: owner(), token: 431 };

/// The unwritten document (client/README.md, 3.5).
pub const DOC_EMPTY: Style = Style { owner: owner(), token: 432 };
/// The bar's draft action (client/README.md, 3.5).
pub const ADD_DRAFT: Style = Style { owner: owner(), token: 433 };

/// Home status (temper/docs/design/client/README.md, 3.5).
pub const HOME_FACTS: Style = Style { owner: owner(), token: 434 };
/// Home brief attribution (temper/docs/design/client/README.md, 3.5).
pub const HOME_BYLINE: Style = Style { owner: owner(), token: 435 };

/// A chat starts a goal or links its start (client/README.md, 3.5).
pub const CHAT_GOAL: Style = Style { owner: owner(), token: 436 };
/// Away controls (temper/docs/design/client/README.md, 3.5).
pub const AWAY: Style = Style { owner: owner(), token: 438 };
pub const AWAY_CHIP: Style = Style { owner: owner(), token: 439 };
pub const AWAY_TRACK: Style = Style { owner: owner(), token: 440 };

/// Home’s decisions (temper/docs/design/client/README.md, 3.5, Away).
pub const DECIDED: Style = Style { owner: owner(), token: 441 };
pub const DECISION: Style = Style { owner: owner(), token: 442 };
pub const DECISION_TOOK: Style = Style { owner: owner(), token: 443 };
pub const DECISION_WHY: Style = Style { owner: owner(), token: 444 };
pub const DECISION_LATE: Style = Style { owner: owner(), token: 445 };
pub const DECISION_OVERTURNED: Style = Style { owner: owner(), token: 446 };

/// The configuring page’s own way card (temper/docs/design/client/README.md, 3.5).
pub const STARTER_HEAD: Style = Style { owner: owner(), token: 447 };
pub const STARTER_OWN: Style = Style { owner: owner(), token: 448 };
pub const STARTER_OWN_TEXT: Style = Style { owner: owner(), token: 449 };
pub const STARTER_OWN_TITLE: Style = Style { owner: owner(), token: 450 };
pub const STARTER_OWN_STEPS: Style = Style { owner: owner(), token: 451 };
pub const STARTER_OWN_ACTS: Style = Style { owner: owner(), token: 452 };

/// The stock reference beside Configure (temper/docs/design/client/README.md, 3.5).
pub const DOC_SUB: Style = Style { owner: owner(), token: 453 };
pub const DOC_FLOW: Style = Style { owner: owner(), token: 454 };

/// The last word of a turn being written (client/README.md, 3.5).
pub const WRITING_END: Style = Style { owner: owner(), token: 469 };
/// A confirmation sheet (client/README.md, 3.6, The sheet).
pub const CONFIRM: Style = Style { owner: owner(), token: 464 };
/// A confirmation sheet (client/README.md, 3.6, The sheet).
pub const CONFIRM_FATES: Style = Style { owner: owner(), token: 465 };
/// A confirmation sheet (client/README.md, 3.6, The sheet).
pub const CONFIRM_ACTS: Style = Style { owner: owner(), token: 466 };
/// A confirmation sheet (client/README.md, 3.6, The sheet).
pub const CONFIRM_DO: Style = Style { owner: owner(), token: 467 };
/// An uncertain release's choices (client/README.md, 3.6, The sheet).
pub const CONFIRM_CHOICES: Style = Style { owner: owner(), token: 468 };
/// The browser's person menu (client/README.md, 5.3).
pub const RAIL_PERSON: Style = Style { owner: owner(), token: 462 };
/// The Accounts tab's sign-out row (client/README.md, 3.5).
pub const SETTINGS_SIGN_OUT: Style = Style { owner: owner(), token: 463 };

/// The conversation’s call runs and reading position (client/README.md, 3.5).
pub const WORK_RUN: Style = Style { owner: owner(), token: 470 };

/// The conversation’s call runs and reading position (client/README.md, 3.5).
pub const WORK_RUNNING: Style = Style { owner: owner(), token: 471 };

/// The conversation’s call runs and reading position (client/README.md, 3.5).
pub const WORK_FAILED: Style = Style { owner: owner(), token: 472 };

/// The conversation’s call runs and reading position (client/README.md, 3.5).
pub const WORK_CALLS: Style = Style { owner: owner(), token: 473 };

/// The conversation’s call runs and reading position (client/README.md, 3.5).
pub const WORK_OUT: Style = Style { owner: owner(), token: 474 };

/// The conversation’s call runs and reading position (client/README.md, 3.5).
pub const LATEST: Style = Style { owner: owner(), token: 475 };

/// The conversation’s call runs and reading position (client/README.md, 3.5).
pub const THREAD_LOADING: Style = Style { owner: owner(), token: 476 };

/// Ready’s model line (client/README.md, 3.5, The try).
pub const WELCOME_STATUS_WHY: Style = Style { owner: owner(), token: 516 };
/// Ready’s model controls (client/README.md, 3.5, The try).
pub const WELCOME_STATUS_ACTS: Style = Style { owner: owner(), token: 517 };

#[cfg(test)]
mod tests {
    #[test]
    fn homes_facts_and_byline_keep_their_own_style_names() {
        use super::{HOME_BYLINE, HOME_FACTS, NAMES};
        assert_eq!(NAMES.style(HOME_FACTS), Some(b"temper-home-facts".as_slice()));
        assert_eq!(NAMES.style(HOME_BYLINE), Some(b"temper-home-byline".as_slice()));
    }
}
