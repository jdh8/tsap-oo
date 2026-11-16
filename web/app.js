// Thin renderer over the wasm engine.  The engine holds the whole game; JS
// asks it for one bot step at a time on a timer so each move can be followed,
// and sends the human's clicks straight through.  Every call returns the
// full snapshot and the page is rebuilt from it.
import init, { WebGame } from './pkg/tsap_oo_web.js';

const PACE_MS = 550; // pause between bot steps, so they can be followed

// Seats are numbers in turn order.  Play runs counterclockwise, so counting
// on from you, the next player (下家) sits to your right.
const SLOTS = [null, 'right', 'top', 'left'];

let game;
let state;
let timer;
let logged = 0; // log lines already rendered

const id = (x) => document.getElementById(x);

// Settings are whatever the form in #settings holds: `{name: value}` per
// control, kept in localStorage and handed to the engine as JSON, which
// defaults anything missing or unreadable.  A new setting is one control
// there and one field of `Settings` in src/lib.rs.
const SETTINGS_KEY = 'tsap-oo-settings'; // github.io shares its storage across pages
let settings = {};
try {
  settings = JSON.parse(localStorage.getItem(SETTINGS_KEY)) ?? {};
} catch {} // no storage or junk in it: the defaults

function openSettings() {
  const form = id('settings').querySelector('form');
  for (const [name, value] of Object.entries(settings)) {
    if (form.elements[name]) form.elements[name].value = value;
  }
  id('settings').showModal();
}

function applySettings(e) {
  // ponytail: every setting is a number; parse per control when one is not
  settings = Object.fromEntries([...new FormData(e.target)].map(([k, v]) => [k, Number(v)]));
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {} // still applies to this visit
  newGame();
}

async function main() {
  await init();
  id('new').onclick = newGame;
  const toggleLog = () => {
    document.body.classList.toggle('log-hidden');
    id('logtoggle').textContent =
      document.body.classList.contains('log-hidden') ? '顯示紀錄' : '隱藏紀錄';
  };
  id('logtoggle').onclick = toggleLog;
  id('logtoggle').textContent = '顯示紀錄';
  id('help').onclick = () => id('rules').showModal();
  id('config').onclick = openSettings;
  id('settings').querySelector('form').onsubmit = applySettings;
  document.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || document.querySelector('dialog[open]')) return;
    if (e.key === '?') id('rules').showModal();
    if (e.key === 's') openSettings();
    if (e.key === 'n') newGame();
    if (e.key === 'l') toggleLog();
    if (e.key === 'h') showHint();
  });
  newGame();
}

function newGame() {
  game = new WebGame(String(Math.floor(Math.random() * 2 ** 53)), JSON.stringify(settings));
  id('log').innerHTML = '<h2>紀錄</h2>';
  logged = 0;
  show(game.snapshot());
}

// Render a snapshot, then keep the bots moving unless it is your decision.
function show(json) {
  state = JSON.parse(json);
  render();
  clearTimeout(timer);
  if (state.phase !== 'finished' && state.actor !== state.human) {
    timer = setTimeout(() => show(game.step()), PACE_MS);
  }
}

// --- rendering ---------------------------------------------------------------

function cardEl(card, cls = '') {
  const el = document.createElement('div');
  el.className = `card ${card.color} ${cls}`.trim();
  el.textContent = card.glyph;
  el.dataset.code = card.code;
  return el;
}

function backEl() {
  const el = document.createElement('div');
  el.className = 'card back';
  return el;
}

// The 胡 in the cards you can see of a seat: its melds on the table, and the
// hand too where it shows (yours, and everyone's once the round is over).
// Yours also shows the 胡 to win.
function huEl(seat, goal) {
  const el = document.createElement('span');
  el.className = 'hu';
  el.textContent = goal ? `${seat.hu}/${goal} 胡` : `${seat.hu} 胡`;
  el.title = '看得到的牌目前的胡數';
  return el;
}

function meldEl(cards) {
  const el = document.createElement('div');
  el.className = 'meld';
  for (const c of cards) el.append(cardEl(c));
  return el;
}

function meldsEl(melds) {
  const el = document.createElement('div');
  el.className = 'melds';
  for (const m of melds) el.append(meldEl(m));
  return el;
}

