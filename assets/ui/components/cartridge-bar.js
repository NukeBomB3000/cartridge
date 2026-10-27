/*
  <cartridge-bar>: the bar along the bottom of a screen. An <action-left> and an <action-right> at its
  ends and, between them, nothing but the keys a screen has beyond the two actions, shown as
  hints: whatever is behind the bar shows through the rest.

    <cartridge-bar hint="Y FORGET">
      <action-left slot="left" label="BACK" hint="B"></action-left>
      <action-right slot="right" label="JOIN" hint="A"></action-right>
    </cartridge-bar>
    <cartridge-bar>
      <action-left slot="left" label="CONTROLS" hint="SELECT"></action-left>
      <cartridge-pill slot="middle" text="GBA" centred></cartridge-pill>
      <action-right slot="right" label="SETTINGS" hint="START"></action-right>
    </cartridge-bar>

  hint      a key and what it does, the first word the key: "Y FORGET", "START FETCH". Two are
            given with a | between them: "Y FORGET | X RENAME". Each is drawn as its key in a
            small dark cap, then the word, in Pixel Code one face pixel to an art pixel, and the
            hints together are centred between the two actions, 8 pixels apart.
  font      the actions' labels, "bold" (or none) or "thin", as Settings > Theme > Font picks:
            the bar gives it to both its actions.

  Slots: left and right, for the two actions. The bar makes them one width: it gives both the
  room (see action-left.js) of the wider label and the wider hint of the two, again whenever
  either's text or the font changes, so each label and each hint is centred in a box as wide as
  the other's. An empty action takes the room of the one beside it. middle, for what stands
  between them in place of hints: the shelf's system, an <cartridge-pill centred> (cartridge-pill.js),
  at the bar's top and across the whole room between the actions, so a centred pill is in the
  middle of the screen. A bar with one has no hints.

  It fills its parent's width and is 15 art pixels tall, the actions' height; their notches rise 4
  above it, over whatever is there. The hints' capitals are on rows 5 to 11, the same rows as the
  words in the actions' buttons, and each key's cap is 9 rows tall around them: a dark row above
  and below its letters' capitals, two dark pixels either side. Areas, each a div that ::part()
  reaches: bar, middle (the room between the actions), hints, hint, key, word.

  How it draws its hints is CartridgeBar, which <cartridge-hints> (cartridge-hints.js) draws its own with:
  CartridgeBar.HINTS, their style, for a shadow root; CartridgeBar.KEY, the cap's sprite, which the style
  takes as --cap; and CartridgeBar.hints(box, value), which fills a .hints box with value's hints and
  sets its --ink.

  Needs everything in foundation/ on the page (see README.md) and the two actions' components; its
  colours follow palette.js.
*/
(() => {
  // Sizes in art pixels: the bar's height and how tall a key's cap is.
  const BAR = 15;
  const KEY_H = 9;
  // How far down the bar the hints' line starts: row 3, as the pills' words' does, so their
  // capitals, two rows down it, are on rows 5 to 11 with the words in the actions' buttons.
  const HINTS_TOP = 3;
  // How far down the hint's line its key's cap starts: a row above the capitals.
  const KEY_TOP = CartridgePixel.ACCENT - (KEY_H - CartridgePixel.CAP) / 2;
  // The gap between a key's cap and its word, and between two hints, in pixels of ink to ink.
  const GAP = 3;
  const BETWEEN = 8;

  // A key's cap: its left end, the column that stretches, its right end; dark, with its four
  // corner pixels left out.
  const KEY = ['.k.', ...Array(KEY_H - 2).fill('kkk'), '.k.'];

  // Pixel Code letters are 6 pixels apart, the last of the 6 blank.
  const ADVANCE = 6;

  // How the hints are drawn in their box, .hints, wherever it is: on a line of 10, each hint its
  // key in a small dark cap (the sprite in --cap), then the word.
  const HINTS = `
      .hints {
        display: flex;
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        white-space: nowrap;
        color: var(--oz-cream);
      }
      .hint { display: flex; }
      .hint + .hint { margin-left: calc(${BETWEEN - 1} * var(--u)); }
      /* The key's cap starts a row above the capitals and ends a row below them. Its own line is
         that row shorter at either end, so its letters stay on the hint's capitals' rows. They
         start two dark pixels in, its end column and one more; on the right the letters' own blank
         pixel and the end column make the same two. */
      .key {
        box-sizing: border-box;
        height: calc(${KEY_H} * var(--u));
        margin-top: calc(${KEY_TOP} * var(--u));
        line-height: calc(${10 - 2 * KEY_TOP} * var(--u));
        padding-left: var(--u);
        border-style: solid;
        border-color: transparent;
        border-width: 0 var(--u);
        border-image-source: var(--cap);
        border-image-slice: 0 1 fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      .word { margin-left: calc(${GAP} * var(--u)); }
      .word:empty { display: none; }`;

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: block; flex: none; height: calc(${BAR} * var(--u)); text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .bar { display: flex; height: 100%; }
      ::slotted(*) { position: relative; z-index: 1; flex: none; }

      /* the room between the two actions, from one's end to the other's: nothing is drawn in it but
         the hints, or the shelf's pill, so whatever is behind the bar shows through */
      .middle { position: relative; flex: 1; min-width: 0; }
      /* what stands in it in place of hints, over the whole room from the bar's top: a page moves
         it by its top, as it does an action */
      ::slotted([slot="middle"]) { position: absolute; left: 0; right: 0; top: 0; z-index: 0; }
${HINTS}
      /* Centred between the actions, rounded down to a whole art pixel: --ink is the hints' width in
         art pixels from their first ink to their last, which hints() works out from the text. */
      .hints {
        width: max-content;
        margin-top: calc(${HINTS_TOP} * var(--u));
        margin-left: max(0px, round(down, (100% - var(--ink, 0) * var(--u)) / 2, var(--u)));
      }
    </style>
    <div class="bar" part="bar">
      <slot name="left"></slot>
      <div class="middle" part="middle"><div class="hints" part="hints"></div><slot name="middle"></slot></div>
      <slot name="right"></slot>
    </div>`;

  // The hints in `value`, each { key, word }.
  function parse(value) {
    return (value ?? '').split('|').map(hint => hint.trim()).filter(Boolean).map(hint => {
      const [key, ...word] = hint.split(/\s+/);
      return { key, word: word.join(' ') };
    });
  }

  // `box` filled with one key and word per hint in `value`, and its --ink set to the width they
  // take from their first ink to their last, in art pixels.
  function hints(box, value) {
    const all = parse(value);
    const length = text => [...text].length;
    box.replaceChildren(...all.map(({ key, word }) => {
      const hint = document.createElement('div');
      hint.className = 'hint';
      hint.setAttribute('part', 'hint');
      for (const [name, text] of [['key', key], ['word', word]]) {
        const area = document.createElement('div');
        area.className = name;
        area.setAttribute('part', name);
        area.textContent = text;
        hint.append(area);
      }
      return hint;
    }));
    // a cap is its letters and three pixels more, a word its letters; the last word's blank
    // pixel is not ink
    const ink = all.reduce((sum, { key, word }, i) =>
      sum + (i ? BETWEEN - 1 : 0) + length(key) * ADVANCE + 3 + (word ? GAP + length(word) * ADVANCE : 0), 0);
    const last = all.at(-1);
    box.style.setProperty('--ink', all.length ? ink - (last.word ? 1 : 0) : 0);
  }

  window.CartridgeBar = { HINTS, KEY, hints };

  customElements.define('cartridge-bar', class extends CartridgePixel.Element {
    static observedAttributes = ['hint', 'font'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
      // the actions paired again whenever one comes or goes, or changes its text
      this.shadowRoot.addEventListener('slotchange', () => this.pair());
      new MutationObserver(() => this.pair()).observe(this, { subtree: true, attributeFilter: ['label', 'hint'] });
    }

    paint() {
      this.shadowRoot.querySelector('.bar').style.setProperty('--cap', CartridgePixel.sprite(KEY, this));
    }

    // The two actions in the bar's font and one width, as action.rs's action_pair: each given
    // the room of the wider label's box and the wider hint's box of the two.
    pair() {
      const actions = [...this.shadowRoot.querySelectorAll('slot')]
        .map(slot => slot.assignedElements()[0])
        .filter(action => action?.boxes);
      const font = this.getAttribute('font') === 'thin' ? 'thin' : 'bold';
      for (const action of actions) {
        if (action.getAttribute('font') !== font) action.setAttribute('font', font);
      }
      const boxes = actions.map(action => action.boxes());
      const room = [0, 1].map(i => Math.max(...boxes.map(box => box[i]))).join(' ');
      for (const action of actions) {
        if (action.getAttribute('room') !== room) action.setAttribute('room', room);
      }
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'hint') hints(this.shadowRoot.querySelector('.hints'), value);
      else this.pair();
    }
  });
})();
