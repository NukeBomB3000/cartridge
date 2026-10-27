/*
  What moves on cartridge's site: the screens' picker and how it advances by itself, the loops,
  the Theme panel, and the bottom bar's two actions. The tabs are <cartridge-tabs>'s own. The page
  reads and works without scripts: every part shown one under the other, every screen too, and
  each loop as its poster with the browser's own controls.
*/
(() => {
  // What the Theme panel was last left at, kept in this browser only; nothing breaks without it.
  const kept = {
    get(key) {
      try { return localStorage.getItem(`cartridge:${key}`); } catch { return null; }
    },
    set(key, value) {
      try { localStorage.setItem(`cartridge:${key}`, value); } catch { /* not kept */ }
    },
  };
  // 'IN GAME' as 'In game', for a name read aloud
  const spoken = label => label[0] + label.slice(1).toLowerCase();
  const still = matchMedia('(prefers-reduced-motion: reduce)');
  const shown = () => document.visibilityState === 'visible';

  // ---- the screens: a row for each, lit for the one the handheld shows ----
  // A tab list down the SCREENS panel; Up and Down (or Left and Right) move through it, as the
  // cursor does on the handheld, and the screen follows.
  const viewer = document.querySelector('.viewer');
  const picker = document.querySelector('.picker');
  const shots = [...document.querySelectorAll('.shot')];
  let current = 0;
  const tabs = shots.map((shot, i) => {
    const tab = document.createElement('button');
    tab.type = 'button';
    tab.id = `tab-${shot.id}`;
    tab.setAttribute('role', 'tab');
    tab.setAttribute('aria-controls', shot.id);
    tab.setAttribute('aria-label', spoken(shot.dataset.label));
    const row = document.createElement('cartridge-row');
    row.setAttribute('label', shot.dataset.label);
    row.setAttribute('opens', '');
    tab.append(row);
    tab.addEventListener('click', () => show(i));
    shot.setAttribute('role', 'tabpanel');
    shot.setAttribute('aria-labelledby', tab.id);
    picker.append(tab);
    return tab;
  });

  function show(i, focus = false) {
    current = i;
    shots.forEach((shot, j) => shot.classList.toggle('on', i === j));
    tabs.forEach((tab, j) => {
      tab.setAttribute('aria-selected', String(i === j));
      tab.tabIndex = i === j ? 0 : -1;
      tab.firstElementChild.toggleAttribute('lit', i === j);
    });
    if (focus) tabs[i].focus();
  }

  picker.addEventListener('keydown', event => {
    const at = tabs.indexOf(document.activeElement);
    const to = {
      ArrowDown: at + 1, ArrowRight: at + 1, ArrowUp: at - 1, ArrowLeft: at - 1,
      Home: 0, End: tabs.length - 1,
    }[event.key];
    if (at < 0 || to === undefined) return;
    event.preventDefault();
    show((to + tabs.length) % tabs.length, true);
  });
  show(0);
  document.querySelector('.shots').classList.add('ready');

  // ---- the screens advance by themselves ----
  // The next one every few seconds while SCREENS is the tab shown and on screen. The pointer over
  // it or the focus in it holds it; any press holds it a while longer, and PAUSE stops it until
  // PLAY. Nothing advances while the system asks for less motion.
  const ADVANCE = 3500;
  const HOLD = 6000;
  const toggle = document.querySelector('.auto-toggle');
  const toggleBox = toggle.closest('.auto-buttons');
  let stopped = false;
  let hovered = false;
  let inView = false;
  let heldUntil = 0;
  let next = performance.now() + ADVANCE;
  const hold = () => { heldUntil = performance.now() + HOLD; };
  const running = () => !still.matches && !stopped && shown() && inView && !viewer.closest('[hidden]')
    && !hovered && !viewer.contains(document.activeElement) && performance.now() >= heldUntil;
  setInterval(() => {
    const now = performance.now();
    if (!running()) {
      next = Math.max(next, now + ADVANCE);
      return;
    }
    if (now >= next) {
      show((current + 1) % shots.length);
      next = now + ADVANCE;
    }
  }, 250);
  viewer.addEventListener('pointerenter', () => { hovered = true; });
  viewer.addEventListener('pointerleave', () => { hovered = false; hold(); });
  viewer.addEventListener('focusout', hold);
  for (const type of ['pointerdown', 'keydown', 'wheel']) viewer.addEventListener(type, hold, { passive: true });
  new IntersectionObserver(([entry]) => { inView = entry.isIntersecting; }).observe(viewer);
  toggle.addEventListener('click', () => {
    stopped = !stopped;
    toggle.textContent = stopped ? 'PLAY' : 'PAUSE';
    toggle.setAttribute('aria-pressed', String(stopped));
  });
  // the button only where the screens can move at all
  const offerToggle = () => { toggleBox.hidden = still.matches; };
  still.addEventListener('change', offerToggle);
  offerToggle();

  // ---- the loops ----
  // Muted, looping and inline; each plays while it is on screen and stops when it is not (a tab
  // not shown is not on screen), and loads only once it is first seen. A poster given as
  // data-poster is fetched only once its loop comes near the screen. Under reduced motion each
  // stays its poster unless pressed. A click, Enter or Space pauses one or plays it again.
  const loops = [...document.querySelectorAll('video.clip')];
  const near = new IntersectionObserver(entries => {
    for (const { target, isIntersecting } of entries) {
      if (!isIntersecting) continue;
      target.poster = target.dataset.poster;
      near.unobserve(target);
    }
  }, { rootMargin: '600px 0px' });
  for (const video of loops) if (video.dataset.poster && !video.poster) near.observe(video);
  const visible = new Set();
  const chosen = new Map(); // a loop pressed: true to play, false to stay paused
  function update(video) {
    const wanted = chosen.get(video) ?? !still.matches;
    if (visible.has(video) && wanted && shown()) {
      video.preload = 'auto';
      video.autoplay = true;
      video.play().catch(() => { /* not allowed or not loaded yet; the poster stays */ });
    } else if (!video.paused) {
      video.pause();
    }
  }
  const watch = new IntersectionObserver(entries => {
    for (const { target, isIntersecting } of entries) {
      if (isIntersecting) visible.add(target);
      else visible.delete(target);
      update(target);
    }
  });
  for (const video of loops) {
    video.muted = true;
    video.removeAttribute('controls');
    video.tabIndex = 0;
    const press = () => {
      chosen.set(video, video.paused);
      update(video);
    };
    video.addEventListener('click', press);
    video.addEventListener('keydown', event => {
      if (event.key !== 'Enter' && event.key !== ' ') return;
      event.preventDefault();
      press();
    });
    watch.observe(video);
  }
  const updateAll = () => loops.forEach(update);
  still.addEventListener('change', updateAll);
  document.addEventListener('visibilitychange', updateAll);

  // ---- the bottom bar ----
  // Its actions are links here, so the bar does not pair them itself: they are given the room
  // of the wider label and hint of the two, as the bar would, and the Theme's font. On a phone
  // DOWNLOAD becomes GET, so both fit across.
  const actions = [...document.querySelectorAll('.site-bar action-left, .site-bar action-right')];
  let font = 'bold';
  const narrow = matchMedia('(max-width: 479px)');
  function pair() {
    const right = actions.find(action => action.localName === 'action-right');
    right?.setAttribute('label', narrow.matches ? 'GET' : 'DOWNLOAD');
    for (const action of actions) action.setAttribute('font', font);
    const boxes = actions.map(action => action.boxes());
    const room = [0, 1].map(i => Math.max(...boxes.map(box => box[i]))).join(' ');
    for (const action of actions) action.setAttribute('room', room);
  }
  narrow.addEventListener('change', pair);

  // ---- the Theme panel: Settings > Theme, for the page ----
  // Each row a slider: Left and Right step through its values (a click on its arrows does too,
  // and a click anywhere else on it steps forward), Up and Down move to the row above or below.
  // The row under the cursor, the focused or pointed-at one, is lit.
  const backdrops = [...document.querySelectorAll('.cartridge-backdrop')];
  const SETTINGS = {
    style: {
      values: CartridgeBackdrop.STYLES,
      apply: value => backdrops.forEach(el => {
        if (value === 'grid') delete el.dataset.style;
        else el.dataset.style = value;
      }),
    },
    motion: {
      values: ['moving', 'still'],
      apply: value => backdrops.forEach(el => {
        el.toggleAttribute('data-moving', value === 'moving');
      }),
    },
    colour: { values: CartridgePalette.COLOURS, apply: value => CartridgePalette.apply(value) },
    font: { values: ['bold', 'thin'], apply: value => { font = value; pair(); } },
  };

  const rows = [...document.querySelectorAll('.setting')];
  for (const row of rows) {
    const name = row.dataset.setting;
    const setting = SETTINGS[name];
    const omega = row.querySelector('cartridge-row');
    // Motion stays still, and its row dim, while the system asks for less motion.
    const locked = () => name === 'motion' && still.matches;
    const set = (index, keep = true) => {
      const count = setting.values.length;
      const value = locked() ? 'still' : setting.values[(index + count) % count];
      setting.apply(value);
      omega.setAttribute('value', value.toUpperCase());
      omega.toggleAttribute('dim', locked());
      row.setAttribute('aria-valuemin', '0');
      row.setAttribute('aria-valuemax', String(count - 1));
      row.setAttribute('aria-valuenow', String(setting.values.indexOf(value)));
      row.setAttribute('aria-valuetext', spoken(value.toUpperCase()));
      if (locked()) row.setAttribute('aria-disabled', 'true');
      else row.removeAttribute('aria-disabled');
      if (keep && !locked()) kept.set(name, value);
    };
    const now = () => setting.values.indexOf(omega.getAttribute('value')?.toLowerCase());
    const step = by => { if (!locked()) set(now() + by); };

    row.addEventListener('keydown', event => {
      const move = { ArrowUp: -1, ArrowDown: 1 }[event.key];
      if (move) {
        event.preventDefault();
        rows[(rows.indexOf(row) + move + rows.length) % rows.length].focus();
        return;
      }
      const by = { ArrowLeft: -1, ArrowRight: 1 }[event.key];
      if (by) step(by);
      else if (event.key === 'Home' && !locked()) set(0);
      else if (event.key === 'End' && !locked()) set(setting.values.length - 1);
      else return;
      event.preventDefault();
    });
    row.addEventListener('click', event => {
      const part = event.composedPath()[0]?.getAttribute?.('part');
      step(part === 'caret-left' ? -1 : 1);
    });
    const light = () => omega.toggleAttribute('lit', row.matches(':focus, :hover'));
    for (const type of ['focus', 'blur', 'mouseenter', 'mouseleave']) {
      row.addEventListener(type, () => requestAnimationFrame(light));
    }
    // what was kept, else the first value: at load, and again when the system's motion changes
    const start = () => set(Math.max(0, setting.values.indexOf(kept.get(name))), false);
    still.addEventListener('change', start);
    start();
  }

  pair();
})();
