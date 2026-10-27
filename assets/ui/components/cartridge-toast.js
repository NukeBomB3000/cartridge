/*
  <cartridge-toast>: a word on what just happened, as a lone panel says it: the panel's title tab
  naming what it is about, then one line in its well. It comes and goes on its own over a game,
  the shelf or a menu: SAVE STATE "State saved", LINK "Link was ended", COLOUR "Correction on".

    <cartridge-toast label="SAVE STATE" text="State saved"></cartridge-toast>

  label   what it is about, in the title tab, in capitals whatever case it is given in.
  text    the line, in its own case.

  It is an <cartridge-panel>, active, as wide as the wider of its header and its line. The tab hugs
  its title, 4 pixels either side as every tab's title has, its slanted end after it, and the
  strip takes whatever room the line leaves. The line starts in line with the title, 3 pixels
  into the well as every note does, in cream, and its capitals have 4 rows of well above them
  and 4 below. It is 34 art pixels tall. Areas, each a div that ::part() reaches: toast (the
  panel), line.

  Needs everything in foundation/ on the page (see README.md) and cartridge-panel.js; its colours
  follow palette.js.
*/
(() => {
  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, as in a panel. A tab
         width of nothing leaves the tab as wide as its title and the room round it. */
      :host {
        display: inline-flex;
        flex-direction: column;
        text-rendering: geometricPrecision;
        --cartridge-tab-width: 0px;
      }
      :host([hidden]) { display: none; }
      cartridge-panel { flex: none; }
      /* the well keeps 2 rows above the line; one more under it than the panel's 2, so the
         capitals, rows 2 to 8 of the line, have 4 rows of well on either side */
      .line { white-space: nowrap; padding-bottom: var(--u); }
    </style>
    <cartridge-panel active part="toast"><div class="line" part="line"></div></cartridge-panel>`;

  customElements.define('cartridge-toast', class extends CartridgePixel.Element {
    static observedAttributes = ['label', 'text'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      const label = (this.getAttribute('label') ?? '').toUpperCase();
      this.shadowRoot.querySelector('cartridge-panel').setAttribute('label', label);
      this.shadowRoot.querySelector('.line').textContent = this.getAttribute('text') ?? '';
    }

    attributeChangedCallback() {
      if (this.isConnected) this.paint();
    }
  });
})();
