/*
  Stand-ins for the three components the new menus are built from, which are being drawn for the
  kit and are not in components/ yet:

    <cartridge-row label value lit carets opens dim>         a setting: label left, value right
    <cartridge-list-row label value signal="0-3" lock lit>   a Wi-Fi network, its name in its own case
    <cartridge-bar hint>                                     the bottom bar, holding the two actions
      <action-left slot="left"></action-left>
      <action-right slot="right"></action-right>
    </cartridge-bar>

  flow.html loads components/cartridge-row.js, cartridge-list-row.js and cartridge-bar.js first (they are not
  found until the real ones are there) and this file after them. Each stand-in is defined only when
  the real component is not, so the page works today and draws with the real ones the moment they
  arrive. They keep to the kit's rules (README.md), plainly: colours from palette.js, sizes in whole
  art pixels, Pixel Code at 18px on a 20px line, and what the face has no letter for (◀ ▶ ›, the
  signal bars, the padlock) as sprites.
*/
(() => {
  const define = (name, element) => {
    if (!customElements.get(name)) customElements.define(name, element);
  };

  // Glyphs as tall as a line of text, 10 rows, the ink on the capitals' rows, 2 to 8. ◀ and › are
  // the full face's own (crates/cartridge-ui/assets/PixelCode-Regular.ttf), five rows on the baseline.
  const line = rows => ['', '', ...rows, ''];
  const CARET = line(['', '', '....#', '..###', '#####', '..###', '....#']);
  const OPENS = line(['', '', '#..', '.#.', '..#', '.#.', '#..']);
  const LOCK = line(['.###.', '#...#', '#...#', '#####', '##.##', '##.##', '#####']);
  // Three bars, 3, 5 and 7 tall, standing on the capitals' baseline: '1', '2', '3' mark each bar's
  // pixels so the bars past the signal can be drawn dark.
  const BARS = line(['......33', '......33', '...22.33', '...22.33', '11.22.33', '11.22.33', '11.22.33']);

  const pad = (rows, width) => rows.map(row => row.padEnd(width, '.'));
  const ink = (rows, key) => pad(rows, Math.max(...rows.map(r => r.length))).map(row => row.replaceAll('#', key));

  // The row both stand-ins share: 13 art pixels tall, its text's line one pixel down so the
  // capitals have three rows of room above them and three below, inside the cursor's band.
  const ROW_CSS = `
    :host { display: block; }
    :host([hidden]) { display: none; }
    .row {
      box-sizing: border-box;
      display: flex;
      height: calc(13 * var(--u));
      padding: var(--u) calc(3 * var(--u)) 0;
      font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
      color: var(--oz-cream);
      white-space: nowrap;
    }
    :host([lit]) .row { background: var(--oz-tab); }
    :host([dim]) .row { color: var(--oz-dim); }
    .label { flex: 1; min-width: 0; overflow: hidden; }
    .value { flex: none; margin-left: calc(6 * var(--u)); }
    :host(:not([value])) .value { display: none; }
    .glyph {
      flex: none;
      height: calc(10 * var(--u));
      background: no-repeat 0 0 / 100% 100%;
      image-rendering: pixelated;
    }`;

  const ROW = document.createElement('template');
  ROW.innerHTML = `
    <style>${ROW_CSS}
      /* ◀ VALUE ▶ with three pixels between each glyph and the value's ink; the value's last
         letter brings one of them, its blank column */
      .caret { width: calc(5 * var(--u)); }
      .caret.left { margin: 0 calc(3 * var(--u)) 0 calc(6 * var(--u)); }
      .caret.right { margin-left: calc(2 * var(--u)); }
      :host([carets]) .value { margin-left: 0; }
      .opens { width: calc(3 * var(--u)); margin-left: calc(2 * var(--u)); }
      :host(:not([carets])) .caret, :host(:not([opens])) .opens { display: none; }
    </style>
    <div class="row" part="row">
      <div class="label" part="label"></div>
      <div class="glyph caret left"></div>
      <div class="value" part="value"></div>
      <div class="glyph caret right"></div>
      <div class="glyph opens"></div>
    </div>`;

  define('cartridge-row', class extends CartridgePixel.Element {
    static observedAttributes = ['label', 'value', 'dim'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(ROW.content.cloneNode(true));
    }

    paint() {
      const { sprite, flip } = CartridgePixel;
      const $ = name => this.shadowRoot.querySelector(name);
      // the glyphs follow the text: bright, or the stripe's tone when the row is dimmed
      const key = this.hasAttribute('dim') ? 's' : 'H';
      $('.caret.left').style.backgroundImage = sprite(ink(CARET, key), this);
      $('.caret.right').style.backgroundImage = sprite(flip(ink(CARET, key)), this);
      $('.opens').style.backgroundImage = sprite(ink(OPENS, key), this);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'dim') this.paint();
      else this.shadowRoot.querySelector(`.${name}`).textContent = value ?? '';
    }
  });

  const LIST_ROW = document.createElement('template');
  LIST_ROW.innerHTML = `
    <style>${ROW_CSS}
      /* VALUE, the padlock and the bars, three pixels apart; the padlock keeps its room when the
         network has none, so the bars stand in one column */
      .lock { width: calc(5 * var(--u)); margin-left: calc(2 * var(--u)); }
      :host(:not([value])) .lock { margin-left: calc(6 * var(--u)); }
      :host(:not([lock])) .lock { visibility: hidden; }
      .bars { width: calc(8 * var(--u)); margin-left: calc(3 * var(--u)); }
    </style>
    <div class="row" part="row">
      <div class="label" part="label"></div>
      <div class="value" part="value"></div>
      <div class="glyph lock" part="lock"></div>
      <div class="glyph bars" part="signal"></div>
    </div>`;

  define('cartridge-list-row', class extends CartridgePixel.Element {
    static observedAttributes = ['label', 'value', 'signal', 'dim'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(LIST_ROW.content.cloneNode(true));
    }

    paint() {
      const { sprite } = CartridgePixel;
      const $ = name => this.shadowRoot.querySelector(name);
      const dim = this.hasAttribute('dim');
      const signal = Math.max(0, Math.min(3, Number(this.getAttribute('signal')) || 0));
      const bars = pad(BARS, 8).map(row => row.replace(/[123]/g, n => (n <= signal && !dim ? 'c' : 'e')));
      $('.lock').style.backgroundImage = sprite(ink(LOCK, dim ? 's' : 'c'), this);
      $('.bars').style.backgroundImage = sprite(bars, this);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'signal' || name === 'dim') this.paint();
      else this.shadowRoot.querySelector(`.${name}`).textContent = value ?? '';
    }
  });

  const BAR = document.createElement('template');
  BAR.innerHTML = `
    <style>
      :host { display: block; height: calc(15 * var(--u)); }
      :host([hidden]) { display: none; }
      .bar { position: relative; display: flex; height: 100%; }
      ::slotted(*) { position: relative; z-index: 1; flex: none; }
      /* nothing between the actions but the hint: whatever is behind the bar shows through */
      .middle { flex: 1; min-width: 0; }
      /* the key hint, across the whole bar so it is centred on the screen (an even number of art
         pixels wide, so the centred word starts on a whole one), its capitals on rows 5 to 11 like
         the hints in the actions' pills */
      .hint {
        position: absolute;
        z-index: 2;
        left: 0;
        right: 0;
        top: calc(3 * var(--u));
        text-align: center;
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        color: var(--oz-cream);
        white-space: pre;
        pointer-events: none;
      }
    </style>
    <div class="bar" part="bar">
      <slot name="left"></slot>
      <div class="middle" part="middle"></div>
      <slot name="right"></slot>
      <div class="hint" part="hint"></div>
    </div>`;

  define('cartridge-bar', class extends CartridgePixel.Element {
    static observedAttributes = ['hint'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(BAR.content.cloneNode(true));
    }

    attributeChangedCallback(_name, _old, value) {
      this.shadowRoot.querySelector('.hint').textContent = value ?? '';
    }
  });
})();
