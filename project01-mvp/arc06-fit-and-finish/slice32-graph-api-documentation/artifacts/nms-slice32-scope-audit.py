"""Exact ordered source preservation, with only 10 named payload layouts normalized."""
import json,pathlib,re,subprocess,hashlib
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/graph-api-documentation')
BASELINE='b60e720a1ba3095e403ab706efab7fc0d11469ed'
PATHS=['crates/nms-graph/src/edges.rs', 'crates/nms-graph/src/error.rs', 'crates/nms-graph/src/extract.rs', 'crates/nms-graph/src/import.rs', 'crates/nms-graph/src/lib.rs', 'crates/nms-graph/src/model.rs', 'crates/nms-graph/src/query.rs', 'crates/nms-graph/src/route.rs', 'crates/nms-graph/src/spatial.rs']
LAYOUTS={'crates/nms-graph/src/edges.rs': [b'Knn { k: usize },', b'WarpRange { max_ly: f64 },'], 'crates/nms-graph/src/error.rs': [b'SystemNotFound(String),', b'BaseNotFound(String),', b'NoSystemsInGalaxy(u8),', b'TargetNotFound(String),'], 'crates/nms-graph/src/import.rs': [b'Csv(#[from] csv::Error),', b'Io(#[from] std::io::Error),'], 'crates/nms-graph/src/route.rs': [b'SystemNotFound(SystemId),', b'NoPath { from: SystemId, to: SystemId },']}
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
 rel='crates/nms-graph/src/model.rs';body=(SOURCE/rel).read_bytes();old=git('show',BASELINE+':'+rel)
 anchor=b'            active_galaxy: 0,';assert body.count(anchor)==1
 assert not compare(body.replace(anchor,b'            active_galaxy: 1,',1),old,rel)
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'changed_paths':sorted(changed),'all_9_ordered_non_doc_comparisons':True,'permitted_named_layout_sites':sum(map(len,LAYOUTS.values())),'opposed_in_memory_executable_mutation_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
