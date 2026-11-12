# Dependencies

Provisional, 2026-11-12, from `../../draft/follows.md`, section 7. A
project may say it builds on another project of the same deployment:
"fabrig depends on vsr-rs". The relation is a setting of the dependent's,
read beside its follows so that a person sees the chain, vsr followed by
vsr-rs, on which fabrig depends. Nothing acts on it: a landing in the
dependency changes nothing in the dependent. What is still open is
section 6; what waits, section 7.

## 1. In one page

- **A dependency is a project's setting,** kind 13 (section 2): the
  number of another project of the deployment, which the dependent's
  owners propose and accept as every setting.
- **Judged as any kind is,** by the forge's structure, then by the root:
  the project it names must be another project of the deployment.
- **Nothing automatic** (section 3): no task, change, wake or watched
  branch comes of it. A role of the dependent reads the dependency's
  repositories only through its own context adoption of them, and
  decides as it would on anything it reads.
- **Shown with the follows** (section 4): the forge's `follows` document
  carries the project's dependencies beside its follows, and `temper ctl
  follow list` prints a line each, "dependency 1: fabrig depends on
  vsr-rs". The document names no project: a reader names the dependency
  from its own projects.

## 2. A dependency

A dependency is a setting of the dependent's project, temper's kind 13
(`../../protocol/configuration/settings.md`, 2), at the project's scope,
numbered from 1 to the forge's `dependencies`
(`../../protocol/configuration/limits.md`), proposed and accepted as
every setting is, its decision that project's owners'. Its value, a
`Dependency`, holds one member:

| Member | What it holds | Absent or empty |
|---|---|---|
| `project` | the project depended on, by its number: a project of the deployment other than the dependent | refused |

- **Its form in a settings file,** binding for `temper ctl` and the
  host tools' settings (`../../protocol/configuration/settings.md`, 2),
  named `dependency`, with the dependent's `project` and its `number`:
  ```json
  {"setting": {"kind": "dependency", "project": 3, "number": 1}, "set": {"project": 7}}
  ```
  Read back, by `see settings dependency 1` and `temper ctl settings`,
  as `{"project": 7}`. A proposal's card names it `dependency 1`.
- **Judged as any kind is:** the forge's structural judgement first, then
  the root's checks across domains (`change/gates.md`, 3), each refusal
  naming the dependency and what fails, and the whole change refused
  with it:
  - at scope 0, for its scope; a number 0 or past `dependencies`, for its
    number; a value not a `Dependency`, or one naming project 0, for its
    value, in the forge's words for each, as kind 12's are;
  - a project the deployment doesn't hold, or one being deleted:
    `dependency N: project Q is not a project of this deployment`;
  - the dependent itself: `dependency N: project P cannot depend on
    itself`;
  - past `dependencies` of the project, by jig's count of a kind's
    numbers at one scope;
  - a removal or a reset always good.
- **Not refused:** a second dependency naming the same project, since
  nothing acts on either and the reads show both; and a cycle, P on Q
  and Q on P, since each project's settings are its own, judged alone,
  and nothing walks the relation.
- **Read** by a party with a role in the dependent's project, as kinds 6
  to 8 and 12 are. The stock holds none.
- **Kept** in family 20, `Setting`, as the forge's other kinds are, its
  value a `Dependency` in temper's store and client codecs. No other row
  is kept for it.
- **A project's removal** erases its own dependencies with its other
  settings. A dependency on a project since removed stays, and is read
  as a number (section 4): a project's number is never given again, so
  it names nothing else, and the dependent's owners remove it.

## 3. What it does

- **Nothing automatic in v0.1.0.** A landing in the dependency's
  repositories makes nothing in the dependent: no task, no change, no
  wake and no subscription. The forge watches no branch for a
  dependency (`keeping-up.md`, 1).
- **No adoption comes with it.** A dependency neither makes nor needs a
  context adoption (`adoption.md`). For its runs to read the
  dependency's code, the dependent adopts the dependency's repositories
  as context, as any project may (`follows.md`, section 8), and a role
  of the dependent reading them decides what to do as it would on
  anything it reads.
- **The dependency's project is told nothing** and gives nothing, as a
  follow's source isn't (`follows.md`, section 8).

## 4. Where it shows

- **In the forge's `follows` document** (`follows.md`, section 7): beside
  the project's follows, its dependencies, each its number and the
  number of the project it names, in number order, at most
  `dependencies`. The document is the project's, read by its members.
- **Named by the reader.** The document carries no project's name. A
  client names the dependency from the projects its reader's `me`
  document lists (jig's `client/domain/contract/documents.md`, 1), and
  otherwise as `project Q`. So a dependency tells no member of the
  dependent the name of a project they aren't a member of, and a
  dependency on a removed project reads as `project Q`.
- **`temper ctl follow list --project P`** prints, after the follows'
  lines, or after `no follows` where the project has none, a line a
  dependency (binding):
  `dependency N: PROJECT depends on DEPENDENCY`, PROJECT the project's
  name and DEPENDENCY the named project's where the reader is its
  member, else `project Q`: "dependency 1: fabrig depends on vsr-rs".
  With `--json`, the panel's object carries `dependencies`, a list of
  `{"number": N, "project": Q}` (`../../client/ctl/tools.md`, 2).
- **On Home,** What follows what draws each dependency between the
  person's projects beside the follows, from this document
  (`../../draft/surfaces.md`, section 3).

## 5. The world

Focused stories, in temper's domain world (`../../testing/README.md`,
section 4) with two projects on the fake forge:

- an owner's dependency proposal is accepted, kept, read by `see
  settings dependency 1`, and read the same after a restart;
- one naming its own project, and one naming a number the deployment
  holds no project at, are refused with section 2's words; one past
  `dependencies` is refused;
- a landing in the dependency's repository makes no task and no change
  in the dependent, and the forge watches no branch for it;
- the dependent's `follows` document carries the dependency, and
  `temper ctl follow list` prints its line, naming the dependency for a
  reader who is its member and `project Q` for one who isn't;
- the dependent's removal erases its dependencies; the dependency's
  removal leaves the dependent's, read as `project Q`.

No fuzzy story: a dependency is a setting no state follows, its
judgement a pure function of the deployment's projects.

## 6. Open questions

- **Words with it:** whether a dependency keeps a line of why, "which
  will replicate skein-kv", as a charter's words are kept
  (`../../protocol/configuration/settings.md`, 2, kind 9).

## 7. Later

- **A dependency bump** (`../../draft/follows.md`, section 7): a follow
  whose change moves the dependent to the dependency's newest landing,
  checked by the dependent's own gates.
- **A dependency on another deployment's project,** read through its
  public reads alone, as a follow's source there is later
  (`follows.md`, section 13).
