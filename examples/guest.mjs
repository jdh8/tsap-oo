// The other end of `guest.rs`: chunhua1972/four-color-game's own
// `chooseAiCommand`, imported from a clone and asked one decision a line.
//
//   node guest.mjs <path to the clone>
//
// A request is space-separated fields, cards as comma-separated type ids
// (theirs are susik's `Card::index`):
//
//   d <hand> <discardable>               -> the type to discard
//   c <hand> <offer> <forced> <claims>   -> the claim's position, or -1 to pass
//
// where <claims> is `;`-separated, each `e` (eat) or `p` (pong) and the
// meld's types in ascending order.  The options are this engine's; only the
// choice among them is theirs.
import { resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { pathToFileURL } from 'node:url';

const core = await import(
  pathToFileURL(resolve(process.argv[2], 'packages/game-core/src/index.ts'))
);
const types = (s) => (s ? s.split(',').map(Number) : []);

/** Their physical ids: a type's copies are 4t..4t+3, a sorted hand's the first ones */
function physical(hand) {
  const seen = new Map();
  return hand.map((t) => {
    const k = seen.get(t) ?? 0;
    seen.set(t, k + 1);
    return 4 * t + k;
  });
}

function choose(ownHand, legalOptions) {
  // `publicHu` is added to every claim and to the pass alike, so any value
  // gives the same choice; the other public fields are never read.
  const obs = { seat: 0, ownHand, public: { seats: [{ publicHu: 0 }] }, legalOptions };
  return core.chooseAiCommand(obs, 'normal', () => {
    throw new Error('normal difficulty draws no random numbers');
  });
}

function discard(hand, allowed) {
  const ids = physical(hand);
  // Their snapshot lists every card of the hand in order; the policy keeps
  // the first of each type.
  const options = allowed.map((t) => {
    const id = ids[hand.indexOf(t)];
    return { id: `discard:${id}`, command: { type: 'discard', tileId: id }, previewTileIds: [id] };
  });
  return Math.floor(choose(ids, options).tileId / 4);
}

function claim(hand, offer, forced, claims) {
  const ids = physical(hand);
  const offered = 4 * offer + 3; // the hand holds at most three copies
  const options = claims.map((text, n) => {
    const action = text[0] === 'p' ? 'pong' : 'eat';
    const meld = types(text.slice(1));
    // Their patterns have no pair of soldiers; this engine may offer one.
    const meldKind = core.classifyMeld(meld) ?? 'pair';
    const needed = [...meld];
    needed.splice(needed.indexOf(offer), 1);
    const pool = [...ids];
    const handTileIds = needed.map(
      (t) => pool.splice(pool.findIndex((id) => Math.floor(id / 4) === t), 1)[0],
    );
    const key = `${meldKind}:${meld.join(',')}`;
    return {
      n,
      key,
      id: `${action}:${key}`,
      command: {
        type: 'respond',
        windowId: 'guest',
        intent: { kind: 'claim', action, meldKind, handTileIds },
        responseRevision: 0,
      },
      previewTileIds: [offered, ...handTileIds],
    };
  });
  // `enumerateLegalClaims` walks the patterns in key order, then the pass.
  options.sort((a, b) => a.key.localeCompare(b.key));
  if (!forced)
    options.push({
      n: -1,
      id: 'pass',
      command: { type: 'respond', windowId: 'guest', intent: { kind: 'pass' }, responseRevision: 0 },
      previewTileIds: [],
    });
  const command = choose(ids, options);
  return options.find((o) => o.command === command).n;
}

for await (const line of createInterface({ input: process.stdin })) {
  const f = line.split(' ');
  const reply =
    f[0] === 'd'
      ? discard(types(f[1]), types(f[2]))
      : claim(types(f[1]), Number(f[2]), f[3] === '1', f[4].split(';'));
  process.stdout.write(`${reply}\n`);
}
