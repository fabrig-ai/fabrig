from pathlib import Path
import re
b=Path('temper/crates/temper-service/src'); c=Path('temper/crates/temper-service-core/src')
p=b/'limits.rs'; s=p.read_text(); checkout=s[s.index('const CHECKOUT:'):s.index('/// Root room')]; git=s[s.index('const GIT:'):s.index('/// Limits decoded')]; ns=(b/'limits/names.rs').read_text()
shared=['forge.forges','forges.installations','forge_component.token_bytes']+re.findall(r'b"((?:checkout|git_component)\.[^"]+)" =>',ns[:ns.index('pub enum Unit')])
names=re.findall(r'name: "([^"]+)"',ns)
for file in ['temper/crates/temper-box-machine/src/limits.rs','temper/crates/temper-forge-setup/src/limits.rs']:
 for n in re.findall(r'name: "([^"]+)"',Path(file).read_text()):
  if n not in names:names.append(n)
pairs={'(':')','{':'}','[':']'}
def bal(text,i):
 stack=[pairs[text[i]]];i+=1;quote=False
 while stack:
  ch=text[i]
  if quote:
   if ch=='\\':i+=2;continue
   if ch=='"':quote=False
  elif ch=='"':quote=True
  elif ch in pairs:stack.append(pairs[ch])
  elif ch==stack[-1]:stack.pop()
  i+=1
 return i
x=c/'limits.rs';text=x.read_text()+'\n'+checkout+git+'''
/// Shared profile parts and peer scalars (protocol/services.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Profile { Small, Standard }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Peer { pub forges: u32, pub installations: u32, pub token_bytes: u32 }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shared { pub checkout: Checkout, pub git_component: GitComponent, pub peer: Peer }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused { Unknown, Range }
pub const FORGE_INSTALLATIONS: u32 = 8;
pub const FORGE_TOKEN_BYTES: u32 = 512;
#[must_use]
pub const fn profile(profile: Profile) -> Shared {
 match profile { Profile::Small | Profile::Standard => Shared { checkout: CHECKOUT, git_component: GIT,
 peer: Peer { forges: 3, installations: FORGE_INSTALLATIONS, token_bytes: FORGE_TOKEN_BYTES } } }
}
/// Every override known to a document, including the other owners' names.
pub const NAMES: &[&str] = &[
'''+''.join(f' "{n}",\n' for n in names)+'];\npub const SHARED_NAMES: &[&str] = &[\n'+''.join(f' "{n}",\n' for n in shared)+'''];
fn duration(value: u64) -> Result<Duration, Refused> {
 match value.checked_mul(1_000_000) { Some(nanos) => Ok(Duration::from_nanos(nanos)), None => Err(Refused::Range) }
}
/// Apply shared overrides; another known name's integer is taken unread.
pub fn apply(limits: &mut Shared, name: &[u8], value: u64) -> Result<(), Refused> {
 match name {
'''
for n in shared:
 start=ns.index(f'        b"{n}" =>');end=ns.find('\n        b"',start+1)
 arm=ns[start:end].replace('limits.forge.forges','limits.peer.forges').replace('limits.forges.installations','limits.peer.installations').replace('limits.forge_component.token_bytes','limits.peer.token_bytes');text+=arm+'\n'
