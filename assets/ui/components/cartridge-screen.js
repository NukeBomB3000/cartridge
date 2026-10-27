/*
  <cartridge-screen>: a picture of a screen in a handheld's bezel. Round it, the shell: a dark
  outline, a rim in the frame colour, the plate colour; inside that the black bezel, its corner
  pixels left out, and the glass's edge line round the picture. The picture is the page's own
  image (or video), put inside it, so it keeps its alt text.

    <cartridge-screen><img src="media/shelf.png" alt="The shelf…" width="640" height="480"></cartridge-screen>
    <cartridge-screen led><img …></cartridge-screen>

  led  the power light, lit, in the bezel under the picture's bottom left corner, which is then
       4 pixels deeper; the light sits in the middle of its black, in line with the glass.

  It is as wide as the page makes it and as tall as the picture at that width, plus the bezel,
  the picture's height rounded to a whole pixel so nothing under it lands between two. The
  bezel is 10 art pixels on each side: the shell 5, the black 4 and the glass's edge 1 (the
  bottom 4 more with led), so a 640×480 picture at its own size makes a 680×500 screen (508 with
  led), and cartridge's 2×2 art pixels stay 2×2 on the page. The picture is drawn pixelated
  where it is shown at least at its own size, each of its pixels a block, and smoothed where it is
  shown smaller, so its text stays legible. Areas, each a div that ::part() reaches: shell, bezel,
  glass, led.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // The shell's top-left corner, mirrored into the other three: the dark outline, the rim in the
  // frame colour, then the plate. The last row and column are the straight edge between corners.
  const SHELL = [
    '...kkk',
    '.kkAAA',
    '.kAppp',
    'kApppp',
    'kApppp',
    'kApppp',
  ];
  const CUT = SHELL.length - 1;

  // The bezel's corner: black, its corner pixel left out.
  const BEZEL = [
    '.k',
    'kk',
  ];
  // How much black is round the glass, and how much more under it with the power light.
  const BLACK = 4;
  const DEEPER = 4;

  // The power light, lit: a pill 3 pixels wide and 2 tall, bright on top.
  const LED = [
    'cHH',
    'HHb',
  ];

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      :host { display: block; box-sizing: border-box; }
      :host([hidden]) { display: none; }

      .shell, .bezel {
        border-style: solid;
        border-color: transparent;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      .shell {
        border-width: calc(${CUT} * var(--u));
        border-image-slice: ${CUT} fill;
      }
      .bezel {
        position: relative;
        padding: calc(${BLACK - 1} * var(--u));
        border-width: var(--u);
        border-image-slice: 1 fill;
      }
      :host([led]) .bezel { padding-bottom: calc(${BLACK + DEEPER - 1} * var(--u)); }
      .glass { border: var(--u) solid var(--oz-edge); background: var(--oz-dark); }
      ::slotted(img), ::slotted(video), ::slotted(picture) {
        display: block;
        width: 100%;
        height: auto;
      }
      .led {
        display: none;
        position: absolute;
        left: calc(${BLACK - 1} * var(--u));
        bottom: calc(${(BLACK + DEEPER - 2) / 2 - 1} * var(--u));
        width: calc(3 * var(--u));
        height: calc(2 * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }
      :host([led]) .led { display: block; }
    </style>
    <div class="shell" part="shell">
      <div class="bezel" part="bezel">
        <div class="glass" part="glass"><slot></slot></div>
        <div class="led" part="led"></div>
      </div>
    </div>`;

  customElements.define('cartridge-screen', class extends CartridgePixel.Element {
    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
      // pixelated at its own size or bigger, smoothed smaller: again whenever its size changes
      this.resized = new ResizeObserver(() => this.scale());
      this.shadowRoot.addEventListener('slotchange', () => this.scale());
    }

    connectedCallback() {
      super.connectedCallback();
      this.resized.observe(this);
      this.addEventListener('load', this, true);
    }

    disconnectedCallback() {
      super.disconnectedCallback();
      this.resized.disconnect();
      this.removeEventListener('load', this, true);
    }

    handleEvent(event) {
      if (event.type === 'load') this.scale();
      else super.handleEvent(event);
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      const part = name => this.shadowRoot.querySelector(`.${name}`);
      part('shell').style.borderImageSource = sprite(mirror(SHELL), this);
      part('bezel').style.borderImageSource = sprite(mirror(BEZEL), this);
      part('led').style.backgroundImage = sprite(LED, this);
    }

    // Each picture a whole number of pixels tall at the width it is shown, pixelated when it is
    // shown with at least one screen pixel to each of its own, smoothed when it is shown smaller.
    scale() {
      const pictures = this.shadowRoot.querySelector('slot').assignedElements()
        .flatMap(el => (el.matches('img, video') ? [el] : [...el.querySelectorAll('img')]));
      for (const picture of pictures) {
        const [w, h] = [picture.naturalWidth || picture.videoWidth, picture.naturalHeight || picture.videoHeight];
        const width = picture.getBoundingClientRect().width;
        if (!w || !h || !width) continue;
        picture.style.height = `${Math.round((width * h) / w)}px`;
        picture.style.imageRendering = width * devicePixelRatio >= w - 0.5 ? 'pixelated' : 'auto';
      }
    }
  });
})();
