/*
  <cartridge-list-row>: one Wi-Fi network or Bluetooth device in a list, an <cartridge-row> with its name
  on the left and, on the right, what it is to the handheld and its icons: a network's padlock if
  it needs a password and how strong it comes in, or the kind of device it is.

    <cartridge-panel label="NETWORKS" active>
      <cartridge-list-row label="Home" value="CONNECTED" signal="3" lock lit></cartridge-list-row>
      <cartridge-list-row label="Office" value="SAVED" signal="1" lock></cartridge-list-row>
      <cartridge-list-row label="Café" signal="2"></cartridge-list-row>
    </cartridge-panel>

    <cartridge-panel label="DEVICES" active>
      <cartridge-list-row label="8BitDo SN30 Pro" value="CONNECTED" device="gamepad" lit></cartridge-list-row>
      <cartridge-list-row label="Keychron K3" device="keyboard"></cartridge-list-row>
    </cartridge-panel>

  label   the network's or the device's name, from the left, in its own case (names are the
          player's).
  value   what it is to the handheld, from the right: CONNECTED, JOINING, SAVED for a network;
          CONNECTED, CONNECTING, PAIRING, PAIRED for a device. Optional.
  signal  0 to 3: the Wi-Fi bars, that many of the three lit and the rest drawn faint. Without
          it there are no bars.
  lock    a padlock beside the bars, for a network with a password.
  device  audio, gamepad, keyboard or other: a Bluetooth device's row, with its kind's icon
          where a network's bars go (headphones, a gamepad, a keyboard, the Bluetooth rune for
          any other kind or one not known), and no signal or padlock.
  lit     the row under the cursor, as <cartridge-row>'s.
  dim     a network or device that cannot be picked, as <cartridge-row>'s.

  The icons stand on the capitals' rows. A network's padlock is 5 art pixels wide like a letter
  and its bars 7, 3 pixels apart and a letter's width (6) clear of the value, so they read as a
  pair of their own; their room is kept when a row has none, so every row's value, padlock and
  bars line up down the list. A device's row keeps no room for either: its icon stands where the
  bars end, in the middle of a room 9 pixels wide (the widest icon's width) a letter's width clear
  of the value, so a list of devices has its icons on one middle column and its values lined up.
  The icons are drawn in the name's colour. Everything else is <cartridge-row>'s: 13 art pixels tall,
  the same band, words and colours. Areas, each a div that ::part() reaches: row, label, value,
  lock, signal, device.

  Needs everything in foundation/ on the page (see README.md) and components/cartridge-row.js before it.
*/
(() => {
  const CartridgeRow = customElements.get('cartridge-row');

  // The top bar's Wi-Fi icon (WIFI in crates/cartridge-ui/src/status_bar.rs), its three parts numbered
  // in the order they light: 1 the dot, 2 the small arc, 3 the big one.
  const SIGNAL = [
    '..333..',
    '.3...3.',
    '3.....3',
    '..222..',
    '.2...2.',
    '.......',
    '...1...',
  ];

  // A padlock as tall as the capitals and as wide as a letter: the shackle, then the body with
  // its keyhole.
  const LOCK = [
    '.###.',
    '#...#',
    '#...#',
    '#####',
    '##.##',
    '##.##',
    '#####',
  ];

  // A Bluetooth device's icon for each kind, as tall as the capitals.
  const DEVICES = {
    // Headphones: the band over the head and two ear cups, each two pixels wide and three tall.
    audio: [
      '..###..',
      '.#...#.',
      '#.....#',
      '#.....#',
      '##...##',
      '##...##',
      '##...##',
    ],
    // A gamepad: its body with the d-pad's cross cut out on the left and a button on the right, as
    // far in from the right as the cross's middle is from the left, and the grips under them.
    gamepad: [
      '.#######.',
      '##.######',
      '#...##.##',
      '##.######',
      '#########',
      '###...###',
      '##.....##',
    ],
    // A keyboard: a block with two rows of keys cut out of it, the second half a key along, and
    // the space bar under them.
    keyboard: [
      '#########',
      '#.#.#.#.#',
      '#########',
      '##.#.#.##',
      '#########',
      '##.....##',
      '#########',
    ],
    // Any other device: the Bluetooth rune, its stem with a stroke from either end out to a point
    // on its right and from there across the stem to its left.
    other: [
      '..##.',
      '#.#.#',
      '.###.',
      '..#..',
      '.###.',
      '#.#.#',
      '..##.',
    ],
  };
  const NAMES = { audio: 'headphones', gamepad: 'gamepad', keyboard: 'keyboard', other: 'Bluetooth device' };

  // How wide a device's icon's room is, the gamepad's and the keyboard's width; each icon is drawn
  // in the middle of it.
  const DEVICE_W = 9;

  // `rows` in the middle of a sprite `width` wide.
  const centred = (rows, width) => rows.map(row => {
    const left = (width - row.length) / 2;
    return '.'.repeat(left) + row + '.'.repeat(width - row.length - left);
  });

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      ${CartridgeRow.STYLE}
      .lock { width: calc(5 * var(--u)); margin-left: calc(6 * var(--u)); }
      .signal { width: calc(7 * var(--u)); margin-left: calc(3 * var(--u)); }
      :host(:not([lock])) .lock, :host(:not([signal])) .signal { visibility: hidden; }
      /* a device's row: its icon a letter's width clear of the value, and no room for the others */
      .device { display: none; width: calc(${DEVICE_W} * var(--u)); margin-left: calc(6 * var(--u)); }
      :host([device]) .device { display: block; }
      :host([device]) .lock, :host([device]) .signal { display: none; }
    </style>
    <div class="row" part="row">
      <div class="label" part="label"></div>
      <div class="value" part="value"></div>
      <div class="art lock" part="lock" role="img" aria-label="needs a password"></div>
      <div class="art signal" part="signal" role="img"></div>
      <div class="art device" part="device" role="img"></div>
    </div>`;

  customElements.define('cartridge-list-row', class extends CartridgeRow {
    static observedAttributes = [...CartridgeRow.observedAttributes, 'signal', 'lock', 'device'];
    static template = template;

    // The bars lit, 0 to 3; none without a signal.
    level() {
      const level = Number.parseInt(this.getAttribute('signal'), 10);
      return Number.isNaN(level) ? null : Math.min(3, Math.max(0, level));
    }

    // The device's kind, other for one not known; none on a network's row.
    kind() {
      if (!this.hasAttribute('device')) return null;
      const kind = this.getAttribute('device');
      return Object.hasOwn(DEVICES, kind) ? kind : 'other';
    }

    // The padlock, the bars and a device's icon, in the name's colour, the bars not reached faint.
    paintArt(ink) {
      const { sprite } = CartridgePixel;
      const level = this.level() ?? 0;
      const bars = SIGNAL.map(row => row.replace(/\d/g, part => (part <= level ? ink.name : ink.off)));
      const signal = this.area('signal');
      signal.style.backgroundImage = sprite(bars, this);
      signal.setAttribute('aria-label', `signal ${level} of 3`);
      this.area('lock').style.backgroundImage = sprite(LOCK.map(row => row.replaceAll('#', ink.name)), this);
      const kind = this.kind() ?? 'other';
      const device = this.area('device');
      device.style.backgroundImage = sprite(centred(DEVICES[kind], DEVICE_W).map(row => row.replaceAll('#', ink.name)), this);
      device.setAttribute('aria-label', NAMES[kind]);
    }
  });
})();
