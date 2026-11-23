#!/usr/bin/env python3
"""Tables from `examples/overspend.rs`: python3 research/overspend.py pairs.csv waiting.csv

A hand is fully melded when it wins on every general not yet seen: its twenty
cards are all in melds, pairs and lone generals.  Its other waits extend a
concealed meld: a pong, a kong, a fourth colour of soldiers.
"""
import csv
import math
import sys
from collections import defaultdict

TEXT = ("played", "hand", "exposed", "offer")


def read(path):
    return [{k: v if k in TEXT else float(v) for k, v in r.items()} for r in csv.DictReader(open(path))]


def rate(rows):
    """Wins among `rows`, with the standard error, in percent"""
    if not rows:
        return "-"
    n, p = len(rows), sum(r["won"] for r in rows) / len(rows)
    return f"{100 * p:.0f}% ± {100 * math.sqrt(p * (1 - p) / n):.0f}"


def table(title, head, key, rows, cells):
    groups = defaultdict(list)
    for r in rows:
        groups[key(r)].append(r)
    print(f"\n{title}\n\n| | " + " | ".join(head) + " |\n|---|" + "---:|" * len(head))
    for k in sorted(groups):
        print(f"| {k} | " + " | ".join(str(c) for c in cells(groups[k])) + " |")


pairs, waiting = read(sys.argv[1]), read(sys.argv[2])
T = 10
ate = lambda rows: [r for r in rows if r["played"] == "pair"]
gain = lambda rows: sum(r["v_pair"] - max(r["v_pass"], r["v_other"]) for r in rows) / len(rows)


def group(r):
    if r["c1a"] < T:
        return "A short even with one more general drawn"
    if r["c0a"] >= T:
        return "C within the limit"
    return "B1 overspent: one short only after eating" if r["c0b"] >= T else "B2 one short already before eating"


def passing(r):
    if r["pass_g"] == r["kings"]:
        return "0 Already fully melded"
    if r["pass_o"] > 0:
        return f"2 搭子 waits on {min(int(r['pass_o']), 5)}" + (" or more" if r["pass_o"] >= 5 else "")
    return "1 士象 waiting for its own general" if r["pass_g"] > 0 else "3 Not waiting"


decisions = (
    ["Decisions", "Eaten", "Engine's estimate, eat − pass (points)", "Won after eating", "Won after passing"],
    lambda c: [len(c), len(ate(c)), f"{gain(c):+.2f}", rate(ate(c)), rate([r for r in c if r["played"] != "pair"])],
)
print(f"{len(pairs)} pair decisions, {len(waiting)} waiting hands")
table("Decisions where a pair can be eaten, by the ceiling after eating", decisions[0], group, pairs, decisions[1])
full = [r for r in pairs if r["kings"] > 0 and r["eat_g"] == r["kings"]]
table("Those fully melded after eating, by ceiling and extension", decisions[0],
      lambda r: group(r)[:2].strip() + (", with extensions to wait on" if r["eat_o"] > 0 else ", extends 0"), full, decisions[1])
table("Those fully melded after eating that extend 0, by what passing waits on", decisions[0], passing,
      [r for r in full if r["eat_o"] == 0], decisions[1])


def wait(r):
    if r["kings"] > 0 and r["generals"] == r["kings"]:
        return f"0 Fully melded, extends {min(int(r['others']), 4)}" + (" or more" if r["others"] >= 4 else "")
    if r["generals"] > 0:
        return "2 士象 waiting for its own general"
    return f"1 搭子, waits on {min(int(r['others']), 5)}" + (" or more" if r["others"] >= 5 else "")


# The DP's chance is of the hands it plays whole, 14 cards or fewer
valued = [r for r in waiting if not math.isnan(r["chance"])]
bands = (("15 draws or more", 15, 99), ("8–14 draws", 8, 14), ("0–7 draws", 0, 7))
within = lambda c, lo, hi: [r for r in c if lo <= r["draws"] <= hi]
chance = lambda c: f"{100 * sum(r['chance'] for r in c) / len(c):.0f}%" if c else "-"
table("Waiting hands: actual win rate (engine's estimate, hands)", [b[0] for b in bands], wait, valued,
      lambda c: [f"{rate(w)} ({chance(w)}, {len(w)})" for _, lo, hi in bands for w in [within(c, lo, hi)]])
