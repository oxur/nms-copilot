import json,base64,hashlib,subprocess
from pathlib import Path
root=Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/user-documentation-contract')
ret=root/'workbench/slice29-return'
ccfile=Path('/private/tmp/nms-slice29-cc-attempts.json')
assert hashlib.sha256(ccfile.read_bytes()).hexdigest()=='0d58ae7f632213c033763c7af4e140e7c8d262eb01c5324166dc8da1a34c5d18'
cc=json.loads(ccfile.read_text())['attempts']; assert len(cc)==29
v=json.loads((ret/'validation.json').read_text()); assert len(v['attempts'])==29
for raw,summary in zip(cc,v['attempts']):
    assert raw['status']=='complete'
    for name in ('id','label','kind','argv','cwd','status','started_utc','overrides'):assert summary[name]==raw[name],(raw['id'],name)
    c=raw['raw_completion']
    assert summary['exit_code']==c['exit_code'] and summary['ended_utc']==c['ended_utc']
    for name in ('stdout','stderr'):
        a=c[name]; b=base64.b64decode(a['base64'],validate=True)
        assert len(b)==a['bytes'] and hashlib.sha256(b).hexdigest()==a['sha256']
        for k,val in summary[name].items(): assert a[k]==val
    for when in ('before','after'):
        assert {'head','status','working_diff_sha256','working_identity_sha256','untracked_files'}<=raw[when].keys()
        for k,val in summary[when].items(): assert raw[when][k]==val
expected={'execution.md':'16cff0e9a0b1107e63b6a36e97e469bad8c573e9fa60e9ae50b57b33cba22981','validation.json':'9036938a2506d58b2cc48f82ee6078cabc07f41ff212de5e931909b51a60f29b','closing-report.md':'2a72b096c24fa9a255d2e5e169d6ff4ac59cc2c0fdf334bf018047a648a0f05d'}
for name,digest in expected.items():assert hashlib.sha256((ret/name).read_bytes()).hexdigest()==digest
for name,meta in v['artifact_inventory']['tracked_source_files'].items():
    b=(root/name).read_bytes();assert len(b)==meta['bytes'] and hashlib.sha256(b).hexdigest()==meta['sha256']
allowed=set(v['source_commit']['changed_paths'])
changed=set(subprocess.check_output(['git','diff','--name-only','be4145b631405ff19c3e403457ecaa581afe805f','HEAD'],cwd=root,text=True).splitlines())
assert changed==allowed and len(allowed)==8
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()=='4a935408460349e9aa64fecd570d9960094191b1'
cdc=json.loads(Path('/private/tmp/nms-slice29-cdc-attempts.json').read_text())['attempts']
assert len(cdc)==15 and cdc[-1]['label']=='CDC-evidence-integrity-support' and cdc[-1]['status']=='running'
for raw in cdc[:-1]:
    assert raw['status']=='complete' and raw['before']==raw['after'] and raw['before']['status']==''
    for name in ('stdout','stderr'):
        a=raw['raw_completion'][name];b=base64.b64decode(a['base64'],validate=True)
        assert len(b)==a['bytes'] and hashlib.sha256(b).hexdigest()==a['sha256']
print('29 CC raw records and complete return projections match; original 3 returns/8 source files match hashes; all other production bytes unchanged; 14 prior CDC raw records complete')