text+=''' _ => if !known(name) { return Err(Refused::Unknown); },
 }
 Ok(())
}
#[must_use]
pub fn known(name: &[u8]) -> bool { for known in NAMES { if known.as_bytes() == name { return true; } } false }
#[must_use]
pub fn shared_name(name: &[u8]) -> bool { for known in SHARED_NAMES { if known.as_bytes() == name { return true; } } false }
''';x.write_text(text)
s=s.replace(checkout,'').replace(git,'');s=re.sub(r'/// Temper\x27s profiles.*?\n}\n','',s,flags=re.S);s=re.sub(r'/// Refusal of an override.*?const CLIENT:', 'pub use temper_service_core::limits::{Profile, Refused, FORGE_INSTALLATIONS, FORGE_TOKEN_BYTES};\nconst CLIENT:',s,flags=re.S)
s=s.replace('    match profile {\n        Profile::Standard => Limits { root: STANDARD_ROOT, forge: STANDARD_FORGE, ..self::profile(Profile::Small) },','    let shared = temper_service_core::limits::profile(profile);\n    let mut limits = match profile {\n        Profile::Standard => Limits { root: STANDARD_ROOT, forge: STANDARD_FORGE, ..self::profile(Profile::Small) },').replace('checkout: CHECKOUT,','checkout: shared.checkout,').replace('git_component: GIT,','git_component: shared.git_component,')
pos=s.index('/// Set one dotted');s=s[:pos].rstrip().removesuffix('}').rstrip().removesuffix('}').rstrip()+''' };
 limits.checkout = shared.checkout;
 limits.git_component = shared.git_component;
 limits.forge.forges = shared.peer.forges;
 limits.forges.installations = shared.peer.installations;
 limits.forge_component.token_bytes = shared.peer.token_bytes;
 limits
}
'''+s[pos:]
s=s.replace('forges: 3,','forges: temper_service_core::limits::profile(Profile::Small).peer.forges,')
start=s.index('/// The worker root');end=s.index('#[cfg(test)]',start);s=s[:start]+'''/// The worker's declared kinds, also read by composition worlds (protocol/services.md, 6).
#[cfg(feature = "worker")]
pub use crate::worker_settings::job_kinds;
/// Project the engine's resolved aggregate for the local worker (protocol/services.md, 6).
#[cfg(feature = "worker")]
#[must_use]
pub fn worker(limits: &Limits) -> crate::worker_settings::Limits {
 crate::worker_settings::Limits { checkout: limits.checkout, git_component: limits.git_component,
 peer: temper_service_core::limits::Peer { forges: limits.forge.forges, installations: limits.forges.installations, token_bytes: limits.forge_component.token_bytes } }
}
'''+s[end:];p.write_text(s)
for n in shared:
 start=ns.index(f'        b"{n}" =>');end=ns.find('\n        b"',start+1);ns=ns[:start]+ns[end+1:]
ns=ns.replace('    if find(name).is_none() {','''    if temper_service_core::limits::shared_name(name) {
 let mut shared = temper_service_core::limits::Shared { checkout: limits.checkout, git_component: limits.git_component,
 peer: temper_service_core::limits::Peer { forges: limits.forge.forges, installations: limits.forges.installations, token_bytes: limits.forge_component.token_bytes } };
 temper_service_core::limits::apply(&mut shared, name, value)?;
 limits.checkout = shared.checkout; limits.git_component = shared.git_component;
 limits.forge.forges = shared.peer.forges; limits.forges.installations = shared.peer.installations; limits.forge_component.token_bytes = shared.peer.token_bytes;
 return Ok(());
 }
    if find(name).is_none() {''');(b/'limits/names.rs').write_text(ns)
# Vocabulary split. The shared generic sum supports concrete half diagnostics without callbacks.
p=b/'startup/problems.rs';s=p.read_text();engine=['Room','Stock','EngineAlone','UnbuiltWorker','Loopback','Connectors','OldAgentCommand','NoWebhooks','TierChanges','StorePage','StoreMap','StoreTask','Agent','Admission','Adapter','Redirect','NoHub','Credentials','Escalation','Provider','BoxSignIn','Handover','MissingNull','InlineSeedPolicy','LandingStored','ProjectLandingStored','CharterStored']
a=s.index('pub enum What {')+len('pub enum What {');end=bal(s,s.index('{',a-2))-1;body=s[a:end];rows=[]
for m in re.finditer(r'^    (\w+)(?=[({, ])',body,re.M):
 name=m.group(1);i=m.end()
 while body[i]==' ':i+=1
 if body[i] in '({':i=bal(body,i)
 if body[i]==',':i+=1
 rows.append((name,body[m.start():i]))
