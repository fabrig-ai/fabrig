from pathlib import Path
import re, json, posixpath
R='temper/docs/design/'
# Exact item prefixes; destinations keep each item's words, not a merged answer.
groups={
'domain/agent/README.md': {
 'tools.md': ['Proposal-form diagnostics','A charter where the process has no local agent','A collaborator\'s login','`read_permission`\'s login','Which host tools a charter offers','Who may name a plan\'s members','A change\'s foremen\'s LLM choice','A cancel\'s grace','A change produced by a job'],
 'agents.md': ['A permission for its own requester alone','Constraining models in settings','The person\'s forge identity in a chat\'s brief','A person known by a login alone','What sizes temper\'s own way','QA in temper\'s own way','Ask temper woken by a landing','A goal\'s question kept with its chat','Who else writes a question\'s line form'],
 'charter.md': ['A direct change result\'s rendering room'],
 'ask-temper.md': ['A `see` answer by its worker\'s reply bound','A document moved within one call','Parts of a settings proposal across a restart','How the settings\' addresses read','A revision\'s values','A project\'s own policy through `see`'],
},
'domain/forge/README.md': {
 'adoption.md': ['A workflow that runs on no pull request','What a refused copy leaves','A merge style per repository','A change whose forge is not connected','`Forbidden` and `Missing` taken','Signed commits required','Connecting a forge as another user'],
 'branches.md': ['Saved branches outside the task\'s saved set'],
 'keeping-up.md': ['The connector\'s own retries','Which checks are CI on GitHub'],
 'attention.md': ['"The brief on Home" in Settings','A status run\'s kept row','A stock project\'s room, shared','What lands on a goal\'s branch','What ends the Away switch','The classes per person','What code decides, listed'],
 'queues.md': ['A queue\'s throughput','A repair of the branch\'s own CI','Several changes in a forge\'s merge queue','The update before entering a forge\'s merge queue','Taking a change out of a forge\'s merge queue'],
 'change/repairs.md': ['A gate\'s live run past the stall','A green head no wake reaches','A retry before the read','A hold at gating at a green head','A repair held for its brief','A forge out of reach or not connected','A failing check\'s repair brief','What failed on a queue\'s group'],
 'change/questions.md': ['A refused `message` keeps no record','A passed-up question whose asker ends','A report\'s head past a merge'],
},
'protocol/configuration/README.md': {
 'stock.md': ['A landing branch with another name than `main`','A chat checking the forge before declaring'],
 'reference.md': ['The reference\'s later runs','The checks\' deadline','Who writes the case\'s workflows'],
 'settings.md': ['Settings refusal positions','A reference call of 128 KB','Large arguments through a real program','A real provider\'s list','A Claude-only box on the small profile','A message, a report or a person\'s spec','A brief past the host part\'s room','Requirements an application fixes'],
 'limits.md': ['The real-browser check of watch sizing','A party\'s fair share of the watch rooms','`views.`\'s other names','Profiles\' numbers','The rest of an assignment\'s transcript'],
 'directories.md': ['A configuration directory moved or copied'],
},
'protocol/setup/README.md': {
 'box.md': ['Repeated admission during fake-box preparation','A box killed outright on macOS','A box started in the background of a terminal','The terminal after a front killed outright','The fake box\'s retained ports','A fake fetch for the fake box','The other fake peers\' refused connections','The time of day on the terminal\'s error lines'],
 'init.md': ['A start holding the engine\'s documents','The deployment\'s own UX','Flags of their own'],
 'owned.md': ['An owned forge made managed','A lock and an exec in skein\'s shell','A copy of a bottle of temper\'s own','The person\'s `claude` installed after init','Reading a runner\'s row','Overrides of `app.ini`','Backups beyond an upgrade\'s'],
 'reconcile.md': ['A story of Reconcile over TLS','A secret drawn at a reconcile','An adopted row\'s kinds after the forge changes','A managed forge whose host list refuses'],
},
'client/forge/README.md': {
 'deployment.md': ['A repository link in a folded forge row','A secret text area for the pasted key','An App of one\'s own, offered again','A connect\'s stop told','A browser on another machine','Mending a stopped managed or owned forge'],
 'repositories.md': ['The picker\'s pages kept'],
},
'client/forge/family.md': {
 'repositories.md': ['A repository\'s role, health and landing rules'],
 'deployment.md': ['A connect\'s end told'],
},
'client/web/README.md': {
 'testing.md': ['A filter row while its project\'s ways arrive','A phone\'s width in a test','A chat\'s way to the Closed filter','The box\'s pace in the tier','A wait\'s CPU in the tier','The remaining browser cost','Merged stories after landing','Reading\'s test past its run\'s forty seconds','A long chat at the person\'s whole bound','A public page\'s plain text in one query','Every wrapped text fragment','A public page changing during measurement','Expected browser errors scoped to an act','A compile cache across the tier\'s tests'],
},
}
files={};moves=[]
for src, destinations in groups.items():
 p=Path(R+src); s=p.read_text(); files[p]=s
 # Top-level list item, preserving every nested line up to the next top-level item/heading.
 matches=list(re.finditer(r'(?m)^- \*\*',s))
 for dest,labels in destinations.items():
  target=p.parent/dest
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
   if path.startswith(('temper/','skein/','jig/','smith/','hearth/')):return m[0]
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
Path('.review-moved.json').write_text(json.dumps(moves,indent=2)+'\n')
print(len(moves),'items relocated;',len(files),'files;',max((len(s.encode()),str(p)) for p,s in files.items()))
