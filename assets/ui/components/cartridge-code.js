/*
  <cartridge-code>: lines to read or type exactly, like a card's folders or a command: a dark box
  inside an edge line with its corner pixels left out, the lines in cream Pixel Code, kept as
  they are written and scrolled sideways inside the box when one is longer than it is wide, so
  the page never is. The lines are the page's own <pre>, put inside it.

    <cartridge-code label="SD1">
      <pre>System/
Games/GBA/</pre>
    </cartridge-code>
    <cartridge-code label="TERMINAL" copy><pre>ssh root@rg35xxsp.local</pre></cartridge-code>

  label  a name for what the lines are, in the dim colour along the top, with an edge line
         under it.
  copy   a COPY key at the top right that puts the lines on the clipboard, and says COPIED.

  It is as wide as its parent and as tall as its lines. Each line is 12 art pixels, its capitals
  on rows 3 to 9; 5 pixels of dark above the first line's capitals and below the last's, and 5
  either side. The top row, with a label or a COPY key, is 13 pixels tall, its words' capitals on
  rows 3 to 9 as a row's are. Areas, each a div that ::part() reaches: code, top, label, copy,
  lines. How the <pre> is drawn is the component's, marked important so the page's own rules for
  <pre> (a box, a font) cannot reach it.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // The box's corner, mirrored into the other three: the edge line, its corner pixel left out,
  // round the dark.
  const BOX = [
    '.e',
    'ek',
  ];
  // The COPY key's ends, 9 pixels tall: the tab colour, its corner pixels left out (k shows the
  // box under them), and the column between that stretches.
  const KEY = ['.t.', ...Array(7).fill('ttt'), '.t.'];

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: block; min-width: 0; text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .code {
        border: var(--u) solid transparent;
        border-image-slice: 1 fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        color: var(--oz-cream);
      }

      /* the top row: the label from the left, the key at the right, both on line 1 of 13 so their
         capitals are on rows 3 to 9; the edge line under it */
      .top {
        display: none;
        height: calc(13 * var(--u));
        box-sizing: border-box;
        padding: var(--u) calc(4 * var(--u)) 0 calc(5 * var(--u));
        border-bottom: var(--u) solid var(--oz-edge);
        white-space: nowrap;
      }
      :host([label]) .top, :host([copy]) .top { display: flex; }
      .label { flex: 1; min-width: 0; overflow: hidden; color: var(--oz-dim); }

      /* COPY in a key 9 pixels tall round its capitals, a row above and below them, 3 pixels of the
         key either side of the letters' ink (on the right, the letters' own blank pixel is one) */
      .copy {
        all: unset;
        display: none;
        box-sizing: border-box;
        height: calc(9 * var(--u));
        margin-top: var(--u);
        padding: 0 var(--u) 0 calc(2 * var(--u));
        line-height: calc(6 * var(--u));
        border-style: solid;
        border-color: transparent;
        border-width: var(--u);
        border-image-source: var(--key);
        border-image-slice: 1 fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
        color: var(--oz-cream);
        cursor: pointer;
      }
      :host([copy]) .copy { display: block; }
      .copy:hover, .copy:focus-visible { border-image-source: var(--key-lit); color: var(--oz-dark); }
      .copy:focus-visible { outline: var(--u) solid var(--oz-cream); outline-offset: var(--u); }

      /* the lines, 12 pixels each: 2 pixels of dark over the first line (whose capitals start 3
         rows down it) and 3 under the last (which has 2 rows under its capitals) */
      .lines {
        overflow-x: auto;
        padding: calc(2 * var(--u)) calc(5 * var(--u)) calc(3 * var(--u));
        line-height: calc(12 * var(--u));
        scrollbar-color: var(--oz-plate) transparent;
      }
      ::slotted(pre) {
        all: unset !important;
        display: block !important;
        white-space: pre !important;
      }
      .status { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
    </style>
    <div class="code" part="code">
      <div class="top" part="top">
        <div class="label" part="label"></div>
        <button class="copy" part="copy" type="button">COPY</button>
      </div>
      <div class="lines" part="lines"><slot></slot></div>
    </div>
    <span class="status" role="status"></span>`;

  customElements.define('cartridge-code', class extends CartridgePixel.Element {
    static observedAttributes = ['label'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
      this.area('copy').addEventListener('click', () => this.copy());
    }

    // One of its areas, by class.
    area(name) {
      return this.shadowRoot.querySelector(`.${name}`);
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      this.area('code').style.borderImageSource = sprite(mirror(BOX), this);
      const key = this.area('copy').style;
      key.setProperty('--key', sprite(KEY, this));
      key.setProperty('--key-lit', sprite(KEY.map(row => row.replaceAll('t', 'H')), this));
    }

    // The lines on the clipboard, and COPIED on the key for a moment.
    async copy() {
      const text = this.shadowRoot.querySelector('slot').assignedNodes().map(node => node.textContent).join('').trim();
      try {
        await navigator.clipboard.writeText(text);
      } catch {
        return;
      }
      const key = this.area('copy');
      key.textContent = 'COPIED';
      this.area('status').textContent = 'Copied';
      clearTimeout(this.copied);
      this.copied = setTimeout(() => {
        key.textContent = 'COPY';
        this.area('status').textContent = '';
      }, 1500);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'label') this.area('label').textContent = value ?? '';
    }
  });
})();
