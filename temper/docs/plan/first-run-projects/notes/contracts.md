# first-run-projects: contracts

The run's binding calls on what crosses its parts. A task cites the entries it builds to; the
design sections its spec cites carry the same, in the design's words.

## K1. Ready makes "Default" (the draft's 4)

- **Who sends it:** `temper-client-domain`'s welcome, once a real forge is connected and a model
  is usable (the welcome's decision, K2, at `Step::Ready`), and only on the person's press of
  Ready's one act. It sends jig's `c::Ask::MakeProject { name: "Default", policy: 1 }` (policy 1
  the stock's, `temper/docs/design/protocol/configuration.md`, 13.4) in the deployment's scope,
  under a key of the welcome's own, one per attempt (the purpose `welcome-default`, the attempt's
  number), never the forge's `start_project_key`.
- **Answered `ProjectMade { project }`:** the welcome opens it as `pages::Act::OpenProject`
  does: the project chosen once `me` lists it, and `/chats` shown.
- **Refused as the name taken:** for now (the draft's status, **Open**, 9), the welcome opens the
  person's project named `Default` from `me` as above, and makes no other name. If `me` lists no
  such project the person holds, it is refused like any other.
- **Any other refusal, or no answer:** nothing is made (one act); Ready says why and offers its act
  again, a new attempt under a new key.
- **One at a time:** while an attempt is out, Ready's act is disabled.

## K2. When the welcome is whole (the draft's 4)

`temper-client-domain`'s `pages/welcome.rs` `decision`: `Whole` once a real forge (any kept
forge but the playground's, not ended) is connected (its row's connection `Connected`, and on
GitHub an accepted installation, `forges::accepted`), a model is usable (`model_usable`, as now)
and `me` lists a project the person holds. No adoption is read. Otherwise the first step not
done: Forge (no real forge connected, or one unfinished), Model (no usable model), Ready (no
project).

## K3. The request's project goes (the draft's 4)

The Forge step names no project, and nothing in the engine reads the one it carries: the client
wire's `SetUp` is `{ forge, org, repository, playground }` and `FirstAsk` `{ org, repository,
label }`; the forge row's `FirstRepository` is `{ org, repository, state, label, playground }`;
`SetUpProject`, `ExistingProject`, `NewProject` and `SetUpRefusal::new_project` go. The store's
`FirstIntent` is `{ playground, org, repository, label }`, and `FirstProject`, `ExistingFirst` and
`NewFirst` go. Both schemas change in place, at their current versions. `temper ctl forge setup`
and `forge connect … --first` keep `--project`: ctl holds it in its own run, as it already makes
the project and adopts after the row says made (`temper/docs/design/client/tools.md`, 5.5).

## K4. A project with no repository (the draft's 5)

A project *has no repository* while the `forges` watch has delivered its rows and no row's
`repositories` names it; with the watch refused, ended or not yet delivered, nothing is said. The
playground's project, while the playground is read, shows its context line instead. Its chats page
shows, where its repository would show, that it has none, with the link "Adopt a repository" to
`Address::Adopt { project }` (`/projects/N/adopt`). The look of that place is the look plan's
(`notes/common.md`); its words and its link are this plan's.
