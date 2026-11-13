// Thin renderer over the wasm engine.  The engine holds the whole game; JS
// asks it for one bot step at a time on a timer so each move can be followed,
// and sends the human's clicks straight through.  Every call returns the
// full snapshot and the page is rebuilt from it.
import init, { WebGame } from './pkg/tsap_oo_web.js';

const PACE_MS = 550; // pause between bot steps, so they can be followed

// Seats run counterclockwise, so the next player (下家) sits to your right.
const SEATS = ['E', 'S', 'W', 'N'];
const NAMES = { E: '東', S: '南', W: '西', N: '北' };
const RELATION = { E: '你', S: '下家', W: '對家', N: '上家' };
const SLOT = { S: 'south', W: 'west', N: 'north' };

let game;
let state;
let timer;
let logged = 0; // log lines already rendered

const id = (x) => document.getElementById(x);

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
  document.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || id('rules').open) return;
    if (e.key === '?') id('rules').showModal();
    if (e.key === 'n') newGame();
    if (e.key === 'l') toggleLog();
    if (e.key === 'h') showHint();
  });
  newGame();
}

function newGame() {
  game = new WebGame(String(Math.floor(Math.random() * 2 ** 53)));
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
  const bySeat = Object.fromEntries(state.seats.map((s) => [s.seat, s]));
  const finished = state.phase === 'finished';

  // Header: scores, the dealer marked.
  const scores = id('scores');
  scores.replaceChildren();
  SEATS.forEach((seat, i) => {
    const span = document.createElement('span');
    span.className = (seat === state.dealer ? 'dealer ' : '') + (seat === state.human ? 'me' : '');
    span.textContent = `${NAMES[seat]}${seat === state.human ? '(你)' : ''} ${state.scores[i]}`;
    scores.append(span);
  });

  // Opponents.
  for (const seat of SEATS) {
    if (seat === state.human) continue;
    const s = bySeat[seat];
    const box = id(SLOT[seat]);
    box.replaceChildren();
    const name = document.createElement('div');
    name.className = 'name' + (state.actor === seat ? ' acting' : '');
    name.textContent = `${NAMES[seat]}・${RELATION[seat]}`;
    if (seat === state.dealer) name.insertAdjacentHTML('beforeend', '<span class="dealer">莊</span>');
    box.append(name);
    const fan = document.createElement('div');
    fan.className = 'fan';
    if (finished && s.hand) for (const c of s.hand.flat()) fan.append(cardEl(c));
    else for (let i = 0; i < s.concealed; i++) fan.append(backEl());
    box.append(fan);
    if (s.exposed.length) box.append(meldsEl(s.exposed));
  }

  // The table: the card on offer, the discards, the pile.
  const offer = id('offer');
  offer.replaceChildren();
  if (state.offer) {
    const who = document.createElement('span');
    who.className = 'who';
    who.textContent = `${NAMES[state.offer.source]}${state.offer.drawn ? '摸' : '打'}`;
    offer.append(who, cardEl(state.offer.card));
  }
  const discards = id('discards');
  discards.replaceChildren();
  for (const d of state.discards) discards.append(cardEl(d.card));
  id('pile').textContent = `牌堆 ${state.pile} 張（尾 6 張廢牌）`;

  // You.
  const me = bySeat[state.human];
  const status = id('status');
  status.className = 'status';
  if (finished) {
    status.textContent = '本局結束';
  } else if (state.actor === state.human) {
    status.classList.add('acting');
    status.textContent = state.phase === 'claim' ? '要吃碰嗎？' : '請打一張牌';
  } else {
    status.textContent = `等 ${NAMES[state.actor]} 行動…`;
  }
  const exposed = id('exposed');
  exposed.replaceChildren();
  if (me.exposed.length) exposed.append(meldsEl(me.exposed));
  const hand = id('hand');
  hand.replaceChildren();
  const myDiscard = !finished && state.phase === 'discard' && state.actor === state.human;
  // Grouped by the engine: the best melds, then the loose cards.
  for (const group of me.hand) {
    const box = document.createElement('div');
    box.className = 'group';
    for (const c of group) {
      const locked = c.code === state.locked;
      const el = cardEl(c, locked ? 'locked' : myDiscard ? 'clickable' : '');
      if (myDiscard && !locked) el.onclick = () => show(game.discard(c.code));
      box.append(el);
    }
    hand.append(box);
  }

  // Claim buttons.
  const actions = id('actions');
  actions.replaceChildren();
  if (state.options) {
    if (state.options.pong) actions.append(button('碰', () => show(game.pong()), '', 'pong'));
    state.options.eats.forEach((cards, i) => {
      const b = button('吃', () => show(game.eat(i)), '', `eat${i}`);
      for (const c of cards) b.append(cardEl(c));
      actions.append(b);
    });
    actions.append(button('過', () => show(game.pass()), 'pass', 'pass'));
  }
  if (!finished && state.actor === state.human) {
    const b = button('提示', showHint, 'pass');
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
    if (r.winner) {
      const how = r.from ? `${NAMES[r.from]}${r.winning_card ? '放' : ''}` : r.winning_card ? '自摸' : '天胡';
      h2.textContent = `${NAMES[r.winner]}${r.winner === state.human ? '（你）' : ''}胡牌，${r.hu} 胡（${how}）`;
      result.append(h2, meldsEl(r.layout));
    } else {
      h2.textContent = '流局';
      result.append(h2);
    }
    result.append(button('下一局', () => show(game.next_round())));
  }
}

// Ring what a bot would do here: a card in your hand or a claim button.  The
// ring lasts until the next render, i.e. for this one decision.
function showHint() {
  const hint = game.hint();
  if (!hint) return;
  document
    .querySelector(`#hand [data-code="${hint}"]:not(.locked), #actions [data-hint="${hint}"]`)
    ?.classList.add('hint');
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
