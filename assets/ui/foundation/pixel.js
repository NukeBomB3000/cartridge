/*
  The pixel-art kit the cartridge components draw with. A sprite is written as rows of colour
  keys, one character to an art pixel; the kit paints it into a PNG one pixel at a time, and the
  components show it at --u per pixel with image-rendering: pixelated, so every edge is a whole art
  pixel and nothing is ever smoothed. A sprite that has to fit any size is used as a border image
  (a 9-slice): its corners stay exactly as drawn and the row and column between them stretch.

  Sprites are drawn in the --oz-* colours palette.js sets, read off the component; a component
  built on CartridgePixel.Element draws its sprites again whenever palette.js changes them.

  Load it after tokens.css and before palette.js and the components.
*/
window.CartridgePixel = (() => {
  // what each key is drawn in; '.' is left empty
  const KEYS = {
    k: '--oz-dark',
    e: '--oz-edge',
    w: '--oz-well',
    p: '--oz-plate',
    t: '--oz-tab',
    b: '--oz-band',
    s: '--oz-stripe',
    d: '--oz-dim',
    A: '--oz-frame',
    H: '--oz-frame-hi',
    L: '--oz-label',
    c: '--oz-cream',
  };

  // `rows` as a CSS url() of a PNG, one image pixel to a character, in the colours `from` sees.
  function sprite(rows, from) {
    const style = getComputedStyle(from);
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, ...rows.map(row => row.length));
    canvas.height = Math.max(1, rows.length);
    const ctx = canvas.getContext('2d');
    const colours = {};
    rows.forEach((row, y) => [...row].forEach((key, x) => {
      if (key === '.') return;
      ctx.fillStyle = colours[key] ??= style.getPropertyValue(KEYS[key]).trim();
      ctx.fillRect(x, y, 1, 1);
    }));
    return `url("${canvas.toDataURL()}")`;
  }

  // A sprite symmetric both ways, from its top-left quarter. The quarter's last row and column are
  // the sprite's middle ones, which both halves share (and a 9-slice stretches).
  function mirror(quarter) {
    const rows = quarter.map(row => row + [...row.slice(0, -1)].reverse().join(''));
    return [...rows, ...rows.slice(0, -1).reverse()];
  }

  // Each row turned around, for the other side's sprite.
  function flip(rows) {
    return rows.map(row => [...row].reverse().join(''));
  }

  // Pixel Code is drawn on a grid of nine pixels to the em: two rows for accents over the
  // capitals, seven of capitals, two for descenders.
  const ACCENT = 2;
  const CAP = 7;
  const DESCENT = 2;

  // `value` in Pixel Code at one face pixel to a sprite pixel, as rows of '#' (ink) and '.', each
  // letter `advance` pixels after the one before (6 is the face's own; its letters are 5 wide).
  // The capitals are rows ACCENT to ACCENT + CAP - 1. Needs the face loaded (see fontReady).
  function letters(value, advance = 6) {
    const chars = [...value];
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, chars.length * advance);
    canvas.height = ACCENT + CAP + DESCENT;
    const ctx = canvas.getContext('2d');
    ctx.font = '9px "Pixel Code"';
    chars.forEach((ch, i) => ctx.fillText(ch, i * advance, ACCENT + CAP));
    const { data, width } = ctx.getImageData(0, 0, canvas.width, canvas.height);
    return Array.from({ length: canvas.height }, (_, y) =>
      Array.from({ length: width }, (_, x) => (data[(y * width + x) * 4 + 3] >= 128 ? '#' : '.')).join(''));
  }

  // Whether Pixel Code is ready to draw into a canvas; if not, `then` runs once it is.
  function fontReady(then) {
    if (document.fonts.check('9px "Pixel Code"')) return true;
    document.fonts.load('9px "Pixel Code"').then(then);
    return false;
  }

  // What a component drawn with the kit extends: paint() runs when it is put on the page and
  // again whenever palette.js gives whatever holds it new colours.
  class Element extends HTMLElement {
    connectedCallback() {
      this.paint();
      document.addEventListener('cartridge-palette', this);
    }

    disconnectedCallback() {
      document.removeEventListener('cartridge-palette', this);
    }

    handleEvent(event) {
      if (event.detail.root.contains(this)) this.paint();
    }

    paint() {}
  }

  return { sprite, mirror, flip, letters, fontReady, ACCENT, CAP, Element };
})();
