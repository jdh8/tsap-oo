#!/bin/sh
# A/B the working tree's HeuristicBot against a revision's on paired deals.
#
#   ab/run.sh [deals] [control]
#
# `deals` defaults to 200000 and `control` to HEAD.  SEED_BASE defaults to
# the clock, fresh for each experiment; set it to replay one.  RESHUFFLE
# (default 8) is how often the pile is reshuffled where the two part, 0 for
# the plain paired estimate.  PLAYERS (default 4) is the table's size and
# THRESHOLD (default 10) the 胡 to win.
# Declare the deal count before the data and do not rebuild while this runs
# (CLAUDE.md).
set -eu
cd "$(dirname "$0")"
deals=${1:-200000}
control=${2:-HEAD}
seed=${SEED_BASE:-$(date +%s)}

rm -rf target/base
mkdir -p target/base
# -m: stamp the files now.  An archive carries its commit's time, and cargo
# would take an older control for the one it already built.
git -C .. archive "$control" | tar -xm -C target/base
# Two packages of one name and version cannot share a lockfile.
perl -pi -e '$done ||= s/^name = "tsap-oo"$/name = "tsap-oo-base"/' target/base/Cargo.toml

cargo build --release
echo "control $(git rev-parse --short "$control")"
exec target/release/tsap-oo-ab "$seed" "$deals"
