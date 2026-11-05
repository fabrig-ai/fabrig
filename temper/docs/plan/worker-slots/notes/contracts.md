# worker-slots: the shapes that cross tasks

Binding for `evict-keeps` and `restart-keeps`. The reasons are in `temper/docs/plan/worker-slots/plan.md`.

## The switch

Set after construction and before the first iteration, so that no constructor or literal changes and an unset deployment runs the code it runs today.

```rust
// temper-worker-domain-checkout, Domain, and its Cache (a `reuse: bool`, false from with_capacity)
pub fn reuse_workspaces(&mut self);
#[must_use] pub const fn reuses_workspaces(&self) -> bool;

// temper-worker-domain, Domain: forwards to its checkout
pub fn reuse_workspaces(&mut self);

// temper-service: Service forwards to its worker, if it has one; Worker forwards to its root
pub fn reuse_workspaces(&mut self);          // on Service
pub(crate) fn reuse_workspaces(&mut self);   // on Worker

// temper (the shell), beside private_root in lib.rs, called by both start paths once the service is built
pub(crate) fn workspace_reuse() -> bool { std::env::var_os("TEMPER_WORKSPACE_REUSE").is_some() }
```

## How a prepare finds its workspace

`temper_worker_domain_checkout::Cached` gains, and nothing else about it changes:

```rust
/// The least recently used workspace no client held, another workstream's,
/// holding exactly the spec's repositories: it keeps them, and is fetched and
/// checked out afresh.
Kept,
```

`restart-keeps` adds `Cached::Looking`, told by a workspace's first hold with the switch on and no repository starting saved.

## The story's world

`temper_worker_checkout_world`:

```rust
pub struct Settings { /* ... */ pub reuse: bool }   // false in Settings::calm
pub struct Told { /* ... */ pub kept: u32, pub makes: u32, pub clones: u32 }   // makes and clones count Fact::Started by Kind, beside started
impl World {
    /// Another party advances `branch` of the repository at `place`, at `at`.
    pub fn advance_at(&mut self, at: Time, place: usize, branch: &[u8]);
    /// The paths, outside the git directories, that this client's check-outs of
    /// `repository` changed: those whose content differs before and after, a
    /// path absent on one side included. So a workspace made empty shows every
    /// file, and one kept shows only what changed.
    pub fn touched(&self, client: u64, repository: &[u8]) -> BTreeSet<Vec<u8>>;
}
```

`restart-keeps` adds `World::restart` and `World::write_disk`, below.

## The restart (`restart-keeps`)

```rust
// temper_worker_domain_checkout::git
Op::Look { at: Place, remote: Box<[u8]> }   // Whether the repository's directory holds a clone of remote. Local and read-only. Ends in Looked.
Done::Looked { holds: bool }
Kind::Look
// temper_worker_domain_checkout (facts)
Ending::Looked { holds: bool }
Cached::Looking
// temper_worker_protocol_git
invocation::Command::Origin { directory: Box<[u8]> }  // workspaces/<token>/<repository>
// argv after the settings: --git-dir=<directory>/.git config --local --get remote.origin.url
impl Component { pub fn keep_workspaces(&mut self); pub const fn keeps_workspaces(&self) -> bool; }  // before its start
// temper_worker_checkout_world
impl World {
    /// The worker stops and starts: a new domain over the same disk and forge,
    /// no client holding anything. Unset, `ws` goes, as a start sets `workspaces` aside.
    pub fn restart(&mut self);
    pub fn write_disk(&mut self, path: &[u8], content: &[u8]);
}
```
