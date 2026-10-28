#!/usr/bin/env bash
# Create or attach a planning worktree while preserving implementation state.
# Adapted from oxur/vlajmaji scripts/planning-worktree.sh.
# Compatible with the Bash 3.2 shipped with macOS.
set -euo pipefail

die() { printf 'planning-worktree: %s\n' "$*" >&2; exit 1; }
usage() {
    printf 'Usage: %s [--remote origin] [--branch planning] [--path .worktrees/planning]\n' "${0##*/}"
}

# Resolve every component, including symlinks reached after a nonexistent/..
# prefix. Resolving only the longest existing ancestor can miss those links.
# This also handles missing suffixes without requiring GNU realpath.
canonical_path() (
    local path=$1 pending part resolved=/ candidate link links=0
    [[ $path == /* ]] || path=$PWD/$path
    pending=${path#/}
    while [[ -n $pending ]]; do
        case $pending in
            */*) part=${pending%%/*}; pending=${pending#*/} ;;
            *) part=$pending; pending='' ;;
        esac
        case $part in
            ''|.) ;;
            ..) resolved=${resolved%/*}; [[ -n $resolved ]] || resolved=/ ;;
            *)
                candidate=${resolved%/}/$part
                if [[ -L $candidate ]]; then
                    links=$((links + 1))
                    [[ $links -le 40 ]] || die 'too many symbolic links in path'
                    link=$(readlink "$candidate") || die "cannot read symbolic link: $candidate"
                    case $link in /*) resolved=/; link=${link#/} ;; esac
                    pending=$link${pending:+/$pending}
                else
                    resolved=$candidate
                fi ;;
        esac
    done
    printf '%s\n' "$resolved"
)

overlaps() {
    local cursor
    if [[ $1 == / || $2 == / || $1 == "$2" || $1 == "$2/"* || $2 == "$1/"* ]]; then
        return 0
    fi
    # On case-insensitive filesystems, different spellings can name the same
    # existing object. Check identity along ancestors of a missing suffix too.
    if [[ -e $2 ]]; then
        cursor=$1
        while :; do
            if [[ $cursor -ef $2 ]]; then return 0; fi
            [[ $cursor != / ]] || break
            cursor=${cursor%/*}
            [[ -n $cursor ]] || cursor=/
        done
    fi
    if [[ -e $1 ]]; then
        cursor=$2
        while :; do
            if [[ $cursor -ef $1 ]]; then return 0; fi
            [[ $cursor != / ]] || break
            cursor=${cursor%/*}
            [[ -n $cursor ]] || cursor=/
        done
    fi
    return 1
}

remote=origin
branch=planning
path=.worktrees/planning
while [[ $# -gt 0 ]]; do
    case $1 in
        --remote|--branch|--path)
            [[ $# -ge 2 ]] || die "missing value for $1"
            case $1 in --remote) remote=$2 ;; --branch) branch=$2 ;; --path) path=$2 ;; esac
            shift 2 ;;
        --remote=*) remote=${1#*=}; shift ;;
        --branch=*) branch=${1#*=}; shift ;;
        --path=*) path=${1#*=}; shift ;;
        -h|--help) usage; exit 0 ;;
        *) die "unknown argument: $1" ;;
    esac
done
[[ -n $branch && $branch != -* ]] || die 'branch must be a non-empty Git branch name'
[[ -n $remote && $remote != -* ]] || die 'remote must be a non-empty name'
[[ -n $path ]] || die 'worktree path must not be empty'
root=$(git rev-parse --show-toplevel)
root=$(canonical_path "$root")
g() { git -C "$root" "$@"; }
g check-ref-format --branch "$branch" >/dev/null
g remote get-url "$remote" >/dev/null || die "remote does not exist: $remote"
case $path in \~) path=$HOME ;; \~/*) path=$HOME/${path#\~/} ;; esac
[[ $path == /* ]] || path=$root/$path
# Ignore trailing separators and dot components when checking the leaf.
# Bash -L otherwise follows alias/ whereas Path-style inputs normalize it.
while [[ $path != / ]]; do
    case $path in
        */) path=${path%/} ;;
        */.) path=${path%/.} ;;
        *) break ;;
    esac
done
[[ -n $path ]] || path=/
[[ ! -L $path ]] || die "target path is a symlink: $path"
target=$(canonical_path "$path")
[[ $target != "$root" ]] || die 'planning worktree path may not be the implementation checkout'

