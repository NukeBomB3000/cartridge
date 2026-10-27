/*
  <action-right>: the bottom bar's right action, as the game's R1 BUTTON / ELF LIST. <action-left>
  mirrored: left to right, a button pill with a hint, a big label, and the arrow in its notch at
  the screen's edge. It comes without text; give it some with attributes when it is used:

    <action-right></action-right>
    <action-right label="SETTINGS" hint="START"></action-right>

  label  the big word beside the arrow, drawn as a sprite: the capitals of Pixel Code made bold
         the pixel-art way (drawn again a pixel to the right), the rows of their bars doubled so
         they stand 10 pixels tall, with a shadow down and to the right.
  font   how the label is drawn, as Settings > Theme > Font picks: "bold" (or none), as above,
         or "thin", in Pixel Code's own one-pixel strokes, its letters a pixel closer.
  hint   the smaller word in the button pill, in Pixel Code.
  room   how wide the label's box and the hint's box are at least, in art pixels: "54 30". Each
         word stays in the middle of its box. <cartridge-bar> gives both its actions the room of the
         wider label and the wider hint, so the two come out one width.

  Everything is pixel art at one art pixel to --u, text included. It is 15 art pixels tall and as
  wide as its text (with room kept for it when there is none). The notch rises 4 pixels above it,
  over whatever is there, as the game's does over the backdrop. Areas, each a div that ::part()
  reaches: action, button, hint, label, word, icon.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // Sizes in art pixels: the bar's height, how far the notch rises above it, the notch's width.
  const BAR = 15;
  const RISE = 4;
  const NOTCH = 11;
  const TALL = BAR + RISE;
  // where the bracket over the notch starts down toward the bar's top rail
  const KNEE = 3;

  // A return arrow: a big head at the top pointing in, toward the label, on a thick band that runs
  // back, turns down and comes round underneath. Drawn as <action-left>'s and mirrored, so here it
  // points left and turns on the right.
  const ARROW = [
    '.....#..',
    '.....##.',
    '..######',
    '.#######',
    '##...##.',
    '##...#..',
    '##......',
    '.#####..',
    '..####..',
  ];

  // The button pill, the bar's full height, drawn as <action-left>'s and turned around when it is
  // used: its square edge, the column that stretches (frame colour on top, a band, a frame line,
  // dark, frame colour at the bottom), then the rounded end, which ends up on the left.
  const PILL = [
    'AAA...',
    'AbbAA.',
    'AAAAAA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkkkA',
    'AkkAA.',
    'AAA...',
  ];
  const PILL_END = 4;

  // The label's box: 3 dark pixels before the word and 2 after its shadow column, and how wide it
  // is kept when there is no word.
  const WORD_PAD = [3, 2];
  const WORD_MIN = 32;
  // The hint's box: never narrower than HINT_MIN, with this room before and after its text; how far
  // it reaches into the pill's rounded end, and how far down the bar its line starts.
  const HINT_MIN = 30;
  const HINT_PAD = [3, 2];
  const HINT_REACH = 3;
  const HINT_TOP = 3;
  // Pixel Code's letters are 6 pixels apart, the last of the 6 blank; a bold label's 7.
  const ADVANCE = 6;
  const advance = font => (font === 'thin' ? ADVANCE : ADVANCE + 1);
  // The rows of the capitals word() doubles: where Pixel Code's letters have their bars, the top,
  // the middle and the bottom.
  const BAR_ROWS = [0, 3, 6];

  // The notch as rows of colour keys (see pixel.js): the bracket over it, the dark inside, the
  // arrow over its shadow. Drawn left to right as <action-left>'s and turned around at the end.
  function notch() {
    const grid = Array.from({ length: TALL }, () => Array(NOTCH).fill('.'));
    const run = (y, from, to, key) => {
      for (let x = Math.max(0, from); x < Math.min(NOTCH, to); x++) grid[y][x] = key;
    };
    // the bracket's frame line: along the top, down 45 degrees, then along the bar's rail
    const line = r => (r === 0 ? [0, KNEE + 1] : r < RISE ? [KNEE + r, KNEE + r + 2] : [KNEE + RISE, NOTCH]);
    for (let y = 2; y < TALL - 1; y++) {
      const stripeFrom = y <= RISE ? KNEE + y - 1 : y === RISE + 1 ? KNEE + RISE : NOTCH;
      run(y, 0, stripeFrom, 'k');
    }
    for (let r = 0; r <= RISE; r++) run(r, ...line(r), 'A');
    // the stripe under it: where the line was a row above and is not on this row
    for (let r = 1; r <= RISE + 1; r++) {
      const [from, to] = line(r - 1);
      for (let x = from; x < to; x++) if (grid[r][x] !== 'A') grid[r][x] = 's';
    }
    run(TALL - 1, 0, NOTCH, 'A');

    // the arrow over its own shadow, one pixel down and right, for some depth, in the middle of
    // the bar between its rail and its bottom line
    const arrow = (dx, dy, key) => ARROW.forEach((row, y) => [...row].forEach((ch, x) => {
      if (ch === '#') grid[RISE + 3 + y + dy][1 + x + dx] = key;
    }));
    arrow(1, 1, 's');
    arrow(0, 0, 'H');
    return CartridgePixel.flip(grid.map(row => row.join('')));
  }

  // The label as sprite rows. The capitals of Pixel Code drawn one face pixel to a pixel; in bold
  // each letter drawn again a pixel to its right (Pixel Code's own bold is fattened by part of a
  // pixel, off the grid), so 6 wide and a pixel clear of the next, and thin as they are, 5 wide
  // and a pixel clear; the rows of the letters' bars doubled (BAR_ROWS), so the capitals stand 10
  // tall; and a shadow one pixel down and right of all of it, in the well's colour. 11 rows: the
  // capitals and the shadow's row under them.
  function word(value, font) {
    const { ACCENT, CAP } = CartridgePixel;
    const ink = CartridgePixel.letters(value, advance(font));
    const rows = Array.from({ length: CAP }, (_, i) => Array(BAR_ROWS.includes(i) ? 2 : 1).fill(ACCENT + i)).flat();
    const inked = (x, y) => ink[rows[y]]?.[x] === '#';
    const at = (x, y) => inked(x, y) || (font !== 'thin' && inked(x - 1, y));
    return Array.from({ length: rows.length + 1 }, (_, y) =>
      Array.from({ length: ink[0].length }, (_, x) => (at(x, y) ? 'L' : at(x - 1, y - 1) ? 'w' : '.')).join(''));
  }

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: block; height: calc(${BAR} * var(--u)); text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .action { display: flex; height: 100%; }

      .icon { flex: none; position: relative; width: calc(${NOTCH} * var(--u)); }
      .icon .art {
        position: absolute; left: 0; bottom: 0;
        width: 100%; height: calc(${TALL} * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }

      /* the bar's top rail (frame colour over a stripe) and bottom line run through the label; how
         wide it is, layout() works out */
      .label {
        flex: none;
        box-sizing: border-box;
        display: flex;
        padding-left: calc(${WORD_PAD[0]} * var(--u));
        background: var(--oz-dark);
        border-top: var(--u) solid var(--oz-frame);
        border-bottom: var(--u) solid var(--oz-frame);
        box-shadow: inset 0 var(--u) 0 var(--oz-stripe);
      }
      /* The word is a sprite (see word()), its top on the bar's row 3 so its capitals fill rows 3
         to 12: a dark row under the rail above them and, below, their shadow on the row over the
         bottom line. 3 pixels of dark before the letters and 2 after them, the first of the 2 the
         shadow; in a box made wider (room), half of what is over goes before them, rounded down. */
      .word {
        flex: none;
        margin-top: calc(2 * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }

      .button {
        flex: none;
        display: flex;
        border-style: solid;
        border-color: transparent;
        border-width: 0 var(--u) 0 calc(${PILL_END} * var(--u));
        border-image-slice: 0 1 0 ${PILL_END} fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      /* Pixel Code one face pixel to an art pixel, its capitals on rows 5 to 11: two dark rows each
         side, between the pill's band and its bottom. Across, the word sits in the middle of the
         pill's whole dark inside, the rounded end's included: the box reaches the 3 dark pixels
         into that end. layout() sets how wide it is and how far in the word starts, 3 pixels and
         half of what is over, rounded down, so the word is never on half a pixel. */
      .hint {
        box-sizing: border-box;
        margin: calc(${HINT_TOP} * var(--u)) 0 0 calc(-${HINT_REACH} * var(--u));
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        color: var(--oz-cream);
        white-space: nowrap;
      }
    </style>
    <div class="action" part="action">
      <div class="button" part="button">
        <div class="hint" part="hint"></div>
      </div>
      <div class="label" part="label"><div class="word" part="word" role="img"></div></div>
      <div class="icon" part="icon"><div class="art"></div></div>
    </div>`;

  customElements.define('action-right', class extends CartridgePixel.Element {
    static observedAttributes = ['label', 'font', 'hint', 'room'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
      this.layout();
    }

    paint() {
      const { sprite, flip } = CartridgePixel;
      const part = name => this.shadowRoot.querySelector(`.${name}`);
      part('icon .art').style.backgroundImage = sprite(notch(), this);
      part('button').style.borderImageSource = sprite(flip(PILL), this);
      this.paintLabel();
    }

    // How wide the label's box and the hint's box are for this action's own text, in art pixels:
    // what <cartridge-bar> works out its two actions' room from.
    boxes() {
      const length = name => [...(this.getAttribute(name) ?? '')].length;
      const label = length('label');
      return [
        label ? WORD_PAD[0] + label * advance(this.getAttribute('font')) + WORD_PAD[1] : WORD_MIN,
        Math.max(HINT_MIN, HINT_PAD[0] + length('hint') * ADVANCE + HINT_PAD[1]),
      ];
    }

    // The label's box and the hint's as wide as their text, or as the room given when that is
    // more, each word in the middle: half of what is over before it, rounded down to a whole pixel.
    layout() {
      const [word, hint] = this.boxes();
      const [wordRoom, hintRoom] = (this.getAttribute('room') ?? '').split(/\s+/).map(Number);
      const [wordW, hintW] = [Math.max(word, wordRoom || 0), Math.max(hint, hintRoom || 0)];
      const text = [...(this.getAttribute('hint') ?? '')].length * ADVANCE;
      const over = hintW - HINT_PAD[0] - HINT_PAD[1] - text;
      const px = n => `calc(${n} * var(--u))`;
      const part = name => this.shadowRoot.querySelector(`.${name}`);
      part('label').style.width = px(wordW);
      part('word').style.marginLeft = px(Math.floor((wordW - word) / 2));
      part('hint').style.width = px(hintW);
      part('hint').style.paddingLeft = px(HINT_PAD[0] + Math.floor(over / 2));
    }

    // The label's sprite, sized to it; nothing, and room kept, when there is no label.
    paintLabel() {
      if (!this.isConnected) return;
      const value = this.getAttribute('label') ?? '';
      const art = this.shadowRoot.querySelector('.word');
      art.setAttribute('aria-label', value);
      if (!value || !CartridgePixel.fontReady(() => this.paintLabel())) {
        art.style.width = art.style.height = art.style.backgroundImage = '';
        return;
      }
      const rows = word(value, this.getAttribute('font'));
      art.style.width = `calc(${rows[0].length} * var(--u))`;
      art.style.height = `calc(${rows.length} * var(--u))`;
      art.style.backgroundImage = CartridgePixel.sprite(rows, this);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'label' || name === 'font') this.paintLabel();
      if (name === 'hint') this.shadowRoot.querySelector('.hint').textContent = value ?? '';
      this.layout();
    }
  });
})();
