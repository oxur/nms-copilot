#!/usr/bin/env bash
# Adapted from oxur/vlajmaji tests/test-build-tools.sh.
# Real-Git planning fixtures use disposable local bare remotes.
# No network, Python, Bats, or Rust compiler is needed. Bash 3.2 compatible.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")/.." && pwd -P)
make_command=${MAKE:-make}
run_root=$(mktemp -d "${TMPDIR:-/tmp}/build-tools.XXXXXX")
# Isolate fixtures from user Git configuration, hooks, signing and environment.
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES
unset GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS GIT_CONFIG GIT_NAMESPACE
unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
export LC_ALL=C

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
equal() { [[ $1 == "$2" ]] || fail "expected [$2], got [$1]"; }
contains() { grep -F -- "$2" "$1" >/dev/null || fail "$1 does not contain: $2"; }
excludes() { if grep -F -- "$2" "$1" >/dev/null; then fail "$1 unexpectedly contains: $2"; fi; }
# A failed command is expected only here; assertions themselves always exit.
reject() {
    local code
    if "$@" > "$case_root/rejected.log" 2>&1; then fail "unexpected success: $*"; else code=$?; fi
    [[ $code -ne 0 ]] || fail 'failure was not propagated'
}
g() { git -C "$source" "$@"; }
w() { git -C "$work" "$@"; }
plan() { "$make_command" -C "$source" --no-print-directory "BASH=$BASH" planning; }
identity() {
    git -C "$1" config user.name 'Fixture User'
    git -C "$1" config user.email fixture@example.invalid
    git -C "$1" config commit.gpgsign false
}
fixture() {
    local base=${1:-$case_root}
    mkdir -p "$base"
    source=$base/'source checkout with spaces'
    origin=$base/origin.git
    work=$source/.worktrees/planning
    mkdir -p "$source"
    git init -b main "$source"
    identity "$source"
    printf 'source base\n' > "$source/tracked.txt"
    g add tracked.txt
    g commit -m 'source base'
    git init --bare "$origin"
    git --git-dir="$origin" symbolic-ref HEAD refs/heads/main
    g remote add origin "$origin"
    g push -u origin main
    cp "$repo/Makefile" "$repo/.gitignore" "$source/"
    mkdir "$source/scripts"
    cp "$repo/scripts/planning-worktree.sh" "$source/scripts/"
    g add Makefile .gitignore scripts/planning-worktree.sh
    g commit -m 'fixture build tooling'
    cp "$source/.git/index" "$base/index-before"
    head_before=$(g rev-parse HEAD)
    index_before=$base/index-before
}
unchanged() {
    equal "$(g rev-parse HEAD)" "$head_before"
    # Compare before status, which may refresh the index's stat cache.
    cmp "$source/.git/index" "$index_before" || fail 'implementation index changed'
    equal "$(g status --porcelain)" ''
}
remote_head() { git --git-dir="$origin" rev-parse refs/heads/planning; }
empty_root() {
    local commit
    commit=$(w rev-parse HEAD)
    equal "$(w rev-list --parents -n 1 "$commit")" "$commit"
    equal "$(w ls-tree -r --name-only HEAD)" ''
}
clone_planning() {
    clone=$case_root/'remote update'
    git clone "$origin" "$clone"
    identity "$clone"
    git -C "$clone" switch planning
}
remote_update() {
    clone_planning
    printf 'remote advanced\n' > "$clone/later.txt"
    git -C "$clone" add later.txt
    git -C "$clone" commit -m 'remote update'
    git -C "$clone" push origin planning
}
local_update() {
    printf 'local\n' > "$work/local.txt"
    w add local.txt
    w commit -m 'local ahead'
}

help_and_planning_alias_do_not_require_rust() {
    local fake=$case_root/no-rust
    mkdir "$fake"
    printf '#!/bin/sh\nexit 127\n' > "$fake/rustc"
    cp "$fake/rustc" "$fake/cargo"
    chmod +x "$fake/rustc" "$fake/cargo"
    export PATH=$fake:$PATH
    "$make_command" -C "$repo" --no-print-directory help > "$case_root/help.log"
    contains "$case_root/help.log" 'make planning'
    contains "$case_root/help.log" 'make planning-worktree'
    contains "$case_root/help.log" 'make test-tools'
    fixture
    "$make_command" -C "$source" --no-print-directory "BASH=$BASH" planning-worktree
    empty_root
    unchanged
}

