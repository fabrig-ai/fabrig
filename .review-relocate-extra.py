from pathlib import Path
import re,json,posixpath
R='temper/docs/design/'
groups={'domain/agent/README.md': {'ask-temper.md': ['Ask temper before M3b'], 'agents.md': ['Ask temper changing the catalogue', 'A real model asking its person', 'A real model revising a proposal', 'A goal as the designer'], 'tools.md': ['A real chat choosing `change`']}, 'domain/forge/README.md': {'change/branch-path.md': ["A closer's stops on a goal's path", "A close-out's own charter"], 'change/gates.md': ['Project-scoped rules on a real forge'], 'change/repairs.md': ["A clean update's other words"], 'keeping-up.md': ['A resource refused as forbidden'], 'change/README.md': ["A forge's run on a head replaced", 'An early update against the bound']}, 'protocol/configuration/README.md': {'stock.md': ["Ask temper's real-model walk"], '../../client/pages/README.md': ['A page refused busy'], '../forge/engine.md': ['A kept GitHub forge whose base ends']}, 'protocol/setup/README.md': {'init.md': ['Four agents on a laptop', 'Discovering models per account', 'The LLM providers out of the document', "Good defaults for a person's real work", "A task's text past what the clients receive"], 'owned.md': ["A handover made by a Forgejo's own restart"], 'box.md': ['A launchd agent for the box'], 'reconcile.md': ['Two deployments on one managed forge'], '../../client/forge/served.md': ["A Forgejo's own links at its `web`"]}}
files={};moves=[]
for src, destinations in groups.items():
 p=Path(R+src); s=p.read_text(); files[p]=s
 # Top-level list item, preserving every nested line up to the next top-level item/heading.
 matches=list(re.finditer(r'(?m)^- \*\*',s))
 for dest,labels in destinations.items():
  target=Path(posixpath.normpath(str(p.parent/dest)))
  if target not in files:files[target]=target.read_text()
  nums=[int(n) for n in re.findall(r'(?m)^## (\d+)\.',files[target])]
  oq=re.search(r'(?m)^## (\d+)\. (?:Open questions|Later)\s*$',files[target])
  num=int(oq[1]) if oq else max(nums,default=0)+1
  if not oq:files[target]=files[target].rstrip()+f'\n\n## {num}. Open questions\n'
  for label in labels:
   hits=[m for m in matches if s[m.start()+4:].startswith(label)]
   if len(hits)!=1:raise ValueError((src,label,len(hits)))
   m=hits[0]; end=re.search(r'(?m)^(?:- \*\*|#{1,6} )',s[m.end():]); e=m.end()+end.start() if end else len(s)
   item=s[m.start():e].rstrip()
   moves.append({'source':str(p),'label':item[4:].split('**')[0],'target':str(target),'section':num,'old':item,'text':item})
# Fix path-bearing citations when crossing into change/, and root-relative own section references.
# Rebase only inline code spans naming a .md path; paths in prose below handled explicitly.
for move in moves:
 src=Path(move['source']);target=Path(move['target']);text=move['text']
 if src.parent!=target.parent:
  def rebase(m):
   path=m[1]
   if path.startswith(('temper/','skein/','jig/','smith/','hearth/')) or not (src.parent/path).exists():return m[0]
   # Every existing relative path remains the same cited document.
   resolved=posixpath.normpath(str(src.parent/path))
   return '`'+posixpath.relpath(resolved,str(target.parent))+'`'
  text=re.sub(r'`([^`\n]+\.md)`',rebase,text)
 # Only the naked own-section references these moved items contain; rebind to original doc.
 own=posixpath.relpath(str(src),str(target.parent))
 if move['source'].endswith('domain/agent/README.md'):
  text=text.replace(' (6):',' (`README.md`, 6):')
 if move['source'].endswith('domain/forge/README.md'):
  text=text.replace('(11):','(`'+own+'`, 11):')
 if move['source'].endswith('protocol/configuration/README.md'):
  text=text.replace('(section 4)', '(`README.md`, section 4)').replace('(section 2)','(`README.md`, section 2)')
 if move['source'].endswith('protocol/setup/README.md'):
  text=text.replace('(section 2)', '(`README.md`, section 2)')
 if move['source'].endswith('client/web/README.md'):
  # The original bare section-2.3 reference is to the browser's addresses.
  text=text.replace('(`README.md`, 2.3,', '(`README.md`, 2.3,')
 move['text']=text
 files[target]=files[target].rstrip()+'\n\n'+text+'\n'
 # A source entry becomes a citation, with the label not a repeated question.
 rel=posixpath.relpath(str(target),str(src.parent))
 replacement='- '+move['label'].rstrip('.:')+f': `{rel}`, section {move["section"]}.'
 if files[src].count(move['old'])!=1:raise ValueError(('item not unique',move))
 files[src]=files[src].replace(move['old'],replacement)
for p,s in files.items():
 if len(s.encode())>65536:raise ValueError(('oversize',str(p),len(s.encode())))
for p,s in files.items():p.write_text(s)
Path('.review-moved.json').write_text(json.dumps(json.loads(Path('.review-moved.json').read_text())+moves,indent=2)+'\n')
print(len(moves),'items relocated;',len(files),'files;',max((len(s.encode()),str(p)) for p,s in files.items()))
