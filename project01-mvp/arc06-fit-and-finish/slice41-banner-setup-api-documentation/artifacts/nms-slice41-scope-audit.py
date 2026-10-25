"""Slice41 four-file ordered documentation-only preservation and opposed scalar-width control."""
import argparse,hashlib,json,pathlib,subprocess
SOURCE=pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/banner-setup-api-documentation')
BASELINE='fff40ed23c02184e70f175cc3fc2dc687862ec34'
PATHS=['crates/nms-copilot/src/banner.rs','crates/nms-copilot/src/setup.rs','crates/nms-copilot/src/lib.rs','crates/nms/src/lib.rs']
ANCHOR=b'\n            width += 1;\n'
def git(*args):return subprocess.check_output(['git',*args],cwd=SOURCE)
def non_doc(body):return b''.join(line for line in body.splitlines(keepends=True) if line.strip() and not line.lstrip().startswith((b'///',b'//!')))
def opposed(body):
 assert body.count(ANCHOR)==1,'actual scalar-width increment must be unique'
 assert non_doc(body)!=non_doc(body.replace(ANCHOR,b'\n            width += 2;\n',1)),'executable scalar-width mutation escaped comparison'
def main(support):
 if support:
  for rel in PATHS:
   body=git('show',BASELINE+':'+rel);assert non_doc(body)==non_doc(b'//! Synthetic support-only doc addition.\n'+body)
  opposed(git('show',BASELINE+':'+PATHS[0]))
  print(json.dumps({'support_only':True,'four_documentation_additions_preserve_ordered_non_doc':True,'actual_scalar_width1_to2_rejected':True}));return
 changed=set(git('diff','--name-only',BASELINE).decode().splitlines());assert changed==set(PATHS),changed
 assert not git('ls-files','--others','--exclude-standard','-z')
 for rel in PATHS:assert non_doc((SOURCE/rel).read_bytes())==non_doc(git('show',BASELINE+':'+rel)),rel
 opposed((SOURCE/PATHS[0]).read_bytes())
 print(json.dumps({'source':str(SOURCE),'baseline':BASELINE,'head':git('rev-parse','HEAD').decode().strip(),'changed_paths':sorted(changed),'ordered_non_doc_comparisons':4,'actual_scalar_width1_to2_rejected':True,'diff_sha256':hashlib.sha256(git('diff',BASELINE,'--binary')).hexdigest()}))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--self-test',action='store_true');args=p.parse_args();main(args.self_test)
