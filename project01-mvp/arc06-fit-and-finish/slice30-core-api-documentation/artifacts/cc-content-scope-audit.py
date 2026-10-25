import hashlib
import json
import pathlib
import re
import subprocess

source = pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/public-api-documentation')
manifest_path = pathlib.Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/planning/project01-mvp/arc06-fit-and-finish/slice30-core-api-documentation/source-allowlist.json')
manifest = json.loads(manifest_path.read_text())
root = str(source.resolve())
assert root == manifest['source_worktree']
paths = manifest['paths']

def git(*args):
    return subprocess.check_output(['git', *args], cwd=source)

def non_doc(data):
    kept = []
    for line in data.splitlines(keepends=True):
        stripped = line.lstrip()
        if not line.strip() or stripped.startswith((b'///', b'//!')):
            continue
        kept.append(line)
    return b''.join(kept)

def normalize_payload_layouts(data, rel):
    declarations = {
        'crates/nms-core/src/biome.rs': (b'BiomeParseError',),
        'crates/nms-core/src/discovery.rs': (b'DiscoveryParseError',),
        'crates/nms-core/src/galaxy.rs': (b'GalaxyTypeParseError',),
        'crates/nms-core/src/glyph.rs': (b'GlyphParseError',),
        'crates/nms-core/src/player.rs': (b'BaseTypeParseError',),
    }
    for name in declarations.get(rel, ()):
        pattern = rb'pub struct ' + name + rb'\(\s*pub String,\s*\);'
        data, count = re.subn(pattern, b'pub struct ' + name + b'(pub String);', data)
        assert count == 1, (rel, name, count)
    if rel == 'crates/nms-core/src/system.rs':
        data, count = re.subn(rb'Community\s*\{\s*source: String,\s*\},', b'Community { source: String },', data)
        assert count == 1, ('DiscoverySource::Community.source', count)
    return data

def compare(data, baseline, rel):
    return normalize_payload_layouts(non_doc(data), rel) == non_doc(baseline)

changed = set(git('diff', '--name-only', 'HEAD').decode().splitlines())
assert changed == set(paths), {'changed': sorted(changed), 'allowlist': sorted(paths)}
untracked = [p for p in git('ls-files', '--others', '--exclude-standard', '-z').decode().split('\0') if p]
assert not untracked, untracked
for rel in paths:
    current = (source / rel).read_bytes()
    baseline = git('show', 'HEAD:' + rel)
    assert compare(current, baseline, rel), rel

# Opposed control is in memory only: the exact non-doc executable return line changes.
rel = 'crates/nms-core/src/address.rs'
current = (source / rel).read_bytes()
baseline = git('show', 'HEAD:' + rel)
anchor = b'        self.packed\n    }\n'
assert current.count(anchor) == 1, ('synthetic mutation anchor count', current.count(anchor))
mutated = current.replace(anchor, b'        self.packed ^ 1\n    }\n', 1)
assert not compare(mutated, baseline, rel), 'scope comparator failed to reject synthetic executable mutation'
print(json.dumps({
    'source': root,
    'head': git('rev-parse', 'HEAD').decode().strip(),
    'changed_paths': sorted(changed),
    'untracked_paths': untracked,
    'permitted_declaration_normalizations': 6,
    'non_doc_comparisons': len(paths),
    'all_current_paths_match_baseline_after_permitted_normalization': True,
    'synthetic_executable_mutation_rejected': True,
    'synthetic_mutation_applied_in_memory_only': True,
    'current_diff_sha256': hashlib.sha256(git('diff', 'HEAD', '--binary')).hexdigest(),
}, sort_keys=True))
