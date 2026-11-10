from pathlib import Path
import re
for root in ['skein','temper']:
 for p in Path(root).rglob('*.rs'):
  s=p.read_text(); changes=[]
  for m in re.finditer(r'(?P<full>(?:(?:\w+)::)*(?:Read::Item|Read::Comments|Request::Comments))\s*\{',s):
   a=m.end();i=a;depth=1
   while depth:
    if s[i]=='{':depth+=1
    if s[i]=='}':depth-=1
    i+=1
   fields=s[a:i-1]
   if '..' in fields or re.search(r'\bsince\b',fields): continue
   full=m['full'];path=str(p)
   pattern=s[i:].lstrip().startswith('=>') or s[i:].lstrip().startswith('if page')
   value='None' if 'Comments' in full else 'skein_lib::Time::ZERO'
   if 'Read::Item' in full and ('skein-fake-forge-domain' in path or 'skein-fake-forge-forgejo' in path or full.startswith(('a::','raw::','da::'))):value='None'
   if pattern:value='_'
   sep=' ' if fields.rstrip().endswith(',') else ', '
   changes.append((i-1,sep+'since: '+value+' '))
  for i,x in reversed(changes):s=s[:i]+x+s[i:]
  if changes:p.write_text(s)
# declarations
p=Path('skein/crates/skein-forge/src/api.rs');s=p.read_text().replace('    /// Read item.\n    Item { number: u64, after: u64 },','''    /// An item and its comments with ids above `after`.
    /// `since` is at or before the update of every comment above `after`;
    /// the dialect sends it as the forge's filter. Zero sends no filter.
    /// `after` stays the answer's check; an owner knowing no time sends zero.
    Item { number: u64, after: u64, since: Time },''');p.write_text(s)
p=Path('skein/crates/skein-forge/src/github/plans/reads.rs');s=p.read_text().replace('Item { number: u64, after: u64 },','Item { number: u64, after: u64, since: skein_lib::Time },').replace('api::Read::Item { number, after , since: _ } => Read::Item { number, after , since: skein_lib::Time::ZERO }','api::Read::Item { number, after, since } => Read::Item { number, after, since }');p.write_text(s)
for path in ['skein/crates/skein-forge/src/forgejo/request.rs','skein/crates/skein-forge/src/github/request.rs']:
 p=Path(path);s=p.read_text();a=s.index('    Comments {');b=s.index('\n    },',a);s=s[:b]+'\n        since: Option<Wall>,'+s[b:];p.write_text(s)
p=Path('skein/crates/skein-fake-forge-domain/src/api.rs');s=p.read_text()
for name in ['Item','Comments']:
 a=s.index('    '+name+' {');b=s.index('\n    },',a);s=s[:b]+'\n        since: Option<Time>,'+s[b:]
p.write_text(s)
# translator fields must forward
for path in ['skein/tests/forge/client/src/translate.rs','temper/tests/engine/forge/src/translate.rs']:
 p=Path(path);s=p.read_text().replace('client::Read::Item { number, after , since: _ }','client::Read::Item { number, after, since }').replace('forge::Read::Item { number: *number, after: *after , since: skein_lib::Time::ZERO }','forge::Read::Item { number: *number, after: *after, since: Some(*since) }');p.write_text(s)
