"""Exact ordered source preservation, with only 11 named payload layouts normalized."""
import json,pathlib,re,subprocess,hashlib
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/query-api-documentation')
BASELINE='e30eb49f77565769c0d6fdb61dbe372f686e77d7'
PATHS=['crates/nms-query/src/display.rs', 'crates/nms-query/src/find.rs', 'crates/nms-query/src/lib.rs', 'crates/nms-query/src/route.rs', 'crates/nms-query/src/show.rs', 'crates/nms-query/src/stats.rs', 'crates/nms-query/src/table.rs', 'crates/nms-query/src/theme.rs']
LAYOUTS={'crates/nms-query/src/find.rs': [b'Base(String),', b'Address(GalacticAddress),'], 'crates/nms-query/src/route.rs': [b'Biome(BiomeFilter),', b'Named(Vec<String>),', b'SystemIds(Vec<SystemId>),', b'Base(String),', b'Address(GalacticAddress),'], 'crates/nms-query/src/show.rs': [b'System(String),', b'Base(String),', b'System(ShowSystemResult),', b'Base(ShowBaseResult),']}
def git(*args):return subprocess.check_output(['git',*args],cwd=SOURCE)
def non_doc(data):return b''.join(x for x in data.splitlines(keepends=True) if x.strip() and not x.lstrip().startswith((b'///',b'//!')))
def normalized(data,rel):
 data=non_doc(data)
 for declaration in LAYOUTS.get(rel,[]):
  tokens=re.findall(rb'[A-Za-z_][A-Za-z_0-9]*|::|[^\s]',declaration)
  closing=max(i for i,t in enumerate(tokens) if t in (b'}',b')'))
  pieces=[re.escape(t) for t in tokens];pieces[closing]=rb'(?:,\s*)?'+pieces[closing]
  pattern=rb'\s*'.join(pieces)
  data,count=re.subn(pattern,lambda _:declaration,data)
  assert count==1,(rel,declaration,count)
 return data
def compare(data,baseline,rel):return normalized(data,rel)==normalized(baseline,rel)
if __name__=='__main__':
 changed=set(git('diff','--name-only',BASELINE).decode().splitlines())
 assert changed and changed.issubset(PATHS),(changed,PATHS)
 assert not git('ls-files','--others','--exclude-standard','-z'), 'unexpected untracked source'
 for rel in PATHS:assert compare((SOURCE/rel).read_bytes(),git('show',BASELINE+':'+rel),rel),rel
 rel='crates/nms-query/src/stats.rs';body=(SOURCE/rel).read_bytes();old=git('show',BASELINE+':'+rel)
 anchor=b'        system_count: model.system_count(),';assert body.count(anchor)==1
 assert not compare(body.replace(anchor,b'        system_count: 0,',1),old,rel)
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'changed_paths':sorted(changed),'all_8_ordered_non_doc_comparisons':True,'permitted_named_layout_sites':sum(map(len,LAYOUTS.values())),'opposed_in_memory_executable_mutation_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