container=s[s.index('#[derive(Clone, PartialEq, Eq, Debug)]'):s.index('fn put(')].replace('pub struct Problem {','pub struct Problem<W> {').replace('pub what: What,','pub what: W,').replace('pub struct Problems {','pub struct Problems<W> {').replace('List<Problem>','List<Problem<W>>').replace('impl Problems {','impl<W> Problems<W> {').replace('what: What','what: W').replace('pub(crate) fn push_file','pub fn push_file')
i=container.index('    #[must_use]\n    pub fn jig');container=container[:i]+'}\nimpl<E> Problems<What<E>> {\n'+container[i:];container=container.replace('pub(crate) fn append','pub fn append')
shared='''//! Shared startup vocabulary, bounded problems and lines (protocol/services.md, 3).
use alloc::boxed::Box;
use jig_protocol_config as config;
use skein_lib::{Decimal, List, Writer};
use crate::limits;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What<E> {
'''+ '\n'.join(row for name,row in rows if name not in engine)+'\n Half(E),\n}\n'+container+'''fn words<E>(what: &What<E>) -> &'static [u8] {
 match what {
 What::Standalone => b"standalone takes one worker, joined to the engine by the in-process name \\\"workers\\\"",
 What::WorkerAlone => b"a worker alone takes one worker dialling a plaintext loopback socket, and only application.workers and application.limits",
 What::Absolute => b"must be an absolute path",
 What::Profile => b"the only profile is \\\"small\\\"",
 What::Override(limits::Refused::Unknown) => b"no limit of temper's has this name",
 What::Override(limits::Refused::Range) => b"outside its type",
 What::Against { .. } => b"must agree with ",
 What::Jig(what) => jig_words(*what),
 What::Checkout { .. } | What::Past { .. } | What::Within { .. } | What::Short { .. } | What::CallInput { .. } | What::Half(_) => b"",
 }
}
'''
shared+=s[s.index('fn put('):s.index('fn words(')].replace('fn put(', 'pub fn put(')
shared+=s[s.index('fn jig_words('):s.index('fn redirect_listener(')]
shared+=s[s.index('fn render_path('):s.index('fn store_page(')].replace('fn render_path(', 'pub fn render_path(')
shared+=s[s.index('fn operation_counts('):s.index('fn store_task(')]
checkout=s[s.index('/// A capacity'):s.index('#[expect(clippy::too_many_lines',s.index('/// A capacity'))];i=checkout.index('        What::Room(')
checkout=checkout[:i]+''' What::Jig(_) | What::Standalone | What::WorkerAlone | What::Absolute | What::Profile | What::Override(_) | What::Against { .. } | What::Past { .. } | What::Short { .. } | What::Within { .. } | What::CallInput { .. } | What::Half(_) => false,
 }
}
'''
checkout=checkout.replace('fn checkout(what: What,','fn checkout<E>(what: &What<E>,').replace('Decimal::of(u64::from(value))','Decimal::of(u64::from(*value))').replace('u64::from(has)','u64::from(*has)').replace('u64::from(needs)','u64::from(*needs)').replace('as_slice(), stdout_of,','as_slice(), *stdout_of,');shared+=checkout
render=s[s.index('        render_path(&problem.path'):s.index('    if problems.past > 0')].removesuffix('    }\n')
# extract engine and common arms using original syntax (including unit fallthrough separately)
original_render=render
for name in engine:
 pat=re.compile(r'(?m)^            What::'+name+r'(?!\w)')
 while (m:=pat.search(render)):
  arrow=render.index('=>',m.start());i=arrow+2
  while render[i].isspace():i+=1
  end=bal(render,i) if render[i]=='{' else render.index(',\n',i)+1
  render=render[:m.start()]+render[end:].lstrip('\n')
