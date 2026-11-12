# squash-in: a goal's branch takes each change as one commit

A goal works on its own branch: many changes, each made on a branch of its own, land into the goal's branch, and the goal's branch then lands into main through the queue ([merge-queue.md](merge-queue.md)). This draft says how work inside a goal's branch is kept: each change squashed into it as one commit, the branch never rewritten, main taken in only when a change needs it. The queue's `combine` (`merge-queue.md`, 5.1 and 10) is its mechanism; this draft is what follows for the goal, its agents and the history they leave.

## 1. In one page

- **Squash in, merge up:** a change lands into its goal's branch as one commit; the goal's branch lands into main as a merge (`merge-queue.md`, 5.3). Main's first-parent line reads one merge per goal, and each merge's second parent the goal's changes, one commit each.
- **The goal's branch is shared and only grows:** nothing rewrites it or pushes to it by force; changes start from its tip.
- **A change's commit** carries its title, a short body, and trailers naming the change, its pull request, its head and its goal (section 3).
- **Main comes in only when needed** (section 4): a conflict's resolution, or a change that needs what main gained; never as routine.
- **Agents stop curating history** (section 5): a change's own commits are its maker's, work in progress included; what the history keeps is the change's title and description.

## 2. The model

- **A goal's branch** is made from main's tip when the goal starts (`../domain/forge/branches.md`), and is the destination of its goal's changes: a train of its own (`merge-queue.md`, 3), one change at a time, combined by squash, the stock `combine` for a branch other than main.
- **Shared, append-only:** every change of the goal starts from its tip, several at once, so nothing may rewrite it. Its only writers are its train's turns (a squash commit at a time), a resolution's merge (section 4), and a catch-up's merge (section 4).
- **A change's branch** is its maker's: it commits as it likes, and may merge its goal's branch into it to stay current. None of that reaches the goal's branch but the change's tree.
- **A change is done when it lands:** its head is no ancestor of its squash commit, so its branch does not go on. A repair of a landed change, or a follow-up, is a new change from the goal's branch's tip.
- **The goal's branch goes on after it lands into main:** it merges, so it stays an ancestor of main, and its close-out, its repairs and its follow-ups land as more changes into it.

## 3. A change's commit

- **Subject:** the change's title, as its pull request carries it.
- **Body:** the change's description, its first paragraph, wrapped, bounded.
- **Trailers,** each once, in this order: `Change: <task>`, `Pull-request: <number>`, `Head: <commit>` (the head it was squashed from, which stays fetchable on the forge while its pull request lives), `Goal: <task>`.
- **Author:** as the deployment authors its agents' commits; the committer is temper.
- **The landing's merge into main** carries the goal's title as its subject, with the same trailers for the goal: its change, pull request and head (`merge-queue.md`, 5.1).

## 4. Main into a goal's branch

- **Only when a change needs it:**
  - **A resolution:** a goal's landing into main conflicts with main (`merge-queue.md`, 5.2): main's tip merged into the goal's branch, the conflict settled there, as `../domain/forge/change/repairs.md` says.
  - **A catch-up:** a change of the goal needs what main gained since the goal's branch was made. The goal asks for main's tip: a clean merge is made into its branch without an agent, a conflicting one becomes a resolution.
- **Never squashed:** a merge of main keeps main's commits main's. A change that brings main's commits into the goal's branch anyway is merged by its train, whatever `combine` says (`merge-queue.md`, 5.3).
- **Never routine:** the queue's merge of the goal's branch onto main's tip is where the two meet (`merge-queue.md`, 5.1); a goal doesn't take main to stay current.

## 5. What the agents do

- **A change's maker** (implementer or designer) commits on its own branch as it works, with no rule for its commits' messages or number; it never rewrites or pushes to its goal's branch; it writes its change's title and description for the history, since they become the commit (section 3).
- **A goal's coordinator** asks for main's tip only for a change that needs it (section 4), and may deliver at any point (`merge-queue.md`, 4.3); it cuts no change whose only work is a catch-up.
- **The stock charters say so** (`../domain/agent/charter.md`, 4), each in its role's words: the coordinator's, the designer's and the coding agent's.

## 6. History as it reads

- **`git log --first-parent main`:** one merge per landed goal, newest first.
- **Below each:** the goal's changes, one commit each, with their trailers; a resolution or a catch-up shows as a merge of main.
- **Blame and bisect** reach a change, then through its trailers its task, pull request and head.

## 7. Acceptance

Focused and fuzzy suites alone, each failing first:

- a change squashed into its goal's branch: one commit, single parent, the subject, body and trailers of section 3, the tree its merge would have;
- a change that brings main's commits merged, not squashed;
- a catch-up asked by a goal: a clean merge made without an agent, a conflicting one a resolution;
- a repair after a change landed starts from the goal's branch's tip;
- the stock charters' words for each role.

The process tier's queue story (`merge-queue.md`, 12) runs with goals whose changes squash in.

## 8. Later

- **A goal's branch read back as its changes,** in the client's goal view: each change's commit, its task and its pull request.
