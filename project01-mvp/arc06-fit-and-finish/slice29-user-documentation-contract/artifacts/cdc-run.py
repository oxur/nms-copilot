import runpy,json,base64
from pathlib import Path
m=runpy.run_path('/private/tmp/nms-slice29-cdc-capture.py')
log=Path('/private/tmp/nms-slice29-cdc-attempts.json')
controls=json.loads(log.read_text())['attempts']
assert len(controls)==2
for x,code in zip(controls,(7,0)):
    assert x['status']=='complete' and x['raw_completion']['exit_code']==code
    assert x['before']==x['after'] and x['before']['status']==''
    assert base64.b64decode(x['raw_completion']['stdout']['base64'])==b'out\x00value\n'
    assert base64.b64decode(x['raw_completion']['stderr']['base64'])==b'err\x00\n'
bin=str(m['SOURCE']/'target/debug/nms')
font='/System/Library/Fonts/Menlo.ttc'
cache={'PYTHONPYCACHEPREFIX':'/private/tmp/nms-slice29-cdc-pycache'}
commands=[
('CDC-format',['make','format'],{},0),
('CDC-lint',['make','lint'],{},0),
('CDC-renderer-pycompile',['python3','-m','py_compile','docs/render-cli-capture.py'],cache,0),
('CDC-render-help',['python3','docs/render-cli-capture.py','docs/captures/nms-help.txt','/private/tmp/nms-slice29-cdc-rendered/nms-help.png','--command','nms --help','--font',font],cache,0),
('CDC-render-stats',['python3','docs/render-cli-capture.py','docs/captures/nms-stats.txt','/private/tmp/nms-slice29-cdc-rendered/nms-stats.png','--command','nms stats --save data/test/save56_multi_system.json --biomes','--font',font],cache,0),
('CDC-route-comma-negative',[bin,'route','--save','data/test/save56_multi_system.json','--biome','Lush,Swamp','--round-trip'],{},1),
('CDC-route-Lush-roundtrip',[bin,'route','--save','data/test/save56_multi_system.json','--biome','Lush','--round-trip'],{},0),
('CDC-doc-checker',['python3','workbench/slice29-return/check_docs.py'],cache,0),
('CDC-authored-working-diff',['git','diff','HEAD','--check','--','.',':(exclude)docs/captures/nms-stats.txt'],{},0),
('CDC-authored-baseline-diff',['git','diff','be4145b631405ff19c3e403457ecaa581afe805f','--check','--','.',':(exclude)docs/captures/nms-stats.txt'],{},0),
('CDC-complete-baseline-immutable-data-finding',['git','diff','be4145b631405ff19c3e403457ecaa581afe805f','--check'],{},2),
('CDC-independent-render-pixel-comparison',['python3','/private/tmp/nms-slice29-cdc-check-images.py'],cache,0),
]
for label,argv,overrides,expected in commands:
    actual=m['capture'](label,argv,overrides,kind='Independent appropriate check' if expected!=2 else 'Complete default check; expected immutable captured-data EOF finding, not green')
    assert actual==expected,(label,actual,expected)
    x=json.loads(log.read_text())['attempts'][-1]
    assert x['status']=='complete' and x['before']==x['after'] and x['after']['status']==''
    if label=='CDC-route-comma-negative':
        assert b'Invalid biome' in base64.b64decode(x['raw_completion']['stderr']['base64'])
    if expected==2:
        assert base64.b64decode(x['raw_completion']['stdout']['base64'])==b'docs/captures/nms-stats.txt:25: new blank line at EOF.\n'
        assert x['raw_completion']['stderr']['bytes']==0
print('Independent complete: 11 appropriate checks, 1 explicitly retained default data EOF finding, 2 support controls')
