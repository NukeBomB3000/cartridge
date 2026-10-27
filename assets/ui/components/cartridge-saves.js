/*
  <cartridge-saves>: which save state the switcher shows (MENU ×2 in a game), over its screenshot: a
  panel titled SAVE STATES, how long ago the save shown was made in its well, and under that a dot
  for each save the cart has, newest first, the one shown lit.

    <cartridge-saves age="4 min ago" count="4" at="1"></cartridge-saves>

  age    how long ago the save shown was made, as the switcher says it: "Just now", "4 min ago",
         "3 hr ago", or its day and time, "2026-08-08 20:00", once it is half a day old.
  count  how many saves there are, a dot for each; a cart keeps up to 10. Default 1.
  at     which of them is shown, from 0, the newest: its dot is lit. Default 0.
  label  the title in the tab. Default SAVE STATES.

  It is an <cartridge-panel> whose tab is only as wide as its title (--cartridge-tab-width: 0), so it is
  as wide as the title, the age or the dots need, and 41 art pixels tall. In the well the age, in
  cream, on a line of 10 that starts in line with the title; then, a row past the line (two clear
  under its capitals, where a descender reaches), the dots: each 5 art pixels round and 3 apart,
  the one shown lit in --oz-cream and the rest in --oz-plate, what is off on a row; and under them
  as much well as there is over the age's capitals. Give it a width to hold it still while what it
  says changes. Areas, each a div that ::part() reaches: panel (the <cartridge-panel>), age, dots.

  Needs everything in foundation/ and cartridge-panel.js on the page (see README.md); its colours
  follow palette.js.
*/
(() => {
  // A dot, 5 pixels round: '#' is drawn lit or not.
  const DOT = [
    '.###.',
    '#####',
    '#####',
    '#####',
    '.###.',
  ];
  // How far apart two dots are.
  const GAP = 3;

  // `n` dots in a row, the one at `at` lit.
  function dots(n, at) {
    return DOT.map(row => Array.from({ length: n }, (_, i) =>
      row.replace(/#/g, i === at ? 'c' : 'p')).join('.'.repeat(GAP)));
  }

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, as in a panel */
      :host { display: inline-flex; text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      /* the panel takes whatever width the page gives, and otherwise what it holds needs */
      cartridge-panel { flex: 1 0 auto; --cartridge-tab-width: 0px; }
      /* the age on the well's line of 10, its capitals on rows 2 to 8 */
      .age { white-space: nowrap; }
      /* a row past the age's line, and two rows of well under them, so the well shows as much
         under the dots as over the capitals */
      .dots {
        height: calc(${DOT.length} * var(--u));
        margin: var(--u) 0 calc(2 * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }
    </style>
    <cartridge-panel active part="panel">
      <div class="age" part="age"></div>
      <div class="dots" part="dots"></div>
    </cartridge-panel>`;

  customElements.define('cartridge-saves', class extends CartridgePixel.Element {
    static observedAttributes = ['label', 'age', 'count', 'at'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
      // the panel made now, not once it is on the page, so paint() can draw it
      customElements.upgrade(this.shadowRoot);
    }

    paint() {
      const panel = this.shadowRoot.querySelector('cartridge-panel');
      panel.setAttribute('label', this.getAttribute('label') ?? 'SAVE STATES');
      // the panel is inside this one's shadow, where a new palette does not reach it: draw it again
      panel.paint();
      this.shadowRoot.querySelector('.age').textContent = this.getAttribute('age') ?? '';

      const n = Math.max(1, Math.round(+(this.getAttribute('count') ?? 1)) || 1);
      const at = Math.min(n - 1, Math.max(0, Math.round(+(this.getAttribute('at') ?? 0)) || 0));
      const d = this.shadowRoot.querySelector('.dots');
      d.style.width = `calc(${n * (DOT[0].length + GAP) - GAP} * var(--u))`;
      d.style.backgroundImage = CartridgePixel.sprite(dots(n, at), this);
    }

    attributeChangedCallback() {
      if (this.isConnected) this.paint();
    }
  });
})();
