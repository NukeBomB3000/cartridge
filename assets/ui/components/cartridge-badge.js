/*
  <cartridge-badge>: what a game is doing that lasts, in the corner over it: fast forward and how
  fast, or a link to another handheld. It is the bottom bar's key cap holding a 7x7 sprite and,
  after it, a word.

    <cartridge-badge icon="fastforward" text="×4"></cartridge-badge>          held
    <cartridge-badge icon="fastforward" text="×4" lit></cartridge-badge>      latched
    <cartridge-badge icon="link" lit></cartridge-badge>                       hosting a link

  icon    the sprite, as tall as the capitals: fastforward, link, unlinked (a link that was
          lost).
  text    a word after it, in Pixel Code one face pixel to an art pixel: the speed, "×4". None
          without it.
  lit     drawn the way a lit row's band is: the tab colour with a frame line round it, its
          corner pixels left out. Unlit it is the bar's dark key cap.

  It is 9 art pixels tall, as tall as a key cap in the bar: a row above the capitals and a row
  below. The sprite starts two pixels in, the cap's end column and one more, the word 3 pixels
  after the sprite, and on the right the word's own blank pixel and the end column make the same
  two (a pixel more after the sprite when there is no word). Sprite and word are in cream. Areas,
  each a div that ::part() reaches: badge, icon, word.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // The cap as a 9-slice: dark, its four corner pixels left out, as the bar's key cap.
  const CAP = [
    '.k',
    'kk',
  ];
  // Lit: a lit row's band, the frame line round the tab colour, its corner pixels left out.
  const BAND = [
    '.A',
    'At',
  ];

  // How tall the badge is, and the room between the sprite and the word, in art pixels.
  const H = 9;
  const GAP = 3;

  // Each sprite as rows of '#', drawn in cream.
  const ICONS = {
    fastforward: [
      '#..#...',
      '##.##..',
      '######.',
      '#######',
      '######.',
      '##.##..',
      '#..#...',
    ],
    link: [
      '.......',
      '.##.##.',
      '#.....#',
      '#.###.#',
      '#.....#',
      '.##.##.',
      '.......',
    ],
    unlinked: [
      '...#...',
      '.##.##.',
      '#.....#',
      '#.....#',
      '#.....#',
      '.##.##.',
      '...#...',
    ],
  };

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, as in the bar */
      :host { display: inline-flex; text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .badge {
        display: flex;
        box-sizing: border-box;
        height: calc(${H} * var(--u));
        padding-left: var(--u);
        border: var(--u) solid transparent;
        border-image-slice: 1 fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      .icon {
        flex: none;
        width: calc(7 * var(--u));
        height: calc(${CartridgePixel.CAP} * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }
      /* with no word after it, the sprite keeps a pixel to its right, as the word's blank one */
      .badge.bare .icon { margin-right: var(--u); }
      /* Pixel Code one face pixel to an art pixel on a line of 10, raised two rows inside a box
         of 7, so the line's capitals, its rows 2 to 8, are all that shows */
      .word {
        height: calc(${CartridgePixel.CAP} * var(--u));
        margin-left: calc(${GAP} * var(--u));
        overflow: hidden;
        white-space: nowrap;
        color: var(--oz-cream);
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
      }
      .word span { display: block; margin-top: calc(-${CartridgePixel.ACCENT} * var(--u)); }
      .badge.bare .word { display: none; }
    </style>
    <div class="badge" part="badge">
      <div class="icon" part="icon"></div>
      <div class="word" part="word"><span></span></div>
    </div>`;

  customElements.define('cartridge-badge', class extends CartridgePixel.Element {
    static observedAttributes = ['icon', 'text', 'lit'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      const part = name => this.shadowRoot.querySelector(`.${name}`);
      const text = this.getAttribute('text') ?? '';
      const badge = part('badge');
      badge.style.borderImageSource = sprite(mirror(this.hasAttribute('lit') ? BAND : CAP), this);
      badge.classList.toggle('bare', !text);
      part('word').firstElementChild.textContent = text;
      const rows = ICONS[this.getAttribute('icon')] ?? ICONS.fastforward;
      part('icon').style.backgroundImage = sprite(rows.map(r => r.replace(/#/g, 'c')), this);
    }

    attributeChangedCallback() {
      if (this.isConnected) this.paint();
    }
  });
})();
