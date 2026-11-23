// The other end of `away.rs`: chunhua1972/four-color-game's own engine
// plays the rounds by its own rules, with its Normal AI (the host) in some
// seats and this crate's bot (the guest) in the others.
//
//   node away.mjs <path to the clone> <deals> <seed base>
//
// The guest's decisions are asked on stdout, one a line starting with `?`,
// and answered on stdin; every other line of stdout is the report.  Cards
// are type ids (theirs are susik's `Card::index`), lists comma-separated:
//
//   ? d <state>                                 -> the type to discard
//   ? c <state> <offer> <drawn> <origin> <claims> -> the claim's position, or -1 to pass
//   <state> = <seat> <dealer> <wall> <hand> <discards> <their seats> <melds>
//
// <melds> is the four seats' `|`-separated, a seat's melds `;`-separated;
// <claims> is `;`-separated, each `e` (eat) or `p` (pong) and the meld.
// A win and a kong are compulsory where the guest comes from, so it is
// never asked about them: it takes both.
import { resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { pathToFileURL } from 'node:url';

const [clone, deals, base] = [process.argv[2], Number(process.argv[3]), Number(process.argv[4])];
const load = (file) => import(pathToFileURL(resolve(clone, file)));
const core = await load('packages/game-core/src/index.ts');
const { seededRandom } = await load('scripts/random.ts');

const replies = createInterface({ input: process.stdin })[Symbol.asyncIterator]();
async function ask(request) {
  process.stdout.write(`? ${request}\n`);
  return Number((await replies.next()).value);
}

const type = (id) => Math.floor(id / 4);
const types = (ids) => ids.map(type).sort((a, b) => a - b);

/** The guest's command at `snap`; `origins` are the seats the discards came from */
async function guest(snap, origins) {
  const options = snap.legalOptions;
  const sure =
    options.find((o) => ['hu', 'opening_hu', 'draw'].includes(o.id)) ??
    options.find((o) => o.terminalConsequence === 'win') ??
    options.find((o) => o.command.intent?.action === 'kong');
  if (sure) return sure.command;
  const pub = snap.public;
  const state = [
    snap.viewerSeat,
    pub.dealerSeat,
    pub.wallCount,
    types(snap.hand),
    pub.discards.map(type),
    origins,
    pub.seats.map((seat) => seat.melds.map((m) => types(m.tileIds)).join(';')).join('|'),
  ].join(' ');
  if (options[0].command.type === 'discard') {
    const reply = await ask(`d ${state}`);
    return options.find((o) => type(o.command.tileId) === reply).command;
  }
  const pass = options.find((o) => o.id === 'pass');
  // A claim that leaves nothing to discard is a foul (相公) here and is
  // not allowed at home: the guest is not shown it.
  const claims = options.filter((o) => o !== pass && o.terminalConsequence !== 'xiang_gong');
  if (!claims.length) return (pass ?? options[0]).command;
  const offer = pub.offer;
  const melds = claims.map(
    (o) => (o.command.intent.action === 'pong' ? 'p' : 'e') + types(o.previewTileIds),
  );
  const reply = await ask(
    `c ${state} ${type(offer.tileId)} ${Number(offer.source === 'open_draw')} ` +
      `${offer.originSeat} ${melds.join(';')}`,
  );
  return (reply < 0 ? pass : claims[reply]).command;
}

/** Play `deck` out with `visitors[seat]` saying whose bot sits where; the result */
async function play(deck, dealer, visitors, id) {
  const controllers = visitors.map(() => ({ kind: 'ai', difficulty: 'normal' }));
  let now = 0;
  const ctx = () => ({ nowMs: (now += 10), id: `${id}:${now}` });
  let s = core.createInitialState(deck, dealer, core.makeRuleConfig(4), controllers, id, ctx());
  const origins = [];
  const act = async (seat) => {
    const snap = core.buildPlayerSnapshot(s, seat, now);
    const command = visitors[seat]
      ? await guest(snap, origins)
      : core.chooseAiCommand(
          { seat, ownHand: snap.hand, public: snap.public, legalOptions: snap.legalOptions },
          'normal',
          () => 0,
        );
    const result = core.applyGameCommand(s, seat, command, ctx());
    if (!result.ok) throw new Error(`${id}: ${JSON.stringify(result.error)}`);
    s = result.state;
  };
  while (!s.result) {
    if (!s.window) {
      await act(s.activeSeat);
      continue;
    }
    const origin = s.offer.originSeat;
    for (const seat of s.window.participantSeats) if (!s.responses[seat]) await act(seat);
    const result = core.resolveReactionWindow(s, s.window.id, ctx());
    if (!result.ok) throw new Error(`${id}: ${JSON.stringify(result.error)}`);
    if (result.state.discards.length > s.discards.length) origins.push(origin);
    s = result.state;
  }
  return s.result;
}

// Seat 1 in four fields: four hosts, the guest among hosts, four guests,
// the host among guests
const FIELDS = [
  [false, false, false, false],
  [true, false, false, false],
  [true, true, true, true],
  [false, true, true, true],
];
const wins = [0, 0, 0, 0];
const fouls = [0, 0, 0, 0];
const [away, home] = [[], []];
for (let i = 0; i < deals; i++) {
  const deck = core.shuffleDeck(core.createDeck(), seededRandom(base + i));
  const scores = [];
  for (const [f, visitors] of FIELDS.entries()) {
    const result = await play(deck, i % 4, visitors, `${base + i}:${f}`);
    wins[f] += result.winnerSeat === 0;
    fouls[f] += result.penaltySeat === 0;
    scores.push(result.scores[0].delta);
  }
  away.push(scores[1] - scores[0]);
  home.push(scores[3] - scores[2]);
}

function report(label, diffs) {
  const n = diffs.length;
  const mean = diffs.reduce((a, b) => a + b, 0) / n;
  const se = Math.sqrt(diffs.reduce((a, d) => a + (d - mean) ** 2, 0) / (n - 1) / n);
  const signed = (x) => (x < 0 ? '' : '+') + x.toFixed(3);
  console.log(
    `${label}: ${signed(mean)} ± ${se.toFixed(3)} points a deal, ` +
      `95% CI [${signed(mean - 1.96 * se)}, ${signed(mean + 1.96 * se)}]`,
  );
}
const rates = (counts) => counts.map((n) => `${((100 * n) / deals).toFixed(2)}%`).join(', ');
console.log(`seed base ${base}, ${deals} deals, their rules and scores`);
console.log('seat 1 as host among hosts, guest among hosts, guest among guests, host among guests:');
console.log(`  wins ${rates(wins)}`);
console.log(`  fouls ${rates(fouls)}`);
report('among three hosts, guest − host', away);
report('among three guests, host − guest', home);
process.exit(0);
