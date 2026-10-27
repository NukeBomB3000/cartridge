/*
  The art flow.html draws itself: placeholders for what Rust draws on the device (the shelf's carts
  and the top bar's icons), the keyboard's keys, and the arrows the Controls panel
  needs and the … a note may end with, which fonts.css's Pixel Code, cut to Latin-1, has no letter
  for (◀ ▶ ▲ ▼ …). Like the kit's components, every shape is a sprite written as rows of colour
  keys (see foundation/pixel.js), shown at one art pixel to --u, in the --oz-* colours palette.js
  sets.

  FlowArt.paint(root) puts every sprite on `root` as a CSS custom property, --flow-<name>, which
  flow.css hangs where it is drawn; call it again whenever palette.js changes the colours.

  Load it after the foundation.
*/
window.FlowArt = (() => {
  // A glyph stands in for a letter, so it is one Pixel Code character cell: 6 art pixels across
  // (5 of ink and a blank column, as the face's letters are) and 10 down (its 20px line). The ink is
  // the full face's own (crates/cartridge-ui/assets/PixelCode-Regular.ttf), pixel for pixel: five rows
  // standing on the baseline, the capitals' rows 4 to 8, as the device prints them.
  const cell = rows => ['', '', '', '', ...rows, ''].map(row => row.padEnd(6, '.'));
  const GLYPHS = {
    left: cell(['....#', '..###', '#####', '..###', '....#']),
    right: cell(['#', '###', '#####', '###', '#']),
    up: cell(['..#', '..#', '.###', '.###', '#####']),
    down: cell(['#####', '.###', '.###', '..#', '..#']),
  };
  // The … that ends a note while something is under way (Looking for devices…), its three dots
  // on the baseline row, drawn in the notes' dim colour.
  const NOTE_GLYPHS = {
    ellipsis: cell(['', '', '', '', '#.#.#']),
  };

  // The shelf's top bar, after status_bar.rs: Wi-Fi on the left, the battery on the right, each one
  // sprite cell (7 art pixels) tall.
  const WIFI = [
    '..###..',
    '.#...#.',
    '#.....#',
    '..###..',
    '.#...#.',
    '.......',
    '...#...',
  ];
  // 12 across with its nub, the charge a bar inside a one-pixel wall and a pixel of air.
  const BATTERY = [
    '############.',
    '#..........#.',
    '#.#######..##',
    '#.#######..##',
    '#.#######..##',
    '#..........#.',
    '############.',
  ];

  // A key of the keyboard: its corner, mirrored into a 9-slice, a dark outline with the corner
  // pixel left out around the key's face. The lit key has a bright face.
  const KEY = ['.k', 'kt'];
  const KEY_LIT = ['.k', 'kH'];

  // Rows with every '#' in the colour key `key`.
  const ink = (rows, key) => rows.map(row => row.replaceAll('#', key));

  // A GBA cart seen from the front, `w` by `h` art pixels: the shoulders across the top at full
  // width with two grip grooves, the body a little narrower under them, the label on the body, and
  // every corner cut in steps. `tones` names the keys it is drawn in.
  function cart(w, h, tones) {
    const shoulder = Math.round((h * 29) / 141);   // cart.svg: 29 of its 141 are shoulders
    const inset = Math.round((w * 6.5) / 240);     // and the body sits 6.5 of 240 in under them
    const inside = (x, y) => {
      if (y < 0 || y >= h) return false;
      const edge = y < shoulder ? 0 : inset;
      const cut = Math.max(3 - y, y - (h - 3), 0);
      return x >= edge + cut && x < w - edge - cut;
    };
    const grid = Array.from({ length: h }, (_, y) => Array.from({ length: w }, (_, x) => {
      if (!inside(x, y)) return '.';
      const rim = !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1);
      return rim ? tones.rim : tones.shell;
    }));
    const run = (y, from, to, key) => { for (let x = from; x < to; x++) grid[y][x] = key; };
    // the grooves, a third and two thirds down the shoulders
    for (const y of [Math.round(shoulder / 3), Math.round((2 * shoulder) / 3)]) run(y, 4, w - 4, tones.groove);
    // the label: an edge around the paper, then two lines standing in for its title
    const [lx, ly] = [inset + 7, shoulder + 5];
    const [lw, lh] = [w - 2 * lx, h - ly - 9];
    for (let y = ly; y < ly + lh; y++) {
      const border = y === ly || y === ly + lh - 1;
      run(y, lx, lx + lw, border ? tones.edge : tones.label);
      grid[y][lx] = grid[y][lx + lw - 1] = tones.edge;
    }
    for (const [dy, part] of [[6, 0.56], [11, 0.36]]) {
      for (let y = ly + dy; y < ly + dy + 3; y++) run(y, lx + 6, lx + 6 + Math.round(lw * part), tones.title);
    }
    return grid.map(row => row.join(''));
  }
  // The cart under the cursor, bigger and lit, and the ones beside it, quieter.
  const CART_LIT = { w: 180, h: 106, rim: 'H', shell: 'b', groove: 'p', edge: 'p', label: 'c', title: 't' };
  const CART_SIDE = { w: 140, h: 82, rim: 'k', shell: 'p', groove: 'w', edge: 'w', label: 'b', title: 'w' };

  function paint(root = document.documentElement) {
    const { sprite, mirror } = CartridgePixel;
    const set = (name, rows) => root.style.setProperty(`--flow-${name}`, sprite(rows, root));
    for (const [name, rows] of Object.entries(GLYPHS)) set(`glyph-${name}`, ink(rows, 'H'));
    for (const [name, rows] of Object.entries(NOTE_GLYPHS)) set(`glyph-${name}`, ink(rows, 'd'));
    set('wifi', ink(WIFI, 'c'));
    set('battery', ink(BATTERY, 'c'));
    set('key', mirror(KEY));
    set('key-lit', mirror(KEY_LIT));
    set('cart-lit', cart(CART_LIT.w, CART_LIT.h, CART_LIT));
    set('cart-side', cart(CART_SIDE.w, CART_SIDE.h, CART_SIDE));
  }

  return { GLYPHS, paint };
})();
