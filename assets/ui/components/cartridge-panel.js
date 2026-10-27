/*
  <cartridge-panel>: one titled section inside an <cartridge-modal>, like BUTTON SETTING and ATTACK MODE.
  A dark title tab with a slanted end, the bright strip beside it, and a well below for content.

    <cartridge-panel label="BUTTON SETTING" active>
      ...whatever the section shows...
    </cartridge-panel>

  label   the title in the tab.
  active  draws the title light, as the section the cursor is in; without it the title is dimmed.

  It grows to share its parent's height (flex: 1 1 0), so two panels split a modal evenly unless
  one is given a bigger flex-grow. --cartridge-tab-width sets how wide the tab is (121 art pixels by
  default, and never narrower than its title). The well keeps 2 art pixels clear above its content
  and 3 at the sides, where text starts in line with the title; <cartridge-row>s and <cartridge-list-row>s
  in it reach a pixel into the sides, so a lit row's band has 2 pixels of well all round. Areas,
  each a div that ::part() reaches: panel, header, tab, label, slant, strip, body.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // The panel's corner: its stripe, with the corner pixel left out, around the frame colour.
  const PANEL = [
    '.s',
    'sA',
  ];
  // The well's corner: its shadow line, the same way, around the well.
  const WELL = [
    '.e',
    'ew',
  ];

  // How tall the header is in art pixels, its bottom stripe included, and the rows the tab and its
  // slant fill above that stripe.
  const H = 13;
  const ROWS = H - 1;

  // The tab's slanted end: one pixel further right each row, so it leans 45 degrees like the
  // game's. Row by row, the tab's fill (its top and bottom rows in the frame colour, as its
  // outline), a frame pixel, a stripe pixel, then nothing, where the strip shows.
  function slant() {
    return Array.from({ length: ROWS }, (_, y) =>
      (y === 0 || y === ROWS - 1 ? 'A' : 't').repeat(y) + 'As' + '.'.repeat(H - y - 2));
  }

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: flex; flex: 1 1 0; min-height: 0; min-width: 0; text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .panel {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        border: var(--u) solid transparent;
        border-image-slice: 1 fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }

      .header {
        flex: none;
        box-sizing: border-box;
        height: calc(${H} * var(--u));
        display: flex;
        background: var(--oz-frame-hi);
        border-bottom: var(--u) solid var(--oz-stripe);
      }
      /* the title starts 4 pixels in from the tab's left edge, in Pixel Code one face pixel to an
         art pixel (18px at 2px a pixel), on a line of 10 so its capitals sit on whole pixels, rows 2
         to 8 of the line */
      .tab {
        flex: none;
        box-sizing: border-box;
        width: var(--cartridge-tab-width, calc(121 * var(--u)));
        min-width: max-content;
        display: flex;
        padding: 0 calc(4 * var(--u));
        background: var(--oz-tab);
        border: var(--u) solid var(--oz-frame);
        border-right: 0;
      }
      .label {
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        white-space: nowrap;
        color: var(--oz-dim);
      }
      :host([active]) .label { color: var(--oz-cream); }
      .slant {
        flex: none;
        width: calc(${H} * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }
      .strip { flex: 1; }

      /* under the header: the stripe, one frame line, the well's shadow line, the well */
      .body {
        flex: 1;
        min-height: 0;
        margin: var(--u);
        padding: calc(2 * var(--u)) calc(3 * var(--u));
        border: var(--u) solid transparent;
        border-image-slice: 1 fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
        color: var(--oz-cream);
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        overflow: hidden;
      }
      /* rows reach a pixel into the well's sides, so a lit row's band has two pixels of well
         around it on every side, as it has above the first row */
      ::slotted(cartridge-row), ::slotted(cartridge-list-row) { margin: 0 calc(-1 * var(--u)); }
    </style>
    <div class="panel" part="panel">
      <div class="header" part="header">
        <div class="tab" part="tab"><div class="label" part="label"></div></div>
        <div class="slant" part="slant"></div>
        <div class="strip" part="strip"></div>
      </div>
      <div class="body" part="body"><slot></slot></div>
    </div>`;

  customElements.define('cartridge-panel', class extends CartridgePixel.Element {
    static observedAttributes = ['label'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      const part = name => this.shadowRoot.querySelector(`.${name}`);
      part('panel').style.borderImageSource = sprite(mirror(PANEL), this);
      part('body').style.borderImageSource = sprite(mirror(WELL), this);
      part('slant').style.backgroundImage = sprite(slant(), this);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'label') this.shadowRoot.querySelector('.label').textContent = value ?? '';
    }
  });
})();
