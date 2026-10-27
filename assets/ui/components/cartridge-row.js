/*
  <cartridge-row>: one setting inside an <cartridge-panel>'s well, its name on the left and its value on
  the right. The row the cursor is on is lit the way a panel's title tab is drawn: a band in the
  tab colour with a frame line round it and its corner pixels left out, the words in it light.

    <cartridge-panel label="CONFIGS" active>
      <cartridge-row label="FAST FORWARD" value="6X" lit carets></cartridge-row>
      <cartridge-row label="SHADER" value="SHARP" carets></cartridge-row>
      <cartridge-row label="CONTROLS" opens></cartridge-row>
    </cartridge-panel>

  label   the setting's name, from the left, in capitals whatever case it is given in.
  value   what it is set to, from the right, as given. A row may have none.
  lit     the row under the cursor: the band, the name in cream, the value and arrows bright.
  carets  a ◀ and a ▶ either side of the value while the row is lit, for a value Left and Right
          change. Their room is kept unlit too, so the value never moves as the cursor passes.
  opens   a › at the right end, lit or not, for a row that opens another screen.
  dim     a setting that is not available: everything in it drawn faint, lit or not, and no carets
          (its value cannot be changed).

  Without lit, carets or opens it is a plain name and value line, like SSH's IP 192.168.1.20.

  Unlit, the name is in the label colour and the value and arrows in the dim one; lit, the name is
  cream and the value and arrows in the frame's bright colour; dim, all of it is in the band colour,
  or the dim one when lit.

  It is 13 art pixels tall, as tall as a panel's header, and as wide as its parent (in a panel's
  well, a pixel wider each side: see cartridge-panel.js). Both words are Pixel Code one face pixel to
  an art pixel, on a line of 10 with their capitals on rows 3 to 9 of the row, three rows clear
  above and below; an accent or a descender still fits inside the band's frame line. Areas, each a
  div that ::part() reaches: row, label, caret-left, value, caret-right, opens. <cartridge-list-row>
  is a row for a network list, built on this one.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // How tall a row is in art pixels, and how far into it, under the band's frame line, the art
  // beside the words starts so that it stands on the capitals' rows.
  const H = 13;
  const ART_TOP = 2;

  // The band under the cursor, a 9-slice: the frame line, its corner pixel left out like a panel's,
  // around the tab colour.
  const BAND = [
    '.A',
    'At',
  ];

  // The ◀ beside a value Left and Right change, as tall as the capitals; '#' is drawn in the arrows'
  // colour. The ▶ is the same turned around.
  const CARET = [
    '...#',
    '..##',
    '.###',
    '####',
    '.###',
    '..##',
    '...#',
  ];

  // The › at the end of a row that opens another screen: a chevron two pixels thick, as tall as the
  // capitals and as wide as a caret.
  const OPENS = [
    '#...',
    '##..',
    '.##.',
    '..##',
    '.##.',
    '##..',
    '#...',
  ];

  // The colour keys (see pixel.js) a row's art is drawn in, for each way it can be lit: its name's
  // colour, its value's (which the arrows share) and, for art that shows a level, the part not
  // reached. The words' own colours are the same ones, set in the style below.
  const INK = {
    plain: { name: 'L', value: 'd', off: 'p' },
    lit: { name: 'c', value: 'H', off: 'w' },
    dim: { name: 'b', value: 'b', off: 'p' },
    dimLit: { name: 'd', value: 'd', off: 'w' },
  };

  // `rows` with every '#' drawn in the colour key `key`.
  const inked = (rows, key) => rows.map(row => row.replaceAll('#', key));

  // What every row draws with, <cartridge-list-row> included: the row, its band, its two words.
  const STYLE = `
    /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
       part of a pixel even where it sits on whole pixels */
    :host { display: block; height: calc(${H} * var(--u)); text-rendering: geometricPrecision; }
    :host([hidden]) { display: none; }

    /* The band's frame line is the row's border, there lit or not, so nothing moves when the
       cursor arrives: 4 pixels of band between it and the words on each side. */
    .row {
      box-sizing: border-box;
      height: 100%;
      display: flex;
      align-items: flex-start;
      padding: 0 calc(4 * var(--u));
      border: var(--u) solid transparent;
      border-image-slice: 1 fill;
      border-image-repeat: stretch;
      image-rendering: pixelated;
      font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
      white-space: nowrap;
    }
    :host([lit]) .row { border-image-source: var(--band); }

    /* the name takes what room is left and is cut off, one letter clear of the value, when it is
       longer than that */
    .label {
      flex: 1 1 auto;
      min-width: 0;
      margin-right: calc(6 * var(--u));
      overflow: hidden;
      color: var(--oz-label);
    }
    /* Every letter's last pixel is blank; the value hands that pixel back on its right, so its ink
       ends as far from what follows (or the band's end) as the name's starts from the band's. */
    .value { flex: none; margin-right: calc(-1 * var(--u)); color: var(--oz-dim); }
    .value:empty { display: none; }

    :host([lit]) .label { color: var(--oz-cream); }
    :host([lit]) .value { color: var(--oz-frame-hi); }
    :host([dim]) .label, :host([dim]) .value { color: var(--oz-band); }
    :host([dim][lit]) .label, :host([dim][lit]) .value { color: var(--oz-dim); }

    /* art beside the words (arrows, icons) is 7 pixels tall and stands on the capitals' rows */
    .art {
      flex: none;
      height: calc(7 * var(--u));
      margin-top: calc(${ART_TOP} * var(--u));
      background: no-repeat 0 0 / 100% 100%;
      image-rendering: pixelated;
    }
  `;

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      ${STYLE}
      .label { text-transform: uppercase; }
      /* the arrows, 3 pixels clear of the value and of each other */
      .caret, .opens { display: none; width: calc(4 * var(--u)); }
      .caret-left { margin-right: calc(3 * var(--u)); }
      .caret-right, .opens { margin-left: calc(3 * var(--u)); }
      :host([carets]) .caret { display: block; visibility: hidden; }
      :host([carets][lit]:not([dim])) .caret { visibility: inherit; }
      :host([opens]) .opens { display: block; }
    </style>
    <div class="row" part="row">
      <div class="label" part="label"></div>
      <div class="art caret caret-left" part="caret-left"></div>
      <div class="value" part="value"></div>
      <div class="art caret caret-right" part="caret-right"></div>
      <div class="art opens" part="opens"></div>
    </div>`;

  class CartridgeRow extends CartridgePixel.Element {
    static observedAttributes = ['label', 'value', 'lit', 'dim'];
    static template = template;
    static STYLE = STYLE;

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(this.constructor.template.content.cloneNode(true));
    }

    // One of its areas, by class.
    area(name) {
      return this.shadowRoot.querySelector(`.${name}`);
    }

    // The colour keys its art is drawn in, for how it is lit now (see INK).
    ink() {
      const lit = this.hasAttribute('lit');
      if (this.hasAttribute('dim')) return lit ? INK.dimLit : INK.dim;
      return lit ? INK.lit : INK.plain;
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      this.area('row').style.setProperty('--band', sprite(mirror(BAND), this));
      this.paintArt(this.ink());
    }

    // The arrows, in the value's colour.
    paintArt(ink) {
      const { sprite, flip } = CartridgePixel;
      const caret = inked(CARET, ink.value);
      this.area('caret-left').style.backgroundImage = sprite(caret, this);
      this.area('caret-right').style.backgroundImage = sprite(flip(caret), this);
      this.area('opens').style.backgroundImage = sprite(inked(OPENS, ink.value), this);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'label' || name === 'value') this.area(name).textContent = value ?? '';
      else if (this.isConnected) this.paintArt(this.ink());
    }
  }

  customElements.define('cartridge-row', CartridgeRow);
})();