create_empty_root_push_idempotence_and_preserve_source() {
    fixture
    plan > "$case_root/first.log"
    local commit
    commit=$(w rev-parse HEAD)
    empty_root
    equal "$(remote_head)" "$commit"
    contains "$case_root/first.log" 'created empty planning root'
    plan
    equal "$(w rev-parse HEAD)" "$commit"
    unchanged
}
remote_branch_without_local_branch_attaches() {
    fixture
    clone=$case_root/seed
    git clone "$origin" "$clone"
    identity "$clone"
    git -C "$clone" switch --orphan planning
    git -C "$clone" commit --allow-empty -m 'remote plan'
    git -C "$clone" push origin planning
    plan
    equal "$(w branch --show-current)" planning
    equal "$(w rev-parse HEAD)" "$(remote_head)"
    unchanged
}
remote_update_fast_forwards_clean_planning_worktree() {
    fixture; plan; remote_update; plan
    equal "$(cat "$work/later.txt")" 'remote advanced'
    equal "$(w rev-parse HEAD)" "$(remote_head)"
    unchanged
}
local_ahead_preserved_without_push() {
    fixture; plan
    local remote before
    remote=$(remote_head)
    local_update
    before=$(w rev-parse HEAD)
    plan
    equal "$(w rev-parse HEAD)" "$before"
    equal "$(remote_head)" "$remote"
    unchanged
}
existing_local_branch_without_remote_is_attached_and_pushed() {
    fixture
    g branch planning HEAD
    local expected
    expected=$(g rev-parse refs/heads/planning)
    plan
    equal "$(w rev-parse HEAD)" "$expected"
    equal "$(remote_head)" "$expected"
    unchanged
}
dirty_planning_refused_without_data_loss() {
    fixture; plan
    printf 'preserve me' > "$work/untracked"
    reject plan
    contains "$case_root/rejected.log" 'is dirty'
    equal "$(cat "$work/untracked")" 'preserve me'
    unchanged
}
true_divergence_refused() {
    fixture; plan; remote_update; local_update
    local before remote
    before=$(w rev-parse HEAD)
    remote=$(remote_head)
    reject plan
    contains "$case_root/rejected.log" diverged
    equal "$(w rev-parse HEAD)" "$before"
    equal "$(remote_head)" "$remote"
    unchanged
}
path_collision_and_wrong_branch_refused() {
    fixture "$case_root/collision"
    mkdir -p "$work"
    printf keep > "$work/keep"
    reject plan
    equal "$(cat "$work/keep")" keep
    unchanged
    fixture "$case_root/wrong-branch"
    g worktree add -b other "$work" HEAD
    reject plan
    equal "$(w branch --show-current)" other
    [[ -f $work/tracked.txt ]] || fail 'wrong-branch worktree damaged'
    unchanged
}
unreachable_origin_is_not_treated_as_missing_branch() {
    fixture
    g remote set-url origin "$case_root/absent.git"
    reject plan
    contains "$case_root/rejected.log" 'could not query remote branch'
    [[ ! -e $work ]] || fail 'created worktree after remote error'
    reject g show-ref --verify --quiet refs/heads/planning
    unchanged
}
implementation_dirty_state_is_preserved() {
    fixture
    printf 'staged change\n' > "$source/tracked.txt"
    g add tracked.txt
    cp "$source/.git/index" "$case_root/staged-index"
    printf 'staged plus unstaged\n' > "$source/tracked.txt"
    printf 'preserve\n' > "$source/untracked.txt"
    plan
    equal "$(g rev-parse HEAD)" "$head_before"
    cmp "$source/.git/index" "$case_root/staged-index"
    equal "$(cat "$source/tracked.txt")" 'staged plus unstaged'
    equal "$(cat "$source/untracked.txt")" preserve
}
branch_checked_out_elsewhere_is_refused() {
    fixture
    g branch planning HEAD
    g worktree add "$case_root/already attached" planning
    reject plan
    contains "$case_root/rejected.log" 'already checked out elsewhere'
    [[ -f $case_root/'already attached'/tracked.txt && ! -e $work ]] || fail 'unexpected worktree change'
    unchanged
}
push_rejection_preserves_empty_local_root_and_worktree() {
    fixture
    printf '#!/bin/sh\necho fixture push rejection >&2\nexit 1\n' > "$origin/hooks/pre-receive"
    chmod +x "$origin/hooks/pre-receive"
    reject plan
    contains "$case_root/rejected.log" 'fixture push rejection'
    empty_root
    reject git --git-dir="$origin" show-ref --verify --quiet refs/heads/planning
    unchanged
}
symlink_path_is_refused() {
    fixture
    mkdir -p "$case_root/outside" "$source/.worktrees"
    ln -s "$case_root/outside" "$work"
    reject plan
    contains "$case_root/rejected.log" symlink
    [[ ! -e $case_root/outside/.git ]] || fail 'symlink destination was changed'
    rm "$work"
    plan
    local commit suffix
    commit=$(w rev-parse HEAD)
    ln -s .worktrees/planning "$source/planning-alias"
    cd "$source"
    for suffix in / /.; do
        reject "$BASH" scripts/planning-worktree.sh --path "planning-alias$suffix"
        contains "$case_root/rejected.log" symlink
        equal "$(w rev-parse HEAD)" "$commit"
    done
    rm "$source/planning-alias"
    unchanged
}
local_branch_behind_remote_without_worktree_fast_forwards_once() {
    fixture; plan
    local old
    old=$(w rev-parse HEAD)
    g worktree remove "$work"
    remote_update
    equal "$(g rev-parse refs/heads/planning)" "$old"
    plan
    equal "$(cat "$work/later.txt")" 'remote advanced'
    equal "$(w rev-parse HEAD)" "$(remote_head)"
    unchanged
}
helper_can_be_invoked_from_repository_subdirectory() {
    fixture
    mkdir "$source/nested"
    (cd "$source/nested" && "$BASH" ../scripts/planning-worktree.sh --path .worktrees/planning) > "$case_root/direct.log"
    contains "$case_root/direct.log" 'created empty planning root'
    equal "$(w branch --show-current)" planning
    unchanged
}
git_administration_path_is_refused() {
    fixture
    cd "$source"
    reject "$BASH" scripts/planning-worktree.sh --path .git/planning
    contains "$case_root/rejected.log" 'Git administration'
    [[ ! -e $source/.git/planning ]] || fail 'Git administration data was changed'
    ln -s .git "$source/admin-alias"
    g worktree list --porcelain -z > "$case_root/worktrees-before"
    reject "$BASH" scripts/planning-worktree.sh --path missing/../admin-alias/review-planning
    contains "$case_root/rejected.log" 'Git administration'
    if [[ -d $source/.GIT ]]; then
        reject "$BASH" scripts/planning-worktree.sh --path .GIT/casefold-planning
        contains "$case_root/rejected.log" 'Git administration'
        [[ ! -e $source/.git/casefold-planning ]] || fail 'case alias bypassed Git directory guard'
    fi
    [[ ! -e $source/.git/review-planning ]] || fail 'created a worktree in Git administration data'
    g worktree list --porcelain -z > "$case_root/worktrees-after"
    cmp "$case_root/worktrees-before" "$case_root/worktrees-after"
    reject g show-ref --verify --quiet refs/heads/planning
    reject git --git-dir="$origin" show-ref --verify --quiet refs/heads/planning
    rm "$source/admin-alias"
    unchanged
}

