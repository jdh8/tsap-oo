#!/bin/sh
# The queue's power switch on a Mac: how much of the machine pueue may use.
#
#   ab/cores.sh 0    stop: pause the queue and the run in flight
#   ab/cores.sh 1    E-cores only: cooler and quieter, not cheaper
#   ab/cores.sh 2    every core
#
# 1 puts pueued and everything under it in background QoS, which keeps a
# process on Apple silicon's E-cores (on an M4, 4 P + 6 E, a run took 5.5
# times as long).  Children inherit it, so tasks that start later run the
# same way; 2 lifts it again.  A paused queue stays paused across a restart
# of pueued, but the level does not: a fresh pueued uses every core.
# The verdict is the same at any level: the deals come from the seed.
set -eu

# Set a process before listing its children: one spawned in between inherits.
apply() {
	for p; do
		taskpolicy "$flag" -p "$p" 2>/dev/null || :
		# shellcheck disable=SC2046
		apply $(pgrep -P "$p" || :)
	done
}

case ${1-} in
0) pueue pause ;;
1 | 2)
	flag=-b
	[ "$1" = 1 ] || flag=-B
	# shellcheck disable=SC2046
	apply $(pgrep -x pueued)
	pueue start
	;;
*)
	sed -n '2,/^set -eu/p' "$0" | sed 's/^# \{0,1\}//;/^set -eu/d' >&2
	exit 2
	;;
esac
