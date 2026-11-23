#!/usr/bin/env bash
# Queue an A/B behind the others on this machine.
#
#   [ENV=...] ab/queue.sh SLUG [deals] [control]
#
# A run takes every core, so they go one at a time through pueue (`brew
# install pueue && brew services start pueue`; its default group is one task
# wide).  This freezes the checkout as it is now, uncommitted and untracked
# files included, into a detached worktree and queues `ab/run.sh` there.
# Keep editing: the run builds and measures the frozen copy when its turn
# comes, so nothing is rebuilt under a run in flight.
#
#   worktree  ${AB_WORKTREES:-~/ab}/tsap-oo-SLUG, with its own ab/target
#   control   resolved now (default HEAD), not when the task starts
#   env       captured now by pueue (SEED_BASE, RESHUFFLE, PLAYERS, ...);
#             an unset SEED_BASE is still the clock when the task starts
#
#   pueue status             the queue and each task's exit status
#   pueue log ID --full      the run's output: seed base, control, verdict
#   pueue kill ID / remove ID
#   git worktree remove --force ~/ab/tsap-oo-SLUG    once the verdict is read
set -euo pipefail

if [[ $# -lt 1 || $1 == -h || $1 == --help ]]; then
	sed -n '2,/^set -euo/p' "$0" | sed 's/^# \{0,1\}//;/^set -euo/d'
	exit 0
fi
slug=$1
deals=${2:-200000}

cd "$(git rev-parse --show-toplevel)"
# In the worktree HEAD is the snapshot, so the control is named by its SHA.
control=$(git rev-parse --verify "${3:-HEAD}^{commit}")
wt=${AB_WORKTREES:-$HOME/ab}/tsap-oo-$slug

# Snapshot the working tree through a scratch index: the real index, the
# branch and the files are untouched.
rev=$(git rev-parse HEAD)
if [[ -n $(git status --porcelain) ]]; then
	idx=$(mktemp)
	trap 'rm -f "$idx"' EXIT
	cp "$(git rev-parse --git-path index)" "$idx"
	GIT_INDEX_FILE=$idx git add -A
	tree=$(GIT_INDEX_FILE=$idx git write-tree)
	rev=$(git commit-tree --no-gpg-sign "$tree" -p "$rev" -m "ab/queue.sh: $slug")
fi

# Refuses an existing worktree: remove it, or `pueue restart -i` its task.
git worktree add --detach "$wt" "$rev" >&2

# Lowest priority, every core.  Never `taskpolicy -b`: background QoS keeps
# a process on Apple silicon's E-cores, and on an M4 (4 P + 6 E) the run
# took 5.5 times as long; `nice` cost nothing measurable.  Linux has the
# politer SCHED_IDLE.
idle=(nice -n19)
if command -v chrt >/dev/null; then idle=(chrt --idle 0); fi

printf -v cmd '%q ' "${idle[@]}" ab/run.sh "$deals" "$control"
pueue add --label "$slug" --working-directory "$wt" -- "$cmd"