fresh_clone_attaches_remote_planning_without_identity() {
    fixture; plan
    g push origin main
    local fresh=$case_root/'fresh clone with spaces'
    git clone "$origin" "$fresh"
    "$make_command" -C "$fresh" --no-print-directory "BASH=$BASH" planning
    equal "$(git -C "$fresh/.worktrees/planning" rev-parse HEAD)" "$(remote_head)"
    equal "$(git -C "$fresh" status --porcelain)" ''
    unchanged
}
locked_planning_is_preserved() {
    fixture; plan; remote_update
    local before
    before=$(w rev-parse HEAD)
    g worktree lock "$work" --reason 'fixture protected worktree'
    reject plan
    contains "$case_root/rejected.log" 'locked or prunable'
    equal "$(w rev-parse HEAD)" "$before"
    g worktree unlock "$work"
    unchanged
}
missing_registered_worktree_is_not_pruned_or_replaced() {
    fixture; plan
    mv "$work" "$case_root/retained planning"
    g worktree list --porcelain -z > "$case_root/registrations-before"
    reject plan
    contains "$case_root/rejected.log" 'locked or prunable'
    g worktree list --porcelain -z > "$case_root/registrations-after"
    cmp "$case_root/registrations-before" "$case_root/registrations-after"
    [[ ! -e $work ]] || fail 'missing worktree unexpectedly recreated'
    mv "$case_root/retained planning" "$work"
    unchanged
}
tracked_paths_and_implementation_root_are_refused() {
    fixture
    cd "$source"
    reject "$BASH" scripts/planning-worktree.sh --path .
    contains "$case_root/rejected.log" 'implementation checkout'
    reject "$BASH" scripts/planning-worktree.sh --path tracked.txt/child
    contains "$case_root/rejected.log" 'tracked implementation content'
    equal "$(cat "$source/tracked.txt")" 'source base'
    [[ ! -e $work ]] || fail 'created planning despite invalid target'
    unchanged
}
failed_make_target_never_reports_ready() {
    fixture
    mkdir -p "$work"
    printf keep > "$work/keep"
    reject plan
    excludes "$case_root/rejected.log" 'Planning worktree ready'
    equal "$(cat "$work/keep")" keep
    unchanged
}
retry_after_rejected_initial_push_reuses_root() {
    fixture
    printf '#!/bin/sh\nexit 1\n' > "$origin/hooks/pre-receive"
    chmod +x "$origin/hooks/pre-receive"
    reject plan
    local before
    before=$(w rev-parse HEAD)
    empty_root
    rm "$origin/hooks/pre-receive"
    plan
    equal "$(w rev-parse HEAD)" "$before"
    equal "$(remote_head)" "$before"
    contains "$case_root/rejected.log" 'rejected'
    unchanged
}

