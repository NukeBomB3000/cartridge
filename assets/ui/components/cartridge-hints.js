/*
  <cartridge-hints>: a screen's keys on one line of the kit, on a ground of their own, for where the
  bottom bar's hints would sit on a picture: a title tab naming what they are for, then a well
  holding the keys, each its key in a small dark cap and then the word, as the bar draws its
  hints. Put one just above the bar over a game or a photo, where the bar's own hints would be
  hard to read.

    <cartridge-hints label="NETWORK" hint="X SCAN | Y FORGET"></cartridge-hints>

  label   the name in the tab, in capitals whatever case it is given in.
  hint    the keys, as cartridge-bar's hint: a key and what it does, the first word the key,
          "X SCAN"; more than one with a | between them, "X SCAN | Y FORGET".

  It is drawn as <cartridge-gauge>, with the keys in the well for its cells: 15 art pixels tall, as
  tall as the bottom bar; a panel's frame line round it, the tab with its title in the middle and
  the panel's 45 degree slant, and beside it, on the strip's bright colour, the well, inset a pixel
  above, below and at the end. Inside the well's shadow line, 2 pixels clear, the keys, 2 pixels
  clear after their last ink. Their capitals stand on the title's rows, so each key's cap, a row
  above and below them, fills the well's inside from top to bottom. The keys are 8 pixels apart,
  a cap's letters 2 dark pixels in from its ends, the word 3 pixels after the cap.

  It is as wide as all that. Given a width, the well takes what is over, the keys at its left, so
  it can keep the room of its widest keys and not move as one comes and goes. Areas, each a div
  that ::part() reaches: frame, tab, label, slant, strip, well, hints, hint, key, word.

  Needs everything in foundation/, cartridge-gauge.js and cartridge-bar.js on the page (see README.md),
  whose chrome and hints it is drawn with; its colours follow palette.js.
*/
(() => {
  const template = document.createElement('template');
  template.innerHTML = `
    <style>${CartridgeGauge.STYLE}${CartridgeBar.HINTS}
      /* given a width, the strip and its well take what the tab leaves */
      .gauge, .strip, .well { flex-grow: 1; }
      /* The keys' capitals a row into the well's inside, on the title's rows: their line starts
         ACCENT rows above them. Their box is as wide as their ink, so the well shows 2 pixels
         after the last word's ink as before the first cap. */
      .hints {
        flex: none;
        align-self: flex-start;
        width: calc(var(--ink, 0) * var(--u));
        margin-top: calc(${1 - CartridgePixel.ACCENT} * var(--u));
      }
    </style>${CartridgeGauge.markup('frame', '<div class="hints" part="hints"></div>')}`;

  customElements.define('cartridge-hints', class extends CartridgePixel.Element {
    static observedAttributes = ['label', 'hint'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      CartridgeGauge.paint(this.shadowRoot, this.getAttribute('label') ?? '');
      const box = this.shadowRoot.querySelector('.hints');
      box.style.setProperty('--cap', CartridgePixel.sprite(CartridgeBar.KEY, this));
      CartridgeBar.hints(box, this.getAttribute('hint'));
    }

    attributeChangedCallback() {
      if (this.isConnected) this.paint();
    }
  });
})();
