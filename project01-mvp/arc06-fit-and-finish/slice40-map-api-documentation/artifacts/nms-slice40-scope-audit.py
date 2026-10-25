"""Slice40 four-file ordered documentation-only preservation and opposed extent control."""
import argparse,hashlib,json,pathlib,subprocess
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/map-api-documentation')
BASELINE='00cfa740c902f5110159a56329b5241dac706a9d'
PATHS=['crates/nms-copilot/src/map/'+name+'.rs' for name in ['mod','input','render','state']]
ANCHOR=b'\n            Self::Galaxy => 4096.0,\n'
def git(*args):return subprocess.check_output(['git',*args],cwd=SOURCE)
def non_doc(body):return b''.join(line for line in body.splitlines(keepends=True) if line.strip() and not line.lstrip().startswith((b'///',b'//!')))
def opposed(body):
 assert body.count(ANCHOR)==1,'actual Galaxy extent statement must be unique'
 assert non_doc(body)!=non_doc(body.replace(ANCHOR,b'\n            Self::Galaxy => 4095.0,\n',1)),'executable extent mutation escaped comparison'
def main(support):
 if support:
  for rel in PATHS:
   body=git('show',BASELINE+':'+rel);assert non_doc(body)==non_doc(b'//! Synthetic support-only doc addition.\n'+body)
  opposed(git('show',BASELINE+':'+PATHS[3]))
  print(json.dumps({'support_only':True,'four_documentation_additions_preserve_ordered_non_doc':True,'actual_extent4096_to4095_rejected':True}));return
 changed=set(git('diff','--name-only',BASELINE).decode().splitlines());assert changed==set(PATHS),changed
 assert not git('ls-files','--others','--exclude-standard','-z')
 for rel in PATHS:assert non_doc((SOURCE/rel).read_bytes())==non_doc(git('show',BASELINE+':'+rel)),rel
 opposed((SOURCE/PATHS[3]).read_bytes())
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'changed_paths':sorted(changed),'ordered_non_doc_comparisons':4,'actual_extent4096_to4095_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--self-test',action='store_true');args=p.parse_args();main(args.self_test)
