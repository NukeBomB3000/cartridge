/*
  <cartridge-card>: one thing told about in a few lines, like a feature: its icon in a dark box,
  its title beside it and its text under both, inside an edge line with its corner pixels left
  out. The title and the text are the page's own, put inside it, so a title can be a real
  heading.

    <cartridge-panel label="HIGHLIGHTS" active>
      <cartridge-card icon="cart">
        <h3 slot="title">The shelf</h3>
        <p>Solid carts in their own plastics, with real label scans.</p>
      </cartridge-card>
    </cartridge-panel>

  icon  what the box shows, an 11×11 sprite in the palette's colours: cart, chip, rewind, link,
        window, gamepad, wifi, bluetooth or theme (the palette's own four lightest steps).
        Without it, or with one not known, there is no box and the title starts at the left.

  Slots: title, for the title (a heading, set in capitals in Pixel Code, cream), and the default
  one for the text (paragraphs, in Pixel Code on a line of 14 art pixels, in the label colour).
  The title's and the paragraphs' font, colour and margins are the card's, marked important so
  the page's rules for headings and paragraphs cannot reach them (Pixel Code has no bold of its
  own: a heading's would smear it off the grid).

  It is as wide as its parent and as tall as its text. The box is 17 art pixels square, the icon in
  its middle; the title's capitals are on the box's middle rows, 5 to 11, 5 pixels clear of it, and
  the text starts 3 pixels under the box. 4 pixels of room inside the edge line on every side but
  the top, which has 3, the box's own dark round its icon making up the rest. Areas, each a div
  that ::part() reaches: card, icon, title, text.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // The edge line's corner, mirrored into the other three, with nothing inside it.
  const EDGE = [
    '.e',
    'e.',
  ];

  // The icon's box, 17 pixels square: dark, its corner pixels left out.
  const BOX = 17;
  const PAD = (BOX - 11) / 2;

  // The icons, 11 pixels square, in the colour keys of pixel.js; '.' shows the box.
  const ICONS = {
    // a Game Boy cart: the corner cut off its top, its label with a stripe across the top, and
    // the arrow under it
    cart: [
      '.AAAAAAA...',
      '.AbbbbbbA..',
      '.AbbbbbbbA.',
      '.AbHHHHHbA.',
      '.AbcccccbA.',
      '.AbcccccbA.',
      '.AbcccccbA.',
      '.AbbbbbbbA.',
      '.AbbAAAbbA.',
      '.AbbbAbbbA.',
      '.AAAAAAAAA.',
    ],
    // a chip, its pins on every side
    chip: [
      '...........',
      '...H.H.H...',
      '..HHHHHHH..',
      'HHHbbbbbHHH',
      '..HbcccbH..',
      'HHHbcbcbHHH',
      '..HbcccbH..',
      'HHHbbbbbHHH',
      '..HHHHHHH..',
      '...H.H.H...',
      '...........',
    ],
    // rewind: two heads pointing back
    rewind: [
      '...........',
      '....H.....H',
      '...HH....HH',
      '..HHH...HHH',
      '.HHHH..HHHH',
      'HHHHH.HHHHH',
      '.HHHH..HHHH',
      '..HHH...HHH',
      '...HH....HH',
      '....H.....H',
      '...........',
    ],
    // a link: two arrows, there and back
    link: [
      '...........',
      '.......H...',
      '.......HH..',
      'HHHHHHHHHH.',
      '.......HH..',
      '.......H...',
      '...H.......',
      '..HH.......',
      '.HHHHHHHHHH',
      '..HH.......',
      '...H.......',
    ],
    // a menu window: its title tab and the strip beside it, a lit row and two more
    window: [
      'AAAAAAAAAAA',
      'AttttAHHHHA',
      'AAAAAAAAAAA',
      'AwwwwwwwwwA',
      'AtcccccctwA',
      'AwwwwwwwwwA',
      'AwLLLLwwwwA',
      'AwwwwwwwwwA',
      'AwLLLLLLwwA',
      'AwwwwwwwwwA',
      'AAAAAAAAAAA',
    ],
    // a gamepad: the d-pad's cross and two buttons cut out of it, and its grips
    gamepad: [
      '...........',
      '...........',
      '.HHHHHHHHH.',
      'HH.HHHHHHHH',
      'H...HHHH.HH',
      'HH.HHHH.HHH',
      'HHHHHHHHHHH',
      'HHHH...HHHH',
      'HHH.....HHH',
      '...........',
      '...........',
    ],
    // Wi-Fi: two arcs over the dot, as the top bar draws it
    wifi: [
      '...........',
      '..HHHHHHH..',
      '.H.......H.',
      'H..HHHHH..H',
      '..H.....H..',
      '.H..HHH..H.',
      '...H...H...',
      '...........',
      '.....H.....',
      '....HHH....',
      '.....H.....',
    ],
    // the Bluetooth rune
    bluetooth: [
      '.....H.....',
      '.....HH....',
      '.....H.H...',
      '...H.H..H..',
      '....HH.H...',
      '.....HH....',
      '....HH.H...',
      '...H.H..H..',
      '.....H.H...',
      '.....HH....',
      '.....H.....',
    ],
    // the palette's four lightest steps, which a Theme colour turns
    theme: [
      'ccccc.HHHHH',
      'ccccc.HHHHH',
      'ccccc.HHHHH',
      'ccccc.HHHHH',
      'ccccc.HHHHH',
      '...........',
      'AAAAA.bbbbb',
      'AAAAA.bbbbb',
      'AAAAA.bbbbb',
      'AAAAA.bbbbb',
      'AAAAA.bbbbb',
    ],
  };

  // The box with `icon` in its middle.
  function boxed(icon) {
    return Array.from({ length: BOX }, (_, y) => Array.from({ length: BOX }, (_, x) => {
      const corner = (x === 0 || x === BOX - 1) && (y === 0 || y === BOX - 1);
      if (corner) return '.';
      const key = icon[y - PAD]?.[x - PAD];
      return key && key !== '.' ? key : 'k';
    }).join(''));
  }

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: block; min-width: 0; text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .card {
        box-sizing: border-box;
        height: 100%;
        display: grid;
        grid-template-columns: auto minmax(0, 1fr);
        grid-template-areas: "icon title" "text text";
        align-content: start;
        padding: calc(3 * var(--u)) calc(4 * var(--u)) calc(4 * var(--u));
        border: var(--u) solid transparent;
        border-image-slice: 1;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      .icon {
        grid-area: icon;
        width: calc(${BOX} * var(--u));
        height: calc(${BOX} * var(--u));
        margin-right: calc(5 * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }
      .icon[hidden] { display: none; }
      /* the title's line starts 3 pixels down the box, so its capitals (rows 2 to 8 of it) are on
         the box's rows 5 to 11, around its middle */
      .title {
        grid-area: title;
        align-self: start;
        margin-top: calc(3 * var(--u));
        min-width: 0;
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        color: var(--oz-cream);
        text-transform: uppercase;
      }
      .icon[hidden] + .title { margin-top: 0; }
      ::slotted([slot="title"]) {
        margin: 0 !important;
        font: inherit !important;
        letter-spacing: normal !important;
        color: inherit !important;
        text-transform: inherit !important;
      }
      /* the text 3 pixels under the box, on lines of 14 so its capitals still sit on whole
         pixels, a paragraph from the next half a line apart */
      .text {
        grid-area: text;
        display: grid;
        row-gap: calc(7 * var(--u));
        min-width: 0;
        margin-top: calc(3 * var(--u));
        font: 400 calc(9 * var(--u)) / calc(14 * var(--u)) var(--oz-font);
        color: var(--oz-label);
      }
      ::slotted(:not([slot])) {
        margin: 0 !important;
        min-width: 0;
        font: inherit !important;
        letter-spacing: normal !important;
        color: inherit !important;
      }
    </style>
    <div class="card" part="card">
      <div class="icon" part="icon" aria-hidden="true"></div>
      <div class="title" part="title"><slot name="title"></slot></div>
      <div class="text" part="text"><slot></slot></div>
    </div>`;

  customElements.define('cartridge-card', class extends CartridgePixel.Element {
    static observedAttributes = ['icon'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      this.shadowRoot.querySelector('.card').style.borderImageSource = sprite(mirror(EDGE), this);
      this.paintIcon();
    }

    // The icon's box, or none.
    paintIcon() {
      const box = this.shadowRoot.querySelector('.icon');
      const icon = ICONS[this.getAttribute('icon')];
      box.hidden = !icon;
      box.style.backgroundImage = icon ? CartridgePixel.sprite(boxed(icon), this) : '';
    }

    attributeChangedCallback() {
      if (this.isConnected) this.paintIcon();
    }
  });
})();