function render() {
  const name = (seat) => state.seats[seat].name;
  const slot = (seat) => SLOTS[(seat - state.human + state.seats.length) % state.seats.length];
  const finished = state.phase === 'finished';

  // The threshold in force, in the help sheet.
  for (const el of document.querySelectorAll('.threshold')) el.textContent = state.threshold;

  // Header: scores, the dealer marked.
  const scores = id('scores');
  scores.replaceChildren();
  state.scores.forEach((score, seat) => {
    const span = document.createElement('span');
    span.className = (seat === state.dealer ? 'dealer ' : '') + (seat === state.human ? 'me' : '');
    span.textContent = `${name(seat)} ${score}`;
    scores.append(span);
  });

  // Opponents.
  state.seats.forEach((s, seat) => {
    if (seat === state.human) return;
    const box = id(slot(seat));
    box.replaceChildren();
    const label = document.createElement('div');
    label.className = 'name' + (state.actor === seat ? ' acting' : '');
    label.textContent = name(seat);
    if (seat === state.dealer) label.insertAdjacentHTML('beforeend', '<span class="dealer">莊</span>');
    label.append(huEl(s));
    box.append(label);
    const fan = document.createElement('div');
    fan.className = 'fan';
    if (finished && s.hand) for (const c of s.hand.flat()) fan.append(cardEl(c));
    else for (let i = 0; i < s.concealed; i++) fan.append(backEl());
    box.append(fan);
    if (s.exposed.length) box.append(meldsEl(s.exposed));
  });

  // The table: the card on offer, the discards, the pile.
  const offer = id('offer');
  offer.replaceChildren();
  if (state.offer) {
    const who = document.createElement('span');
    who.className = 'who';
    who.textContent = `${name(state.offer.source)}${state.offer.drawn ? '摸' : '打'}`;
    offer.append(who, cardEl(state.offer.card));
  }
  const discards = id('discards');
  discards.replaceChildren();
  for (const d of state.discards) discards.append(cardEl(d.card));
  id('pile').textContent = `牌堆 ${state.pile} 張（尾 6 張廢牌）`;

  // You.
  const me = state.seats[state.human];
  const status = id('status');
  status.className = 'status';
  if (finished) {
    status.textContent = '本局結束';
  } else if (state.actor === state.human) {
    status.classList.add('acting');
    status.textContent =
      state.phase !== 'claim' ? '請打一張牌' : state.options?.forced ? '將帥一定要吃' : '要吃碰嗎？';
  } else {
    status.textContent = `等${name(state.actor)}行動…`;
  }
  status.append(huEl(me, state.threshold));
  const exposed = id('exposed');
  exposed.replaceChildren();
  if (me.exposed.length) exposed.append(meldsEl(me.exposed));
  const hand = id('hand');
  hand.replaceChildren();
  hand.style.setProperty('--n', me.hand.flat().length);
  const myDiscard = !finished && state.phase === 'discard' && state.actor === state.human;
  // Grouped by the engine: the best melds, then the loose cards with the
  // lone generals among them.
  for (const group of me.hand) {
    const box = document.createElement('div');
    box.className = 'group';
    for (const c of group) {
      const locked = state.locked.includes(c.code);
      const el = cardEl(c, locked ? 'locked' : myDiscard ? 'clickable' : '');
      if (myDiscard && !locked) el.onclick = () => show(game.discard(c.code));
      box.append(el);
    }
    hand.append(box);
  }

  // Your waits (聽牌), each card that wins with the 胡 it wins at and the
  // copies you have not seen.  While you discard, a card whose discard
  // leaves you waiting is marked 聽, and pointing at it shows those waits;
  // the row keeps its room meanwhile so the buttons below stay put.
  const waits = id('waits');
  const showWaits = (rows) => {
    waits.replaceChildren();
    if (!rows.length) return;
    waits.append('聽');
    for (const w of rows) {
      const item = document.createElement('div');
      item.className = w.left ? 'wait' : 'wait gone';
      const note = document.createElement('span');
      note.innerHTML = `${w.hu} 胡<br>${w.left} 張`;
      item.append(cardEl(w.card), note);
      waits.append(item);
    }
  };
  showWaits(state.waits);
  waits.classList.toggle('reserve', state.discard_waits.length > 0);
  for (const { code, waits: rows } of state.discard_waits) {
    for (const el of hand.querySelectorAll(`[data-code="${code}"]`)) {
      el.classList.add('tenpai');
      el.onmouseenter = () => showWaits(rows);
      el.onmouseleave = () => showWaits([]);
    }
  }

  // Claim buttons.
  const actions = id('actions');
  actions.replaceChildren();
  if (state.options) {
    // As in Tenhou, a button shows only the cards leaving your hand: the
    // meld minus the offered card.
    const offered = state.offer.card;
    const held = (meld) => {
      const rest = [...meld];
      rest.splice(rest.findIndex((c) => c.code === offered.code), 1);
      return rest;
    };
    const claim = (label, cards, onclick, key) => {
      const b = button(label, onclick, 'claim', key);
      for (const c of cards) b.append(cardEl(c));
      // Claims the rules forbid, shown so the reason is visible.
      if (!onclick) {
        b.disabled = true;
        b.title = '吃碰後會沒有牌可打（將帥不能打），所以不行';
      }
      actions.append(b);
    };
    const pair = [offered, offered];
    if (state.options.pong) claim('碰', pair, () => show(game.pong()), 'pong');
    state.options.eats.forEach((meld, i) => claim('吃', held(meld), () => show(game.eat(i)), `eat${i}`));
    if (state.options.blocked_pong) claim('碰', pair, null);
    for (const meld of state.options.blocked_eats) claim('吃', held(meld), null);
    // A drawn general must be eaten: alone (a bare 吃) or with the 士象.
    if (!state.options.forced) actions.append(button('過', () => show(game.pass()), '', 'pass'));
  }
  if (!finished && state.actor === state.human) {
    const b = button('提示', showHint);
    b.title = '電腦會怎麼做（h）';
    actions.append(b);
  }

  // The log grows by whatever happened since the last render.
  const log = id('log');
  for (; logged < state.log.length; logged++) {
    const line = document.createElement('div');
    line.textContent = state.log[logged];
    log.append(line);
  }
  log.scrollTop = log.scrollHeight;

  // The result sheet.
  const result = id('result');
  result.hidden = !finished;
  if (finished) {
    result.replaceChildren();
    const h2 = document.createElement('h2');
    const r = state.result;
    if (r.winner != null) {
      const how = r.from != null ? `${name(r.from)}${r.winning_card ? '放' : ''}` : r.winning_card ? '自摸' : '天胡';
      h2.textContent = `${name(r.winner)}胡牌，${r.hu} 胡（${how}）`;
      result.append(h2, meldsEl(r.exposed), meldsEl(r.concealed));
    } else if (r.foul != null) {
      h2.textContent = `${name(r.foul)}相公：沒有牌可打，賠每家 ${state.threshold} 胡`;
      result.append(h2);
    } else {
      h2.textContent = '流局';
      result.append(h2);
    }
    result.append(button('下一局', () => show(game.next_round())));
  }
}