render=re.sub(r'(?m)^            \| What::(?:'+'|'.join(engine)+r')(?:\([^\n]*\)| \{ \.\. \})?\n','',render).replace('            | What::Handover => {}','            => {}').replace('            | What::Override(_)\n','            | What::Override(_) => {}\n').replace('            => {}\n','')
render=render.replace('        match problem.what {','        match &problem.what {\n What::Half(_) => unreachable!("half rendered by its service"),').replace('words(problem.what)','words(&problem.what)')
render='''#[expect(clippy::too_many_lines, reason = "one exhaustive shared vocabulary for both rendering passes")]
pub fn render_problem<E>(problem: &Problem<What<E>>, writer: &mut Writer, measuring: bool) -> usize {
 let mut length = 0;
 if checkout(&problem.what, writer, &mut length, measuring) { return length; }
'''+render+' length\n}\n'
for name in ['bytes','input','ask','reply','bound','needs']:render=render.replace('Decimal::of('+name+')','Decimal::of(*'+name+')')
render=render.replace('operation_counts(operations, delegates, goals,','operation_counts(*operations, *delegates, *goals,').replace('codec_words(what,','codec_words(*what,').replace('u64::from(charter)','u64::from(*charter)');shared+=render
shared+='''pub fn render_past(past: u32, writer: &mut Writer, length: &mut usize, measuring: bool) {
 if past > 0 { put(writer, length, measuring, b"and "); put(writer, length, measuring, Decimal::of(u64::from(past)).as_bytes()); put(writer, length, measuring, b" more\\n"); }
}
fn render<E>(problems: &Problems<What<E>>, writer: &mut Writer, measuring: bool) -> usize {
 let mut length: usize = 0;
 for problem in &problems.problems { length = length.checked_add(render_problem(problem, writer, measuring)).expect("bounded diagnostics"); }
 render_past(problems.past, writer, &mut length, measuring); length
}
#[must_use]
pub fn lines<E>(problems: &Problems<What<E>>) -> Box<[u8]> {
 let length = render(problems, &mut Writer::new(0), true); let mut writer = Writer::new(length);
 assert_eq!(render(problems, &mut writer, false), length, "diagnostic passes measure identically"); writer.finish()
}
/// A half with no additional startup problems (protocol/services.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SharedOnly {}
pub type SharedWhat = What<SharedOnly>;
pub type SharedProblems = Problems<SharedWhat>;
impl SharedWhat {
 #[must_use]
 pub fn extend<E>(self) -> What<E> {
 match self {
'''
fields={'Jig':'what','Standalone':'','WorkerAlone':'','Absolute':'','Profile':'','Override':'what','Against':'other','Checkout':'conflicts, path_bytes, repositories, stdout, stdout_of, agent_conflicts, agent_paths, saved, engine, worker','Past':'bytes, bound','Short':'bytes, of, needs','Within':'bytes, limit, bound','CallInput':'bytes, input, ask, reply'}
for name,args in fields.items():
 suffix='('+args+')' if name in ['Jig','Override'] else ' { '+args+' }' if args else '';shared+=f' What::{name}{suffix} => What::{name}{suffix},\n'
shared+=' What::Half(half) => match half {},\n }\n }\n}\n';(c/'startup').mkdir(exist_ok=True);(c/'startup/problems.rs').write_text(shared)
words=s[s.index('fn words('):s.index('fn jig_words(')]
new='''//! Engine startup problems around the core's vocabulary (protocol/services.md, 3).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};
use temper_service_core::startup::problems::{put, render_path, render_problem, render_past};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EngineWhat {
'''+ '\n'.join(row for name,row in rows if name in engine)+'''\n}
pub type What = temper_service_core::startup::problems::What<EngineWhat>;
pub type Problem = temper_service_core::startup::problems::Problem<What>;
pub type Problems = temper_service_core::startup::problems::Problems<What>;
fn words(what: EngineWhat) -> &'static [u8] {
 match what {
'''
blanks=[]
for name in engine:
 m=re.search(r'        What::'+name+r'(?!\w)(?:\([^\n]*?\))? => ([^\n]*),',words)
 if m:new+=m.group(0).replace('What::','EngineWhat::')+'\n'
 else:blanks.append('EngineWhat::'+name+('(_)' if name in ['Room','Stock','StoreMap'] else ' { .. }' if name in ['TierChanges','StorePage','StoreTask','Admission'] else ''))
