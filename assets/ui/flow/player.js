/*
  flow.html's player: it takes the handheld's buttons (keys, or the buttons under the screen),
  asks the screen on show what they do (screens.js), and slides from one screen to the next as the
  device will:

    SLIDE_MS 280, p = ease(t), ease(u) = u³(u(6u − 15) + 10), smootherstep.
    Going deeper, the screen on show leaves to the right, dx = +even(p·W), and the next comes in
      from the left, dx = −even((1 − p)·W); going back is the mirror. even() rounds to a whole
      art pixel, 2px. Settings stands beside the shelf rather than under it, and slides the
      mirror way: START sends the shelf out to the left and brings Settings in from the right,
      and back sends Settings out to the right and brings the shelf in from the left. Only that
      pair goes the mirror way: the screens under Settings, and Controls, go deeper and back.
    Leaving the shelf, its top bar goes up (dy = −p·36) and its carts across the way the slide
      goes (dx = ±p·(W + the widest cart): right for Controls, left for Settings), while the menu
      comes in from the other side. Coming back, the same with q = 1 − p.
    The bottom bar never leaves. When its actions change (shelf and menus), the old ones sink out
      of the screen over p 0 to 0.5 and the new ones rise into place over p 0.5 to 1; between two
      menus they stay put. Its hint changes with the screen, and its font the moment Theme's FONT
      does. Only on the screen after the power menu's choice, where nothing can be pressed, does
      the whole bar sink away, dy = even(p·38), as it does on the device when a cart goes in.
    The shelf's pill, its system between the shelf's actions, is there whenever they are and
      sinks with them. A new word in it, the system L1 and R1 go to or a letter Up and Down jump
      to, rises into place over PILL_IN_MS 220, sunk 2·round((1 − ease(t))·19) pixels, and a
      letter gives way to the system again, rising in the same way, LETTER_MS 2200 after it came.
    Holding POWER (P) opens the power menu deeper from whatever screen is shown, and its B goes
      back to that screen. The power menu and the screens after its choice slide down rather than
      across: going deeper, the screen on show drops out of the bottom, dy = +even(p·H), as the
      next comes down from the top, dy = −even((1 − p)·H), the shelf as one, its top bar and
      carts together; going back is the mirror, up out of the top and up from the bottom.

  Everything is placed with left and top in whole pixels, a frame at a time, from
  requestAnimationFrame: no transforms and no CSS transitions, so nothing ever lands between
  pixels. A key pressed during a slide finishes it at once, then does what it does.
*/
(() => {
  const SLIDE_MS = 280;
  const ease = u => u * u * u * (u * (6 * u - 15) + 10);
  const even = x => 2 * Math.round(x / 2) + 0;   // + 0: never -0px
  const FRAME_MS = 1000 / 60;

  // The shelf's parts' sizes in pixels (flow.css): its top bar; the widest cart.
  const TOP_H = 36;
  const WIDEST = 360;
  // An action's 15 art pixels and the 4 its notch rises above them: how far it sinks to be gone.
  const SINK = 19 * 2;
  // How long the pill takes to rise into place with a new word, and how long after it came a
  // letter gives way to the system, as app.rs has them (PILL_IN_MS, and PLATFORM_NAME_IN_MS +
  // _HOLD_MS + _OUT_MS).
  const PILL_IN_MS = 220;
  const LETTER_MS = 2200;

  const screen = document.getElementById('screen');
  const backdrop = screen.querySelector('.backdrop');
  const bar = screen.querySelector('cartridge-bar');
  const actions = [bar.querySelector('action-left'), bar.querySelector('action-right')];
  const pill = bar.querySelector('cartridge-pill');
  const shelf = {
    top: screen.querySelector('.shelf-top'),
    carts: screen.querySelector('.shelf-carts'),
  };
  const page = name => screen.querySelector(`.page[data-screen="${name}"]`);
  const { model, PARENT, GOING } = FlowScreens;
  const screens = FlowScreens.build(screen);

  const NAMES = {
    shelf: 'Shelf', settings: 'Settings', network: 'Network', wifi: 'Wi-Fi', time: 'Network Time',
    ssh: 'SSH', keyboard: 'Keyboard',
    bluetooth: 'Bluetooth', labels: 'Labels', datetime: 'Date & Time', background: 'Theme', about: 'About',
    controls: 'Controls', power: 'Power', restarting: 'Restarting', poweroff: 'Power Off',
  };
  // What the bottom bar shows: the shelf's actions, every menu's, and the key hints some menus add;
  // null on the screens after the power menu's choice, which have none.
  const SHELF_BAR = { left: ['CONTROLS', 'SELECT'], right: ['SETTINGS', 'START'] };
  const MENU_BAR = { left: ['RETURN', 'B'], right: ['CONFIRM', 'A'] };
  const barOf = name => (name === 'shelf' ? SHELF_BAR : GOING.includes(name) ? null : MENU_BAR);
  const HINTS = { labels: 'START FETCH' };
  // The two screens that stand beside each other rather than one under the other: a slide
  // between them goes the mirror way. A slide from Settings to a screen under it, or from the
  // shelf to Controls, goes the usual way.
  const ASIDE = ['shelf', 'settings'];
  const aside = ({ from, to }) => from !== to && ASIDE.includes(from) && ASIDE.includes(to);
  // Which way the screen a slide leaves goes, 1 right or −1 left: right going deeper, left going
  // back, and the mirror of each between the shelf and Settings.
  const leavesTo = slide => (slide.dir === 'in' ? 1 : -1) * (aside(slide) ? -1 : 1);
  // The power menu and the screens after its choice: a slide to or from one goes down rather
  // than across, the screen it leaves going down where it would go right.
  const POWERS = ['power', ...GOING];
  const drops = ({ from, to }) => POWERS.includes(from) || POWERS.includes(to);

  // what the bar shows too: its actions (SHELF_BAR or MENU_BAR) and how far they are sunk, in
  // pixels
  const state = { current: 'shelf', slide: null, slow: false, armed: false, actions: null, sunk: 0 };
  const duration = () => SLIDE_MS * (state.slow ? 5 : 1);
  // the pill's word: when it started rising into place (none once it is there), whether a letter
  // in it stays there (the URL asks for a still), whether a frame is coming for it, and whether
  // the URL is what put it there
  const word = { since: null, held: false, running: false, reading: false };
  const size = () => screen.dataset.size.split('x').map(Number);

  // ---- placing ----

  // The shelf `k` of the way out, its carts off to the right (side 1) or the left (side −1): 0
  // at rest, 1 gone.
  function placeShelf(k, side) {
    const [W] = size();
    shelf.top.style.top = `${-even(k * TOP_H)}px`;
    shelf.carts.style.left = `${side * even(k * (W + WIDEST)) + 0}px`;
    shelf.carts.style.top = `${TOP_H}px`;
  }

  // A screen `dx` pixels across; the shelf instead `k` of the way out, its carts to `side`.
  function place(name, dx, k, side) {
    if (name === 'shelf') return placeShelf(k, side);
    page(name).style.left = `${dx}px`;
    page(name).style.top = '0px';
  }

  // A screen `dy` pixels down, the shelf as one: its top bar and carts together.
  function placeDown(name, dy) {
    if (name !== 'shelf') {
      page(name).style.left = '0px';
      page(name).style.top = `${dy}px`;
      return;
    }
    shelf.top.style.top = `${dy}px`;
    shelf.carts.style.left = '0px';
    shelf.carts.style.top = `${TOP_H + dy}px`;
  }

  // Only these screens' parts shown; the rest are hidden, keeping their layout.
  function showOnly(...names) {
    for (const part of screen.querySelectorAll('.part')) {
      part.classList.toggle('away', !names.includes(part.dataset.screen));
    }
  }

  // The bar's actions, touched only when they change: a new label is a new sprite.
  function setActions(which) {
    const { left, right } = which;
    state.actions = which;
    actions.forEach((action, i) => {
      const [label, hint] = i ? right : left;
      if (action.getAttribute('label') !== label) action.setAttribute('label', label);
      if (action.getAttribute('hint') !== hint) action.setAttribute('hint', hint);
    });
  }
  function sinkActions(dy) {
    for (const action of actions) action.style.top = `${dy}px`;
    state.sunk = dy;
    placePill();
  }

  // ---- the shelf's pill ----

  // How far below its place the pill's word still is as it rises in, in pixels.
  function rising(now) {
    if (word.since === null) return 0;
    const t = Math.min(1, (now - word.since) / (PILL_IN_MS * (state.slow ? 5 : 1)));
    return 2 * Math.round(((1 - ease(t)) * SINK) / 2);
  }

  // The pill with the shelf's actions, sunk as far as they are and as far as its word still has
  // to rise, never further than out of sight.
  function placePill(now = performance.now()) {
    pill.hidden = state.actions !== SHELF_BAR;
    pill.style.top = `${Math.min(SINK, state.sunk + rising(now))}px`;
  }

  // The pill's word, the letter jumped to or else the system; `rise` brings it up into place.
  function setWord(rise) {
    pill.setAttribute('text', model.letter ?? model.system);
    word.since = rise ? performance.now() : null;
    if (rise) word.held = false;
    placePill();
    if (rise) runWord();
  }

  // The pill's word rising, a frame at a time, and a letter giving way to the system when its
  // time is up.
  function runWord() {
    if (word.running) return;
    word.running = true;
    requestAnimationFrame(function frame(now) {
      const slow = state.slow ? 5 : 1;
      const passing = () => model.letter && !word.held;
      if (passing() && now - word.since >= LETTER_MS * slow) FlowScreens.set('letter', null);
      placePill(now);
      // done once the word is in place, unless it is a letter still to give way
      if (now - word.since >= PILL_IN_MS * slow && !passing()) word.since = null;
      if (word.since !== null) requestAnimationFrame(frame);
      else word.running = false;
    });
  }
  // The whole bar, hints and actions, `dy` pixels down out of the screen.
  function dropBar(dy) {
    bar.style.bottom = `${-dy}px`;
  }
  // The screen's hint for where its cursor is, if it has one (screens.js), else the screen's own.
  function setHint(name) {
    const hint = screens[name].hint?.() ?? HINTS[name];
    if (!hint) bar.removeAttribute('hint');
    else if (bar.getAttribute('hint') !== hint) bar.setAttribute('hint', hint);
  }

  // A screen at rest, alone; one with no bar has it sunk away.
  function rest(name) {
    showOnly(name);
    place(name, 0, 0, 1);
    setActions(barOf(name) ?? MENU_BAR);
    sinkActions(0);
    dropBar(barOf(name) ? 0 : SINK);
    setHint(name);
  }

  // One frame of a slide, at its time t.
  function render(slide) {
    const { from, to, t } = slide;
    const p = ease(t);
    const [W, H] = size();
    const sign = leavesTo(slide);
    const down = drops(slide);
    const leaving = sign * even(p * (down ? H : W));
    const arriving = -sign * even((1 - p) * (down ? H : W));
    if (down) {
      placeDown(from, leaving);
      placeDown(to, arriving);
    } else {
      place(from, leaving, p, sign);
      place(to, arriving, 1 - p, -sign);
    }
    const [before, after] = [barOf(from), barOf(to)];
    let sunk = 0;
    let drop = 0;
    // to a screen with no bar, the whole bar sinks away with the actions it had
    if (!after) { setActions(before ?? MENU_BAR); drop = even(p * SINK); }
    else if (!before) { setActions(after); drop = even((1 - p) * SINK); }
    else if (before === after) setActions(after);
    else if (p < 0.5) { setActions(before); sunk = even((p / 0.5) * SINK); }
    else { setActions(after); sunk = even(((1 - p) / 0.5) * SINK); }
    sinkActions(sunk);
    dropBar(drop);
    setHint(p < 0.5 ? from : to);
    readout(slide, p, leaving, arriving, sunk);
  }

  // ---- sliding ----

  function go(to, dir, { t = 0, hold = false } = {}) {
    finish();
    const slide = { from: state.current, to, dir, t, hold: hold || state.armed, start: 0 };
    state.armed = false;
    state.current = to;
    state.slide = slide;
    showOnly(slide.from, to);
    centre();
    render(slide);
    if (!slide.hold) run(slide);
    writeHash();
  }

  function run(slide) {
    requestAnimationFrame(function frame(now) {
      if (state.slide !== slide || slide.hold) return;
      if (!slide.start) slide.start = now - slide.t * duration();
      slide.t = Math.min(1, (now - slide.start) / duration());
      render(slide);
      if (slide.t < 1) requestAnimationFrame(frame);
      else finish();
    });
  }

  // The slide under way, done at once.
  function finish() {
    const slide = state.slide;
    if (!slide) return;
    state.slide = null;
    rest(slide.to);
    readout();
    writeHash();
  }

  // A screen at once, with the cursors up the tree on the rows that lead to it, as if walked: the
  // power menu and the screens after it as if POWER were held on the shelf.
  function jump(name) {
    finish();
    if (name === 'power' || GOING.includes(name)) PARENT.power = 'shelf';
    for (let child = name; PARENT[child]; child = PARENT[child]) screens[PARENT[child]].focus?.(child);
    state.current = name;
    rest(name);
    centre();
    readout();
    writeHash();
  }

  function press(key) {
    finish();
    if (key === 'power') return holdPower();
    const answer = screens[state.current].press(key);
    if (answer?.go) go(answer.go, 'in');
    else if (answer?.back && PARENT[state.current]) go(PARENT[state.current], 'out');
    else {
      setHint(state.current);
      writeHash();
    }
  }

  // Holding POWER: the power menu, its cursor on RESTART, deeper from the screen shown, which its
  // B goes back to. Nothing happens on the menu itself or after its choice.
  function holdPower() {
    if (state.current === 'power' || GOING.includes(state.current)) return;
    PARENT.power = state.current;
    screens.power.cursor = 0;
    screens.power.light();
    go('power', 'in');
  }

  // ---- whole pixels for what is centred ----

  // A lone panel or plate in the middle of the screen above the bar, and the keyboard down its
  // well, each offset rounded down to a whole art pixel.
  const down = px => Math.floor(px / 4) * 2;
  function centre() {
    for (const lone of screen.querySelectorAll('.lone')) {
      const area = lone.parentElement;
      lone.style.left = `${down(area.clientWidth - lone.offsetWidth)}px`;
      lone.style.top = `${down(area.clientHeight - lone.offsetHeight)}px`;
    }
    const panel = page('keyboard').querySelector('cartridge-panel');
    const well = panel.shadowRoot.querySelector('[part="body"]');
    const typing = panel.querySelector('.typing');
    const style = getComputedStyle(well);
    const room = well.clientHeight - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom);
    typing.style.marginTop = `${Math.max(0, down(room - typing.offsetHeight))}px`;
  }
  const watch = new ResizeObserver(centre);
  for (const el of [screen, ...screen.querySelectorAll('.lone, .page cartridge-panel')]) watch.observe(el);
  document.fonts.ready.then(centre);

  // ---- what the page shows of it ----

  const box = document.getElementById('readout');
  function readout(slide, p, leaving, arriving, sunk) {
    if (!slide) {
      box.textContent = `${NAMES[state.current]}${state.armed ? '\nthe next slide waits for Step' : ''}`;
      return;
    }
    const frames = Math.round(duration() / FRAME_MS);
    const way = `${slide.dir === 'in' ? 'deeper' : 'back'}${aside(slide) ? ', mirrored' : ''}`;
    const shelfK = slide.from === 'shelf' ? p : slide.to === 'shelf' ? 1 - p : null;
    // the side the shelf's carts are off to, and how far across the menu is
    const [side, menuDx] = slide.from === 'shelf'
      ? [leavesTo(slide), arriving]
      : [-leavesTo(slide), leaving];
    const carts = side * even(shelfK * (size()[0] + WIDEST)) + 0;
    box.textContent = [
      `${NAMES[slide.from]} → ${NAMES[slide.to]}, ${way}${slide.hold ? ', held' : ''}`,
      `t ${slide.t.toFixed(3)}  p ${p.toFixed(3)}  frame ${Math.round(slide.t * frames)}/${frames}`,
      drops(slide)
        ? `leaving dy ${leaving}px  arriving dy ${arriving}px  actions ${sunk}px down`
        : shelfK === null
        ? `leaving dx ${leaving}px  arriving dx ${arriving}px  actions ${sunk}px down`
        : `shelf: top ${-even(shelfK * TOP_H)}px  carts ${carts}px  ` +
          `menu dx ${menuDx}px  actions ${sunk}px down`,
    ].join('\n');
  }

  // ---- the URL ----

  const SIZES = ['640x480', '720x480', '720x720'];
  const FONTS = ['bold', 'thin'];
  const clamp01 = v => Math.max(0, Math.min(1, v));

  function writeHash() {
    const q = new URLSearchParams();
    const slide = state.slide;
    q.set('screen', state.current);
    if (slide?.hold) {
      q.set('from', slide.from);
      q.set('t', String(Math.round(slide.t * 1000) / 1000));
    }
    if (screen.dataset.size !== '640x480') q.set('size', screen.dataset.size);
    if (model.colour !== 'night') q.set('colour', model.colour);
    if (model.style !== 'grid') q.set('style', model.style);
    if (!model.moving) q.set('moving', '0');
    if (model.font !== 'bold') q.set('font', model.font);
    if (!model.wifi) q.set('wifi', 'off');
    if (!model.ntp) q.set('ntp', 'off');
    if (FlowScreens.scanState() !== 'none') q.set('scan', FlowScreens.scanState());
    if (FlowScreens.bluetoothState() !== 'on') q.set('bt', FlowScreens.bluetoothState());
    if (model.system !== 'GBA') q.set('system', model.system.toLowerCase());
    if (model.letter && word.held) q.set('letter', model.letter);
    if (state.slow) q.set('slow', '1');
    if (document.body.classList.contains('shot')) q.set('shot', '1');
    history.replaceState(null, '', `#${q}`.replaceAll('%2C', ','));
  }

  // The cursor where `row` says, on a screen that has one.
  function setRow(name, row) {
    if (row === null) return;
    const target = screens[name];
    const [a, b] = row.split(',').map(Number);
    if (target.rows && a >= 0 && a < target.rows.length) { target.cursor = a; target.light(); }
    else if (target.line && a >= 0 && a < 6) { target.cursor = a; target.draw(); }
    else if (target.grid && a >= 0 && a < 5) { target.at = { row: a, col: b || 0 }; target.draw(); }
  }

  // Is `name` under `above` in the tree?
  const under = (name, above) => {
    for (let at = PARENT[name]; at; at = PARENT[at]) if (at === above) return true;
    return false;
  };

  function readHash() {
    const q = new URLSearchParams(location.hash.slice(1));
    // what the hash leaves out is the default, so it always says the whole of what is shown
    const pick = (key, values, fallback) => (values.includes(q.get(key)) ? q.get(key) : fallback);
    document.body.classList.toggle('shot', q.get('shot') === '1');
    screen.dataset.size = pick('size', SIZES, '640x480');
    FlowScreens.set('colour', pick('colour', CartridgePalette.COLOURS, 'night'));
    FlowScreens.set('style', pick('style', CartridgeBackdrop.STYLES, 'grid'));
    FlowScreens.set('moving', q.get('moving') !== '0');
    FlowScreens.set('font', pick('font', FONTS, 'bold'));
    FlowScreens.set('wifi', q.get('wifi') !== 'off');
    FlowScreens.set('ntp', q.get('ntp') !== 'off');
    FlowScreens.scanAs(pick('scan', FlowScreens.SCAN, 'none'));
    FlowScreens.bluetoothAs(pick('bt', FlowScreens.BT, 'on'));
    // the pill at rest with what the URL says, a letter in it held there
    word.reading = true;
    const systems = FlowScreens.SYSTEMS.map(name => name.toLowerCase());
    FlowScreens.shelfAs(pick('system', systems, 'gba').toUpperCase(), pick('letter', FlowScreens.INITIALS, null));
    word.reading = false;
    setWord(false);
    word.held = Boolean(model.letter);
    state.slow = q.get('slow') === '1';
    syncControls();

    const to = q.get('screen') in NAMES ? q.get('screen') : 'shelf';
    const from = q.get('from') in NAMES ? q.get('from') : null;
    if (!from || from === to) {
      jump(to);
      setRow(to, q.get('row'));
      // a hint that follows the cursor follows it to its row
      setHint(to);
      return;
    }
    jump(from);
    // the power menu goes back to the screen it slid in from
    if (to === 'power') PARENT.power = from;
    const dir = ['in', 'out'].includes(q.get('dir')) ? q.get('dir') :
      under(from, to) ? 'out' : 'in';
    // going deeper, the cursor on the row that opens the next screen
    if (dir === 'in') screens[from].focus?.(to);
    setRow(to, q.get('row'));
    if (q.has('t')) go(to, dir, { t: clamp01(Number(q.get('t')) || 0), hold: true });
    else go(to, dir);
  }

  // ---- the page's controls ----

  const radios = name => [...document.querySelectorAll(`input[name="${name}"]`)];

  const list = document.getElementById('screens');
  for (const [name, label] of Object.entries(NAMES)) {
    const button = document.createElement('button');
    button.textContent = label;
    button.addEventListener('click', () => jump(name));
    list.append(button);
  }

  const STYLE_NAMES = { grid: 'Grid', plates: 'Plates', dotted: 'Dotted', dither: 'Dither', rivets: 'Rivets', bits: 'Bits', solid: 'Solid' };
  document.getElementById('styles').append(...CartridgeBackdrop.STYLES.map(style => {
    const label = document.createElement('label');
    label.innerHTML = `<input type="radio" name="style" value="${style}"><span>${STYLE_NAMES[style]}</span>`;
    label.querySelector('input').addEventListener('change', () => FlowScreens.set('style', style));
    return label;
  }));

  const colours = document.getElementById('colours');
  for (const name of CartridgePalette.COLOURS) {
    const tones = CartridgePalette.palette(name);
    const label = document.createElement('label');
    label.innerHTML = `<input type="radio" name="colour" value="${name}">` +
      `<span><i class="chip" style="background: ${tones['--oz-bg']}; --chip-frame: ${tones['--oz-frame']}"></i>` +
      `${name[0].toUpperCase()}${name.slice(1)}</span>`;
    label.querySelector('input').addEventListener('change', () => FlowScreens.set('colour', name));
    colours.append(label);
  }

  document.getElementById('moving').addEventListener('change', e => FlowScreens.set('moving', e.target.checked));
  for (const input of radios('font')) {
    input.addEventListener('change', () => FlowScreens.set('font', input.value));
  }
  document.getElementById('wifi').addEventListener('change', e => FlowScreens.set('wifi', e.target.checked));
  document.getElementById('bluetooth').addEventListener('change', e => FlowScreens.set('bluetooth', e.target.checked));
  for (const input of radios('size')) {
    input.addEventListener('change', () => {
      screen.dataset.size = input.value;
      if (state.slide) render(state.slide); else rest(state.current);
      centre();
      writeHash();
    });
  }
  for (const input of radios('zoom')) {
    input.addEventListener('change', () => { screen.style.zoom = input.value; });
  }
  document.getElementById('slow').addEventListener('change', e => {
    state.slow = e.target.checked;
    // keep a running slide where it is, at its new speed
    const slide = state.slide;
    if (slide && !slide.hold) slide.start = performance.now() - slide.t * duration();
    writeHash();
  });
  document.getElementById('step').addEventListener('click', () => {
    const slide = state.slide;
    if (!slide) {
      state.armed = true;
      readout();
      return;
    }
    slide.hold = true;
    slide.t = Math.min(1, slide.t + FRAME_MS / duration());
    render(slide);
    writeHash();
    if (slide.t >= 1) finish();
  });

  // the page's controls showing what the model and the screen hold
  function syncControls() {
    for (const input of radios('style')) input.checked = input.value === model.style;
    for (const input of radios('colour')) input.checked = input.value === model.colour;
    for (const input of radios('font')) input.checked = input.value === model.font;
    for (const input of radios('size')) input.checked = input.value === screen.dataset.size;
    document.getElementById('moving').checked = model.moving;
    document.getElementById('wifi').checked = model.wifi;
    document.getElementById('bluetooth').checked = model.bluetooth;
    document.getElementById('slow').checked = state.slow;
  }

  // Settings › Theme works the real backdrop, palette and bar, and so does the page.
  FlowScreens.onChange(key => {
    if (key === 'style' && backdrop.dataset.style !== model.style) backdrop.dataset.style = model.style;
    if (key === 'moving' && backdrop.hasAttribute('data-moving') !== model.moving) backdrop.toggleAttribute('data-moving', model.moving);
    if (key === 'colour' && document.documentElement.dataset.colour !== model.colour) CartridgePalette.apply(model.colour);
    if (key === 'font') bar.setAttribute('font', model.font);
    // a new system or letter rises into the pill, but not one the URL put there
    if ((key === 'system' || key === 'letter') && !word.reading) setWord(true);
    // a device paired or forgotten under the cursor, or Network's scan, changes the hint
    if (!state.slide) setHint(state.current);
    syncControls();
    writeHash();
  });

  // the placeholders' sprites, in the palette's colours, again whenever it changes
  FlowArt.paint();
  document.addEventListener('cartridge-palette', () => FlowArt.paint());

  // ---- the buttons ----

  const KEYS = {
    ArrowUp: 'up', ArrowDown: 'down', ArrowLeft: 'left', ArrowRight: 'right',
    a: 'a', z: 'b', b: 'b', Escape: 'b', x: 'x', s: 'x', y: 'y', Enter: 'start', ' ': 'select',
    l: 'l1', r: 'r1', p: 'power',
  };
  document.addEventListener('keydown', e => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const key = e.code === 'ShiftRight' ? 'select' : KEYS[e.key] ?? KEYS[e.key.toLowerCase()];
    if (!key) return;
    e.preventDefault();
    // a held arrow walks on; a held button presses once
    if (e.repeat && !['up', 'down', 'left', 'right'].includes(key)) return;
    press(key);
  });
  for (const button of document.querySelectorAll('.pad [data-key]')) {
    button.addEventListener('click', () => press(button.dataset.key));
  }

  // the actions slide by their own top, inside the bar
  for (const action of actions) action.style.position = 'relative';

  window.addEventListener('hashchange', readHash);
  readHash();
})();