common_dir() {
    local directory
    directory=$(git -C "$1" rev-parse --git-common-dir) || die "cannot inspect Git directory: $1"
    [[ $directory == /* ]] || directory=$1/$directory
    canonical_path "$directory"
}
common=$(common_dir "$root")
for administration in "$root/.git" "$common"; do
    administration=$(canonical_path "$administration")
    if overlaps "$target" "$administration"; then
        die 'planning worktree path overlaps Git administration data'
    fi
done

records=""
tracked=""
trap 'rm -f -- "$records" "$tracked"' EXIT
records=$(mktemp "${TMPDIR:-/tmp}/planning-worktrees.XXXXXX")
tracked=$(mktemp "${TMPDIR:-/tmp}/planning-tracked.XXXXXX")
g ls-files -z > "$tracked"
while IFS= read -r -d '' name; do
    resolved=$(canonical_path "$root/$name")
    if overlaps "$target" "$resolved"; then
        die 'planning worktree path overlaps tracked implementation content'
    fi
done < "$tracked"

# Git's NUL-delimited porcelain keeps spaces and embedded newlines intact.
load_worktrees() {
    local field
    wt_paths=(); wt_branches=(); wt_locked=(); wt_prunable=(); wt_count=0
    g worktree list --porcelain -z > "$records"
    while IFS= read -r -d '' field; do
        case $field in
            'worktree '*)
                wt_paths[wt_count]=$(canonical_path "${field#worktree }")
                wt_branches[wt_count]=""
                wt_locked[wt_count]=0
                wt_prunable[wt_count]=0
                wt_count=$((wt_count + 1)) ;;
            'branch '*) wt_branches[wt_count - 1]=${field#branch } ;;
            locked*) wt_locked[wt_count - 1]=1 ;;
            prunable*) wt_prunable[wt_count - 1]=1 ;;
        esac
    done < "$records"
    [[ $wt_count -gt 0 ]] || die 'Git returned no registered worktrees'
}
load_worktrees
existing=0
target_index=-1
for ((i=0; i<wt_count; i++)); do
    if [[ ${wt_paths[$i]} == "$target" ]]; then target_index=$i; fi
    if [[ ${wt_branches[$i]} == "refs/heads/$branch" ]]; then
        [[ ${wt_paths[$i]} == "$target" ]] || die "planning branch is already checked out elsewhere: ${wt_paths[$i]}"
        existing=1
    fi
done
if [[ -e $target && $target_index -lt 0 ]]; then
    die "target path already exists and is not a registered worktree: $target"
fi

ensure_registered() {
    local found=-1 index status target_common
    load_worktrees
    for ((index=0; index<wt_count; index++)); do
        if [[ ${wt_paths[$index]} == "$target" ]]; then found=$index; break; fi
    done
    [[ $found -ge 0 ]] || die "planning path is not registered as a Git worktree: $target"
    [[ ${wt_locked[$found]} == 0 && ${wt_prunable[$found]} == 0 ]] || die 'planning worktree is locked or prunable'
    [[ ${wt_branches[$found]} == "refs/heads/$branch" ]] || die 'worktree at target is on the wrong branch'
    target_common=$(common_dir "$target")
    [[ $target_common == "$common" ]] || die 'target belongs to a different Git repository'
    status=$(git -C "$target" status --porcelain --untracked-files=all)
    [[ -z $status ]] || die "planning worktree is dirty; preserve and clean it manually: $target"
}

local_oid=
if g show-ref --verify --quiet "refs/heads/$branch"; then
    local_oid=$(g rev-parse "refs/heads/$branch")
else
    status=$?
    [[ $status -eq 1 ]] || die "could not inspect local planning branch (exit $status)"
fi
remote_exists=0
if remote_output=$(g ls-remote --exit-code --heads "$remote" "refs/heads/$branch"); then
    [[ -n $remote_output ]] || die 'remote query succeeded without returning its branch'
    remote_exists=1
else
    status=$?
    [[ $status -eq 2 ]] || die "could not query remote branch (exit $status); refusing to treat this as missing"
fi
if [[ $remote_exists -eq 1 ]]; then
    g fetch "$remote" "refs/heads/$branch:refs/remotes/$remote/$branch"
    remote_oid=$(g rev-parse "refs/remotes/$remote/$branch")
fi
if [[ $existing -eq 1 ]]; then
    ensure_registered
elif [[ -e $target ]]; then
    die "target path is occupied: $target"
fi

ancestor() {
    local status
    if g merge-base --is-ancestor "$1" "$2"; then return 0; else status=$?; fi
    [[ $status -eq 1 ]] || die "could not compare planning history (exit $status)"
    return 1
}

if [[ $remote_exists -eq 1 && -z $local_oid ]]; then
    g branch --track "$branch" "$remote/$branch"
    g worktree add "$target" "$branch"
    ensure_registered
    printf 'attached planning worktree at %s from %s/%s\n' "$target" "$remote" "$branch"
elif [[ -n $local_oid && $remote_exists -eq 1 ]]; then
    added=0
    if ancestor "$local_oid" "$remote_oid"; then
        if [[ $local_oid != "$remote_oid" ]]; then
            if [[ $existing -eq 0 ]]; then g worktree add "$target" "$branch"; added=1; fi
            git -C "$target" merge --ff-only "$remote_oid"
        fi
    elif ! ancestor "$remote_oid" "$local_oid"; then
        die 'local and remote planning branches have diverged; reconcile manually'
    fi
    if [[ $existing -eq 0 && $added -eq 0 ]]; then g worktree add "$target" "$branch"; fi
    ensure_registered
    printf 'planning worktree ready at %s (local branch preserved)\n' "$target"
elif [[ -n $local_oid ]]; then
    if [[ $existing -eq 0 ]]; then g worktree add "$target" "$branch"; fi
    ensure_registered
    g push -u "$remote" "refs/heads/$branch:refs/heads/$branch"
    printf 'attached and pushed existing local planning branch at %s\n' "$target"
else
    g worktree add --detach --no-checkout "$target" HEAD
    git -C "$target" switch --orphan "$branch"
    git -C "$target" commit --allow-empty -m 'Initialize planning worktree' \
        -m $'Co-authored-by: Codex <noreply@openai.com>\nCo-authored-by: Billo AI <ai-engineering@billo.systems>'
    commit=$(git -C "$target" rev-parse HEAD)
    parents=$(git -C "$target" rev-list --parents -n 1 "$commit")
    tree=$(git -C "$target" rev-parse "$commit^{tree}")
    empty_tree=$(printf '' | g hash-object -t tree --stdin)
    ensure_registered
    [[ $parents == "$commit" && $tree == "$empty_tree" ]] || die 'new planning root is not a zero-parent, empty-tree commit; preserving worktree for inspection'
    g push -u "$remote" "refs/heads/$branch:refs/heads/$branch"
    printf 'created empty planning root and pushed %s/%s at %s\n' "$remote" "$branch" "$target"
fi
