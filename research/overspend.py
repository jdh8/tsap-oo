#!/usr/bin/env python3
"""Tables from `examples/overspend.rs`: python3 research/overspend.py pairs.csv waiting.csv

A hand is 完成形 when it wins on every general not yet seen: its twenty
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
        return "甲 再摸一張將帥也不到"
    if r["c0a"] >= T:
        return "丙 上限以內"
    return "乙1 超支：吃了才欠一張" if r["c0b"] >= T else "乙2 吃之前就欠一張"


def passing(r):
    if r["pass_g"] == r["kings"]:
        return "0 已經是完成形"
    if r["pass_o"] > 0:
        return f"2 搭子聽 {min(int(r['pass_o']), 5)} 張" + ("以上" if r["pass_o"] >= 5 else "")
    return "1 士象等自己的將帥" if r["pass_g"] > 0 else "3 沒聽"


decisions = (
    ["決定", "吃了", "引擎估 吃 − 不吃（分）", "吃的局胡牌", "不吃的局胡牌"],
    lambda c: [len(c), len(ate(c)), f"{gain(c):+.2f}", rate(ate(c)), rate([r for r in c if r["played"] != "pair"])],
)
print(f"{len(pairs)} pair decisions, {len(waiting)} waiting hands")
table("可以吃對子的決定，依吃完的天花板", decisions[0], group, pairs, decisions[1])
full = [r for r in pairs if r["kings"] > 0 and r["eat_g"] == r["kings"]]
table("吃完是完成形的，依天花板與延伸", decisions[0],
      lambda r: group(r)[:2].strip() + ("，有延伸可聽" if r["eat_o"] > 0 else "，延伸 0 張"), full, decisions[1])
table("吃完是完成形、延伸 0 張的，依不吃時聽什麼", decisions[0], passing,
      [r for r in full if r["eat_o"] == 0], decisions[1])


def wait(r):
    if r["kings"] > 0 and r["generals"] == r["kings"]:
        return f"0 完成形，延伸 {min(int(r['others']), 4)} 張" + ("以上" if r["others"] >= 4 else "")
    if r["generals"] > 0:
        return "2 士象等自己的將帥"
    return f"1 搭子，聽 {min(int(r['others']), 5)} 張" + ("以上" if r["others"] >= 5 else "")


# The DP's chance is of the hands it plays whole, 14 cards or fewer
valued = [r for r in waiting if not math.isnan(r["chance"])]
bands = (("摸 15 以上", 15, 99), ("摸 8–14", 8, 14), ("摸 0–7", 0, 7))
within = lambda c, lo, hi: [r for r in c if lo <= r["draws"] <= hi]
chance = lambda c: f"{100 * sum(r['chance'] for r in c) / len(c):.0f}%" if c else "-"
table("聽牌的手：實際胡牌率（引擎估的，手數）", [b[0] for b in bands], wait, valued,
      lambda c: [f"{rate(w)}（{chance(w)}，{len(w)}）" for _, lo, hi in bands for w in [within(c, lo, hi)]])
