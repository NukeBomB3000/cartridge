/*
  <cartridge-modal>: the window of the Mega Man Zero menus, and nothing else. It draws the frame
  and stacks whatever it is given top to bottom, so the same modal holds the Button Setting screen
  today and any other screen later. Usually what it holds is <cartridge-panel>s, which grow to share
  its height (give one a bigger `flex-grow` to make it taller); two panels share one stripe.

    <cartridge-modal>
      <cartridge-panel label="BUTTON SETTING" active></cartridge-panel>
      <cartridge-panel label="ATTACK MODE"></cartridge-panel>
    </cartridge-modal>

  It fills the box the page gives it; set its width and height from outside. Areas, each a div
  that ::part() reaches: frame, content.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // The frame's top-left corner, drawn pixel by pixel and mirrored into the other three: a dark
  // outline, a bright line, a stripe, then the frame colour the panels sit on. The last row and
  // column are the straight edge between corners.
  const CORNER = [
    '...kk',
    '.kkHH',
    '.kHss',
    'kHsAA',
    'kHsAA',
  ];
  const CUT = CORNER.length - 1;

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      :host { display: block; box-sizing: border-box; }
      :host([hidden]) { display: none; }

      .frame {
        box-sizing: border-box;
        height: 100%;
        border: calc(${CUT} * var(--u)) solid transparent;
        border-image-slice: ${CUT} fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      /* The panels reach two pixels up into the frame, so the first one's stripe lands on the
         frame's own and its header starts right under it, as in the game. */
      .content {
        height: calc(100% + 2 * var(--u));
        margin-top: calc(-2 * var(--u));
        display: flex;
        flex-direction: column;
      }
      ::slotted(:not(:first-child)) { margin-top: calc(-1 * var(--u)); }
    </style>
    <div class="frame" part="frame">
      <div class="content" part="content"><slot></slot></div>
    </div>`;

  customElements.define('cartridge-modal', class extends CartridgePixel.Element {
    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      this.shadowRoot.querySelector('.frame').style.borderImageSource = sprite(mirror(CORNER), this);
    }
  });
})();