new+=' | '.join(blanks)+' => b"",\n }\n}\n'
new+=s[s.index('fn redirect_listener('):s.index('fn render_path(')].replace('words(What::Agent)','words(EngineWhat::Agent)')
new+=s[s.index('fn store_page('):s.index('fn operation_counts(')]+s[s.index('fn store_task('):s.index('/// A capacity')]
new+='''fn render(problems: &Problems, writer: &mut Writer, measuring: bool) -> usize {
 let mut length = 0_usize;
 for problem in &problems.problems {
 let what = match problem.what {
 What::Half(what) => what,
'''
common=[]
for name,row in rows:
 if name in engine:continue
 common.append('What::'+name+('(_)' if name in ['Jig','Override'] else ' { .. }' if name in ['Against','Checkout','Past','Short','Within','CallInput'] else ''))
new+=' | '.join(common)+''' => { length = length.checked_add(render_problem(problem, writer, measuring)).expect("bounded diagnostics"); continue; }
 };
 render_path(&problem.path, writer, &mut length, measuring); put(writer, &mut length, measuring, b": ");
 if let Some(file) = &problem.file { put(writer, &mut length, measuring, file); put(writer, &mut length, measuring, b": "); }
 put(writer, &mut length, measuring, words(what));
 match what {
'''
blanks=[]
for name in engine:
 m=re.search(r'^            What::'+name+r'(?!\w)',original_render,re.M)
 if m:
  arrow=original_render.index('=>',m.start());i=arrow+2
  while original_render[i].isspace():i+=1
  if not '\n' in original_render[m.start():arrow]:
   end=bal(original_render,i) if original_render[i]=='{' else original_render.index(',\n',i)+1
   new+=original_render[m.start():end].replace('What::','EngineWhat::')+'\n';continue
 suffix='(_)' if name in ['Room','Stock','StoreMap','Redirect'] else ' { .. }' if name in ['TierChanges','StorePage','StoreTask','Admission'] else '';blanks.append('EngineWhat::'+name+suffix)
new+=' | '.join(blanks)+''' => {},
 }
 put(writer, &mut length, measuring, b"\\n");
 }
 render_past(problems.past, writer, &mut length, measuring); length
}
'''+s[s.index('#[must_use]\npub fn lines'):]+'''
pub(super) fn from_worker(found: temper_service_core::startup::problems::SharedProblems) -> Problems {
 let mut result = Problems::empty();
 for problem in found.problems.into_boxed() { result.push_file(problem.path, problem.what.extend(), problem.file); }
 result.past = result.past.saturating_add(found.past); result
}
''';p.write_text(new)
for file in b.rglob('*.rs'):
 if file==p or file==b/'startup/worker.rs':continue
 t=file.read_text();pattern=re.compile(r'(?<!\w)(?P<path>(?:crate::startup::|super::|startup::)?What)::(?P<name>'+ '|'.join(engine)+r')(?!\w)');pos=0;out=''
 for m in pattern.finditer(t):
  if m.start()<pos:continue
  i=m.end()
  if i<len(t) and t[i]=='(':end=bal(t,i)
  elif t[i:].lstrip().startswith('{'):end=bal(t,i+len(t[i:])-len(t[i:].lstrip()))
  else:end=i
  out+=t[pos:m.start()]+m.group('path')+'::Half(crate::startup::EngineWhat::'+m.group('name')+t[i:end]+')';pos=end
 if pos:file.write_text(out+t[pos:])
