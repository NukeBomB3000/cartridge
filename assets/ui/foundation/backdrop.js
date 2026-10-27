/*
  The shelf's backdrops, as crates/cartridge-ui/src/backdrop.rs draws them. Each pattern is painted as
  one tile, a period of it, one image pixel to an art pixel (to a 2x2 bit for Bits), in the
  backdrop colour palette.js picks, and handed to tokens.css as --oz-pattern-<style>. A page shows
  one with the cartridge-backdrop class and data-style, Grid without one:

    <div class="cartridge-backdrop" data-style="rivets" data-moving></div>

  palette.js paints the tiles again whenever it changes the colour. Load it after pixel.js and
  before palette.js.
*/
window.CartridgeBackdrop = (() => {
  // Settings > Theme > Background in the device's order, less Wallpaper, a picture of the player's.
  const STYLES = ['grid', 'plates', 'dotted', 'dither', 'rivets', 'bits', 'solid'];
  // A tiled pattern's square in art pixels, one period: 40px.
  const TILE = 20;
  // Bits' tile in bits, one period: 80px.
  const BITS = 20;
  // Ordered dither thresholds, out of 16.
  const BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

  // A plate lit from the top left: a light edge above and to the left, a dark one below and to the
  // right, and a face a shade above the colour so the dark edge shows even on Night.
  function plate(tx, ty) {
    const top = ty === 0, left = tx === 0, bottom = ty === TILE - 1, right = tx === TILE - 1;
    if ((top && right) || (bottom && left)) return 0.045;
    if (top || left) return 0.13;
    if (bottom || right) return -0.6;
    return 0.045;
  }

  // How far the art pixel (tx, ty) of a tiled pattern's square lifts the colour (see tint).
  function tileLift(style, tx, ty) {
    const last = TILE - 1;
    switch (style) {
      case 'plates':
        return plate(tx, ty);
      case 'rivets': {
        const at = v => v === 2 || v === last - 2;
        if (at(tx) && at(ty)) return 0.2;
        if (at(tx - 1) && at(ty - 1)) return -0.6;
        return plate(tx, ty);
      }
      case 'dotted': {
        const cross = (tx === 0 && [0, 1, last].includes(ty)) || (ty === 0 && [1, last].includes(tx));
        if (cross) return 0.17;
        if ((tx === 0 && ty % 2 === 1) || (ty === 0 && tx % 2 === 1)) return 0.09;
        return 0;
      }
      case 'dither': {
        if (tx === 0 || ty === 0) return 0.1;
        const light = 1 - (tx + ty - 2) / (2 * (TILE - 2));
        return BAYER[ty % 4][tx % 4] < light * light * 11 ? 0.045 : 0;
      }
      default:
        return tx === 0 || ty === 0 ? 0.09 : 0;
    }
  }

  // How brightly the bit at (cx, cy) of Bits' tile is lit: a scatter that looks random but is the
  // same every time.
  function bitLift(cx, cy) {
    let h = (Math.imul(cx, 0x9e3779b1) ^ Math.imul(cy, 0x85ebca77)) >>> 0;
    h = (h ^ (h >>> 15)) >>> 0;
    h = Math.imul(h, 0x2c1b3c6d) >>> 0;
    h = (h ^ (h >>> 12)) >>> 0;
    const roll = h % 100;
    return roll <= 5 ? 0.07 : roll <= 8 ? 0.14 : 0;
  }

  // `rgb` lifted toward white by `lift`, or sunk toward black by as much when it is below 0.
  const tint = (rgb, lift) => rgb.map(v => Math.round(lift < 0 ? v * (1 + lift) : v + (255 - v) * lift));

  // One tile of `style` in the colour `hex`, as a CSS image: none for Solid.
  function tile(style, hex) {
    if (style === 'solid') return 'none';
    const base = [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16));
    const size = style === 'bits' ? BITS : TILE;
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    const ctx = canvas.getContext('2d');
    const image = ctx.createImageData(size, size);
    for (let y = 0; y < size; y++) {
      for (let x = 0; x < size; x++) {
        const lift = style === 'bits' ? bitLift(x, y) : tileLift(style, x, y);
        image.data.set([...tint(base, lift), 255], (y * size + x) * 4);
      }
    }
    ctx.putImageData(image, 0, 0);
    return `url("${canvas.toDataURL()}")`;
  }

  // Every style's tile on `root`, in the backdrop colour `hex`.
  function paint(root, hex) {
    for (const style of STYLES) root.style.setProperty(`--oz-pattern-${style}`, tile(style, hex));
  }

  return { STYLES, tile, paint };
})();
