from pathlib import Path
from PIL import Image
root=Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/user-documentation-contract')
for name in ('nms-help','nms-stats'):
    with Image.open(root/'docs/captures'/f'{name}.png') as a, Image.open(Path('/private/tmp/nms-slice29-cdc-rendered')/f'{name}.png') as b:
        assert a.mode==b.mode and a.size==b.size and a.tobytes()==b.tobytes(), name
        print(name, a.size, 'complete pixel reproduction matches')
