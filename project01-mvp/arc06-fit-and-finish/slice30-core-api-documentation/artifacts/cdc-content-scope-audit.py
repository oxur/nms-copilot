"""Check the issued comment-only scope; this is not a Rust lexer or AST proof."""
from pathlib import Path
import hashlib
import json
import re
import subprocess

SOURCE = Path('/Users/oubiwann/lab/oxur/nms-copilot/.worktrees/public-api-documentation')
BASE = '4a935408460349e9aa64fecd570d9960094191b1'
ALLOWED = {'crates/nms-core/src/' + name + '.rs' for name in
           ('lib', 'address', 'biome', 'delta', 'discovery', 'galaxy', 'glyph', 'player', 'system')}

def git(*args):
    return subprocess.check_output(['git', *args], cwd=SOURCE)

TUPLES = {'biome': 'BiomeParseError', 'discovery': 'DiscoveryParseError',
          'galaxy': 'GalaxyTypeParseError', 'glyph': 'GlyphParseError',
          'player': 'BaseTypeParseError'}

def content(data, path):
    value = b''.join(line for line in data.splitlines(keepends=True)
                     if line.strip() and not line.lstrip().startswith((b'///', b'//!')))
    stem = Path(path).stem
    if stem in TUPLES:
        name = TUPLES[stem].encode()
        pattern = rb'pub struct ' + name + rb'\(\s*pub String,?\s*\);'
        value, count = re.subn(pattern, b'pub struct ' + name + b'(pub String);', value)
        assert count == 1, (path, count)
    if stem == 'system':
        value, count = re.subn(rb'Community\s*\{\s*source: String,?\s*\},',
                               b'Community { source: String },', value)
        assert count == 1, (path, count)
    return value

def main():
    changed = {x.decode() for x in git('diff', BASE, '--name-only', '-z').split(b'\0') if x}
    assert changed <= ALLOWED, sorted(changed - ALLOWED)
    unknown = [x.decode() for x in git('ls-files', '--others', '--exclude-standard', '-z').split(b'\0') if x]
    assert not unknown, unknown
    result = []
    for path in sorted(ALLOWED):
        old = git('show', BASE + ':' + path)
        new = (SOURCE / path).read_bytes()
        assert content(old, path) == content(new, path), path
        result.append({'path': path, 'before_sha256': hashlib.sha256(old).hexdigest(),
                       'after_sha256': hashlib.sha256(new).hexdigest(),
                       'normalized_non_doc_bytes': len(content(new, path))})
    original = git('show', BASE + ':crates/nms-core/src/lib.rs')
    assert b'pub mod address;' in original
    mutant = original.replace(b'pub mod address;', b'pub mod addresz;', 1)
    assert content(original, 'lib.rs') != content(mutant, 'lib.rs'), 'opposed executable-line mutation escaped'
    print(json.dumps({'baseline': BASE, 'source_head': git('rev-parse', 'HEAD').decode().strip(),
                      'changed': sorted(changed), 'complete_allowed_population': result,
                      'synthetic_module_declaration_mutation_rejected': True}, indent=2))

if __name__ == '__main__':
    main()
