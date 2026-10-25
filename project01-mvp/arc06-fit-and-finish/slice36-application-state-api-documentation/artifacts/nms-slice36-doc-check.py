"""Validate actual Cargo JSON against exact residual owners; no full-crate acceptance."""
import argparse,base64,collections,copy,hashlib,json,pathlib,subprocess
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/application-state-api-documentation')
REGISTRY=pathlib.Path('/private/tmp/nms-slice36-doc-registry.json')
ARGV=['cargo','doc','-p','nms-copilot','--lib','--all-features','--no-deps','--offline','--message-format=json']
FLAGS='-D warnings --force-warn missing_docs'
def raw_channel(item):
 b=base64.b64decode(item['base64'],validate=True)
 assert len(b)==item['bytes'] and hashlib.sha256(b).hexdigest()==item['sha256'],'raw channel mismatch'
 return b
def fingerprint(message):
 spans=[s for s in message['spans'] if s['is_primary']]
 assert len(spans)==1,'unexpected primary span count'
 return (spans[0]['file_name'],(message.get('code') or {}).get('code'),message['message'],tuple(t['text'] for t in spans[0]['text']))
def keyed(rows):return collections.Counter((x['path'],x['code'],x['message'],tuple(x['primary_text'])) for x in rows)
def messages(stdout):
 result=[]; finished=[]
 for line in stdout.splitlines():
  obj=json.loads(line);reason=obj['reason']
  assert reason in {'compiler-artifact','build-script-executed','compiler-message','build-finished'},('unexpected Cargo JSON',reason)
  if reason=='build-finished':finished.append(obj['success'])
  if reason=='compiler-message':
   assert obj['manifest_path']==str(SOURCE/'crates/nms-copilot/Cargo.toml') and obj['target']['kind']==['lib'] and obj['target']['name']=='nms_copilot','unexpected diagnostic target'
   result.append(obj['message'])
 assert finished==[True],'compiler did not successfully finish'
 return result
def validate(items,registry,mode):
 expected=registry['all_baseline'] if mode=='baseline' else registry['residual']
 for m in items:
  assert m['level']=='warning' and (m.get('code') or {}).get('code')=='missing_docs','unexpected diagnostic class'
  def children(ms):
   for c in ms:
    assert c['level'] in {'note','help'},'unexpected nested diagnostic';children(c.get('children',[]))
  children(m.get('children',[]))
  if mode=='scoped':assert fingerprint(m)[0] not in registry['owned_paths'],'owned missing-doc diagnostic'
 actual=collections.Counter(fingerprint(m) for m in items)
 assert actual==keyed(expected),('residual population differs',actual-keyed(expected),keyed(expected)-actual)
 return len(items)
def current_metadata():
 def git(*a):return subprocess.check_output(['git',*a],cwd=SOURCE)
 untracked={}
 for b in git('ls-files','--others','--exclude-standard','-z').split(b'\0'):
  if b:
   rel=b.decode();data=(SOURCE/rel).read_bytes();untracked[rel]={'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
 head=git('rev-parse','HEAD').decode().strip();diff=hashlib.sha256(git('diff','HEAD','--binary')).hexdigest()
 ident={'head':head,'tracked_diff_sha256':diff,'untracked':untracked}
 return {'head':head,'status':git('status','--short').decode(),'working_diff_sha256':diff,'untracked_files':untracked,'working_identity_sha256':hashlib.sha256(json.dumps(ident,sort_keys=True).encode()).hexdigest()}
def checked_entry(log,identity,mode):
 reg=json.loads(REGISTRY.read_text());data=json.loads(pathlib.Path(log).read_text());matches=[x for x in data['attempts'] if x['id']==identity];assert len(matches)==1
 entry=matches[0];assert entry['status']=='complete' and entry['raw_completion']['exit_code']==0
 assert entry['argv']==ARGV and entry['cwd']==str(SOURCE) and entry['overrides']=={'CARGO_NET_OFFLINE':'true','RUSTDOCFLAGS':FLAGS}
 assert entry['before']==entry['after']==current_metadata(),'stale compiler source identity'
 stdout=raw_channel(entry['raw_completion']['stdout']);stderr=raw_channel(entry['raw_completion']['stderr']).decode()
 for line in stderr.splitlines():
  if line.lstrip().startswith('warning:'):assert line in reg['allowed_cargo_warning_lines'],'unexpected Cargo warning'
  assert not line.lstrip().startswith('error:'),'unexpected Cargo error'
 items=messages(stdout);count=validate(items,reg,mode)
 return reg,items,{'record':identity,'mode':mode,'diagnostic_count':count,'owned_missing_docs':sum(fingerprint(m)[0] in reg['owned_paths'] for m in items),'residual_expected':len(reg['residual']),'source':entry['before'],'full_crate_strict_acceptance':False}
def self_test(log,identity):
 reg,items,out=checked_entry(log,identity,'baseline')
 residual=[copy.deepcopy(m) for m in items if fingerprint(m)[0] not in reg['owned_paths']]
 assert validate(residual,reg,'scoped')==24
 cases=[]
 own=next(copy.deepcopy(m) for m in items if fingerprint(m)[0] in reg['owned_paths'])
 unexpected=copy.deepcopy(residual[0]);unexpected['code']={'code':'rustdoc::broken_intra_doc_links'}
 for name,rows in [('owned',residual+[own]),('unexpected',residual+[unexpected]),('missing',residual[1:]),('duplicate',residual+[copy.deepcopy(residual[0])])]:
  try:validate(rows,reg,'scoped')
  except AssertionError:cases.append(name)
  else:raise AssertionError('accepted negative '+name)
 print(json.dumps({'support_only':True,'actual_baseline':out,'synthetic_scoped_positive':24,'rejected_controls':cases,'no_module_or_whole_crate_acceptance_credit':True}))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--log',required=True);p.add_argument('--id',required=True);p.add_argument('--mode',choices=['baseline','scoped']);p.add_argument('--self-test',action='store_true');a=p.parse_args()
 if a.self_test:self_test(a.log,a.id)
 else:
  assert a.mode is not None;print(json.dumps(checked_entry(a.log,a.id,a.mode)[2]))