cases=(
    fresh_clone_attaches_remote_planning_without_identity
    locked_planning_is_preserved
    missing_registered_worktree_is_not_pruned_or_replaced
    tracked_paths_and_implementation_root_are_refused
    failed_make_target_never_reports_ready
    retry_after_rejected_initial_push_reuses_root
    help_and_planning_alias_do_not_require_rust
    create_empty_root_push_idempotence_and_preserve_source
    remote_branch_without_local_branch_attaches
    remote_update_fast_forwards_clean_planning_worktree
    local_ahead_preserved_without_push
    existing_local_branch_without_remote_is_attached_and_pushed
    dirty_planning_refused_without_data_loss
    true_divergence_refused
    path_collision_and_wrong_branch_refused
    unreachable_origin_is_not_treated_as_missing_branch
    implementation_dirty_state_is_preserved
    branch_checked_out_elsewhere_is_refused
    push_rejection_preserves_empty_local_root_and_worktree
    symlink_path_is_refused
    local_branch_behind_remote_without_worktree_fast_forwards_once
    helper_can_be_invoked_from_repository_subdirectory
    git_administration_path_is_refused
)
failed=0
passed=0
for test_case in "${cases[@]}"; do
    case_root=$run_root/$test_case
    mkdir "$case_root"
    # Do not put the function in an if/|| condition: that disables errexit in
    # the whole function. Only the parent disables it to collect the exit code.
    set +e
    (set -euo pipefail; cd "$case_root"; "$test_case") > "$case_root/output.log" 2>&1
    status=$?
    set -e
    if [[ $status -eq 0 ]]; then
        printf 'ok - %s\n' "$test_case"
        passed=$((passed + 1))
    else
        printf 'not ok - %s (exit %s)\n' "$test_case" "$status" >&2
        cat "$case_root/output.log" >&2
        failed=$((failed + 1))
    fi
done
printf '%s passed; %s failed (%s total)\n' "$passed" "$failed" "${#cases[@]}"
if [[ $failed -gt 0 ]]; then
    printf 'Fixtures and logs retained at %s\n' "$run_root" >&2
    exit 1
fi
rm -rf -- "$run_root"
