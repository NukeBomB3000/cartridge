/*
  <cartridge-pill>: the bottom bar's button pill standing on its own, both its ends rounded, a word in
  its dark middle. On the shelf it is the shelf's system, GBA, GB or GBC, between the bar's two
  actions, and for a while the letter Up or Down jumped to.

    <cartridge-pill text="GBA"></cartridge-pill>
    <cartridge-bar>
      <action-left slot="left" label="CONTROLS" hint="SELECT"></action-left>
      <cartridge-pill slot="middle" text="GBA" centred></cartridge-pill>
      <action-right slot="right" label="SETTINGS" hint="START"></action-right>
    </cartridge-bar>

  text     the word, in Pixel Code one face pixel to an art pixel, in cream; give it in capitals.
           Without one nothing is drawn, as the shelf shows no pill until it knows its system.
  centred  the pill in the middle of what holds it, half of what is over before it rounded up to
           a whole art pixel. Across a screen, or in the bar between its two actions (which are
           one width), that puts its middle column on the screen's, as bar.rs does.

  It is <action-left>'s pill with its square edge rounded off as its other end is: the rounded end
  mirrored, and the column between them stretched. 15 art pixels tall, the bar's full height, and
  as wide as its word: the end's frame column, 3 dark pixels, the letters (6 pixels each, the last
  of the 6 blank), 2 dark pixels more and the other end's frame column, so 3 dark pixels either
  side of the ink. The capitals are on rows 5 to 11, the rows the bar's buttons print their hints
  on. crates/cartridge-ui/src/kit/action.rs's pill_face draws the same pixels. Areas, each a div that
  ::part() reaches: pill, text.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // The bar's button pill (action-left.js): its left edge, the column that stretches (frame colour
  // on top, a band, a frame line, dark, frame colour at the bottom), then the rounded end.
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
  const END = 4;
  // Standing on its own: the rounded end turned round for its left end, the column that stretches,
  // the rounded end. A 9-slice cut END from either side.
  const ROWS = PILL.map(row => {
    const end = row.slice(2, 2 + END);
    return [...end].reverse().join('') + row[1] + end;
  });

  // Sizes in art pixels: the bar's height; the dark before the letters, after the end's frame
  // column, and after them, past their own blank column; how far down the word's line starts.
  const H = 15;
  const BEFORE = 3;
  const AFTER = 2;
  const TEXT_TOP = 3;
  // Pixel Code's letters are 6 pixels apart, the last of the 6 blank.
  const ADVANCE = 6;

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: inline-flex; vertical-align: top; text-rendering: geometricPrecision; }
      :host([centred]) { display: flex; }
      :host([hidden]), :host(:not([text])), :host([text=""]) { display: none; }

      /* --w is the pill's width in art pixels, which paint() works out from the word */
      .pill {
        flex: none;
        box-sizing: border-box;
        width: calc(var(--w) * var(--u));
        height: calc(${H} * var(--u));
        border-style: solid;
        border-color: transparent;
        border-width: 0 calc(${END} * var(--u));
        border-image-source: var(--pill);
        border-image-slice: 0 ${END} fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      /* across what holds it, half of what is over first, rounded up: the pill is always an odd
         number of pixels wide, and rounded up its middle column is the middle one of an even
         width, as bar.rs's out_w / 2 - w / 2 has it */
      :host([centred]) .pill { margin-left: round(up, (100% - var(--w) * var(--u)) / 2, var(--u)); }

      /* the word's line starts on row 3, so its capitals are on rows 5 to 11, and it starts
         straight after the end's frame column and its 3 dark pixels, where the border ends */
      .text {
        margin-top: calc(${TEXT_TOP} * var(--u));
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        color: var(--oz-cream);
        white-space: nowrap;
      }
    </style>
    <div class="pill" part="pill"><div class="text" part="text"></div></div>`;

  customElements.define('cartridge-pill', class extends CartridgePixel.Element {
    static observedAttributes = ['text'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
      this.layout();
    }

    paint() {
      this.shadowRoot.querySelector('.pill').style.setProperty('--pill', CartridgePixel.sprite(ROWS, this));
    }

    // The word, and the pill as wide as it: the frame column, the dark before it, its letters, the
    // dark after them and the other frame column.
    layout() {
      const text = this.getAttribute('text') ?? '';
      this.shadowRoot.querySelector('.text').textContent = text;
      const w = 1 + BEFORE + [...text].length * ADVANCE + AFTER + 1;
      this.shadowRoot.querySelector('.pill').style.setProperty('--w', w);
    }

    attributeChangedCallback() {
      this.layout();
    }
  });
})();