// Ring what a bot would do here, a card in your hand or a claim button, and
// mark every move with how far it grades behind that pick.  The marks last
// until the next render, i.e. for this one decision.  Of several copies of
// the pick it rings the last: the loose cards are the last group, so a loose
// copy is ringed before one sitting in a meld.
function showHint() {
  const rows = JSON.parse(game.hint());
  if (!rows.length || document.querySelector('.grade')) return;
  const els = (code) => [
    ...document.querySelectorAll(
      `#hand [data-code="${code}"]:not(.locked), #actions [data-hint="${code}"]`,
    ),
  ];
  els(rows[0].code).at(-1)?.classList.add('hint');
  for (const { code, behind } of rows) {
    for (const el of els(code)) {
      const mark = document.createElement('span');
      mark.className = 'grade';
      // A priority without a unit: it orders the moves and prices nothing.
      mark.textContent = behind ?? '死';
      mark.title =
        behind == null ? '走了這步，手牌再也湊不到胡牌要的胡數' : `電腦把這步排在首選後面 ${behind}（只是排序用的分數，沒有單位）`;
      el.append(mark);
    }
  }
}

function button(label, onclick, cls = '', hint = '') {
  const b = document.createElement('button');
  b.className = `action ${cls}`.trim();
  if (hint) b.dataset.hint = hint;
  b.textContent = label;
  b.onclick = onclick;
  return b;
}

main();
