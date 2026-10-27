/*
  <cartridge-link>: the art of the link screen a game opens with SELECT + MENU, one piece at a time:
  the link port along the console's top edge, the GBA link cable's plug on its cable, the wireless
  adapter, the radio arcs it calls out with and the clicks of a plug going home. The screen moves
  them; this draws them.

    <cartridge-link piece="plug" role="host"></cartridge-link>
    <cartridge-link piece="cable" length="40"></cartridge-link>
    <cartridge-link piece="port" width="320" height="36"></cartridge-link>
    <cartridge-link piece="adapter"></cartridge-link>
    <cartridge-link piece="arc" ring="2" side="right"></cartridge-link>
    <cartridge-link piece="clicks"></cartridge-link>

  piece   which piece: plug, cable, port, adapter, arc or clicks.
  role    the plug's end of the cable: host (default), the real cable's purple and narrow, or
          join, grey and wider, with the socket a third player's cable goes into.
  length  how long the cable is in art pixels, from the top of the plug's neck up. Default 20.
  width   how wide the port's strip is, in art pixels: the screen's width. Default 160.
  height  how tall the strip is, from its top edge to the bottom of the screen. Default 36.
  ring    which radio arc, 0 the smallest, nearest the adapter, to 2. Default 0.
  side    the side of the adapter the arc is on: right (default) or left, the same turned round.

  Every piece is a sprite written here a character to an art pixel, drawn at --u per pixel. The
  plug and the adapter are real things and keep their own colours, the plug the purple of the
  GBA's link cable; the port, the arcs and the clicks are drawn in the --oz-* colours palette.js
  sets, so they follow Settings > Theme > Colour. The adapter has CART on its label in Pixel Code,
  a face pixel to an art pixel.

  The plug is 54 art pixels tall, its tip the bottom 12 (18 wide, in the middle), which goes into
  the port; the adapter is 80 by 48, its connector the bottom 8. The port's opening is 22 wide at
  the top of the strip, in its middle. crates/cartridge-ui/src/link_art.rs draws the same sprites, held
  to these pixel for pixel.

  Needs everything in foundation/ on the page (see README.md).
*/
(() => {
  // The plug's own colours, lit from the top left: o its outline, D the dark of its slot, S
  // shadow, B its body, G lit, W the brightest edge.
  const PURPLE = { o: '#1d1430', D: '#2e2352', S: '#46397a', B: '#6a55a6', G: '#8f7bd0', W: '#b7a8ea' };
  const GREY = { o: '#1f2024', D: '#3a3b40', S: '#6f7078', B: '#9c9ca4', G: '#c2c2c9', W: '#e2e2e7' };
  // The adapter's smoky grey plastic, the same keys.
  const SMOKE = { o: '#15171a', D: '#272a2f', S: '#3a3f46', B: '#4f5660', G: '#6e7681', W: '#9aa2ac' };

  // The host's plug, the cable's narrow end: the neck the cable goes into with its two rings, the
  // housing with three grip ridges, and the tip with its slot, which goes into the port.
  const PLUG_HOST = [
    '..........oGBBBBBBSo..........',
    '..........oGBBBBBBSo..........',
    '..........oSSSSSSSSo..........',
    '..........oGBBBBBBSo..........',
    '..........oSSSSSSSSo..........',
    '.........oGBBBBBBBBSo.........',
    '...oooooooooooooooooooooooo...',
    '.ooGGGGGGGGGGGGGGGGGGGGGGGSoo.',
    '.oWGBBBBBBBBBBBBBBBBBBBBBBSSo.',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBSSSSSSSSSSSSSSSSSSBBBSSo',
    'oGWGBBGGGGGGGGGGGGGGGGGGBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBSSSSSSSSSSSSSSSSSSBBBSSo',
    'oGWGBBGGGGGGGGGGGGGGGGGGBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBSSSSSSSSSSSSSSSSSSBBBSSo',
    'oGWGBBGGGGGGGGGGGGGGGGGGBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBSSSSSSSSSSBBBBBBBSSo',
    'oGWGBBBBBSBBBBBBBBBBGBBBBBBSSo',
    'oGWGBBBBBSBBBBBBBBBBGBBBBBBSSo',
    'oGWGBBBBBSBBBBBBBBBBGBBBBBBSSo',
    'oGWGBBBBBBGGGGGGGGGGBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oBSSSSSSSSSSSSSSSSSSSSSSSSSSSo',
    '.oSSSSSSSSSSSSSSSSSSSSSSSSSSo.',
    '.ooSSSSSSSSSSSSSSSSSSSSSSSSoo.',
    '...oooooooooooooooooooooooo...',
    '......oDDDDDDDDDDDDDDDDo......',
    '......oGBBBBBBBBBBBBBBSo......',
    '......oGBBBBBBBBBBBBBBSo......',
    '......oGBSSSSSSSSSSSSBSo......',
    '......oGBDDDDDDDDDDDDBSo......',
    '......oGBDDDDDDDDDDDDBSo......',
    '......oGBDDDDDDDDDDDDBSo......',
    '......oGBGGGGGGGGGGGGBSo......',
    '......oGBBBBBBBBBBBBBBSo......',
    '......oGBBBBBBBBBBBBBBSo......',
    '......oSSSSSSSSSSSSSSSSo......',
    '.......oooooooooooooooo.......',
  ];

  // The joiner's plug, the cable's wide end, with the socket a third player's cable goes into.
  const PLUG_JOIN = [
    '.............oGBBBBBBSo.............',
    '.............oGBBBBBBSo.............',
    '.............oSSSSSSSSo.............',
    '.............oGBBBBBBSo.............',
    '.............oSSSSSSSSo.............',
    '............oGBBBBBBBBSo............',
    '...oooooooooooooooooooooooooooooo...',
    '.ooGGGGGGGGGGGGGGGGGGGGGGGGGGGGGSoo.',
    '.oWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBSSSSSSSSSSSSSSSSSSSSSSSSBBBSSo',
    'oGWGBBGGGGGGGGGGGGGGGGGGGGGGGGBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBSSSSSSSSSSSSSSSSSSSSSSSSBBBSSo',
    'oGWGBBGGGGGGGGGGGGGGGGGGGGGGGGBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBSSSSSSSSSSSSSSSSSSSSSSSSBBBSSo',
    'oGWGBBGGGGGGGGGGGGGGGGGGGGGGGGBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBSSSSSSSSSSSSSSBBBBBBBBSSo',
    'oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo',
    'oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo',
    'oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo',
    'oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo',
    'oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo',
    'oGWGBBBBBBBSDDDDDDDDDDDDGBBBBBBBBSSo',
    'oGWGBBBBBBBGGGGGGGGGGGGGGBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oBSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo',
    '.oSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo.',
    '.ooSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSoo.',
    '...oooooooooooooooooooooooooooooo...',
    '.........oDDDDDDDDDDDDDDDDo.........',
    '.........oGBBBBBBBBBBBBBBSo.........',
    '.........oGBBBBBBBBBBBBBBSo.........',
    '.........oGBSSSSSSSSSSSSBSo.........',
    '.........oGBDDDDDDDDDDDDBSo.........',
    '.........oGBDDDDDDDDDDDDBSo.........',
    '.........oGBDDDDDDDDDDDDBSo.........',
    '.........oGBGGGGGGGGGGGGBSo.........',
    '.........oGBBBBBBBBBBBBBBSo.........',
    '.........oGBBBBBBBBBBBBBBSo.........',
    '.........oSSSSSSSSSSSSSSSSo.........',
    '..........oooooooooooooooo..........',
  ];

  // A row of the cable, in the grey plug's colours, repeated as long as the cable is.
  const CABLE = ['oGBBSo'];

  // The wireless adapter: a smoky dome over the antenna inside, a label plate, two screws, and
  // under it the connector that goes into the port.
  const ADAPTER = [
    '....................oooooooooooooooooooooooooooooooooooooooo....................',
    '................ooooGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGSSoooo................',
    '.............oooGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSooo.............',
    '...........ooGBBBBBDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBBBBSSoo...........',
    '..........oGBBBDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBBSSo..........',
    '.........oGBBDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDBSSo.........',
    '........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........',
    '........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........',
    '........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........',
    '........oGWGDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDBSSo........',
    '.......oGWGDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDBSSo.......',
    '.......oGWGDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDBSSo.......',
    '.......oGWGDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDBSSo.......',
    '.......oGWGSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSBSSo.......',
    '......oGWGDDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDDBSSo......',
    '......oGWGDDDDDDDDDDDDDDSDDDDDDDSDDDDDDSSDDDDDDSDDDDDDDSDDDDDDDDDDDDDDBSSo......',
    '......oGWGDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBSSo......',
    '......oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo......',
    '.....oSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo.....',
    '.....oGWGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGSSo.....',
    '.....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.....',
    '.....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.....',
    '....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo....',
    '....oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo....',
    '....oGWGBBBBBBBBBBBBBBBDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDBBBBBBBBBBBBBBBBBSSo....',
    '....oGWGBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBSSo....',
    '...oGWGBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBSSo...',
    '...oGWGBBBoooBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBoooBBBBSSo...',
    '...oGWGBBoGSGoBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBoGSGoBBBSSo...',
    '...oGWGBBoSSSoBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBoSSSoBBBSSo...',
    '..oGWGBBBoGSGoBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBoGSGoBBBBSSo..',
    '..oGWGBBBBoooBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBoooBBBBBSSo..',
    '..oGWGBBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBBSSo..',
    '..oGWGBBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBBSSo..',
    '.oGWGBBBBBBBBBBBBBBBBBBDSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSGBBBBBBBBBBBBBBBBBBBSSo.',
    '.oGWGBBBBBBBBBBBBBBBBBBBGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGBBBBBBBBBBBBBBBBBBBSSo.',
    '.oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.',
    '.oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo.',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGWGBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBSSo',
    'oGSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSo',
    'oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo',
    '................................oWGGGGGGGGGGGGSo................................',
    '................................oWGGGGGGGGGGGGSo................................',
    '................................oGooooooooooooSo................................',
    '................................oGoDDDDDDDDDDoSo................................',
    '................................oGooooooooooooSo................................',
    '................................oGGGGGGGGGGGGGSo................................',
    '................................oSSSSSSSSSSSSSSo................................',
    '.................................oooooooooooooo.................................',
  ];
  // Where CART's line goes on the adapter, its capitals two rows down it: in the label plate.
  const ADAPTER_LABEL = [25, 24];

  // The port's opening in the middle of the strip's top edge, its five pins in the frame's bright
  // colour, drawn over the strip.
  const NOTCH = [
    'kkkkkkkkkkkkkkkkkkkkkk',
    '.kkkkkkkkkkkkkkkkkkkk.',
    '..kkkkkkkkkkkkkkkkkk..',
    '..kkHHkHHkHHkHHkHHkk..',
    '..kkHHkHHkHHkHHkHHkk..',
    '..kkkkkkkkkkkkkkkkkk..',
    '..kkkkkkkkkkkkkkkkkk..',
    '...kkkkkkkkkkkkkkkk...',
  ];
  // The strip's top edge, lit, over a stripe; under them the strip is the well colour.
  const EDGE = ['A', 's'];

  // The radio arcs on the adapter's right, two pixels thick, the smallest first.
  const ARCS = [
    [
      'cc...',
      '.cc..',
      '..cc.',
      '..cc.',
      '...cc',
      '...cc',
      '...cc',
      '...cc',
      '...cc',
      '...cc',
      '..cc.',
      '..cc.',
      '.cc..',
      'cc...',
    ],
    [
      'cc.....',
      '.cc....',
      '..cc...',
      '...cc..',
      '...cc..',
      '....cc.',
      '....cc.',
      '.....cc',
      '.....cc',
      '.....cc',
      '.....cc',
      '.....cc',
      '.....cc',
      '.....cc',
      '.....cc',
      '....cc.',
      '....cc.',
      '...cc..',
      '...cc..',
      '..cc...',
      '.cc....',
      'cc.....',
    ],
    [
      'cc.......',
      '.cc......',
      '..cc.....',
      '...cc....',
      '...cc....',
      '....cc...',
      '....cc...',
      '.....cc..',
      '.....cc..',
      '......cc.',
      '......cc.',
      '.......cc',
      '.......cc',
      '.......cc',
      '.......cc',
      '.......cc',
      '.......cc',
      '.......cc',
      '.......cc',
      '......cc.',
      '......cc.',
      '.....cc..',
      '.....cc..',
      '....cc...',
      '....cc...',
      '...cc....',
      '...cc....',
      '..cc.....',
      '.cc......',
      'cc.......',
    ],
  ];

  // The clicks of a plug going home, either side of its housing over the port: the left half,
  // the right the same turned round.
  const CLICKS_LEFT = [
    '...cc.....................',
    '....cc....................',
    '.....cc...................',
    '......cc..................',
    '..........................',
    '..........................',
    '..........................',
    'cccccc....................',
    '..........................',
    '..........................',
  ];

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      :host { display: inline-block; vertical-align: top; line-height: 0; }
      :host([hidden]) { display: none; }
      .art { background: no-repeat 0 0 / 100% 100%; image-rendering: pixelated; }
    </style>
    <div class="art" part="art"></div>`;

  // The palette's keys, as pixel.js has them.
  const KEYS = {
    k: '--oz-dark', e: '--oz-edge', w: '--oz-well', p: '--oz-plate', t: '--oz-tab', b: '--oz-band',
    s: '--oz-stripe', d: '--oz-dim', A: '--oz-frame', H: '--oz-frame-hi', L: '--oz-label', c: '--oz-cream',
  };

  // `rows` as a CSS url() of a PNG, a pixel to a character: a key `own` has in its own colour,
  // the palette's keys in the colours `from` sees, '.' left clear.
  function paintRows(rows, from, own = {}) {
    const style = getComputedStyle(from);
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, ...rows.map(row => row.length));
    canvas.height = Math.max(1, rows.length);
    const ctx = canvas.getContext('2d');
    rows.forEach((row, y) => [...row].forEach((key, x) => {
      const colour = own[key] ?? (KEYS[key] && style.getPropertyValue(KEYS[key]).trim());
      if (!colour) return;
      ctx.fillStyle = colour;
      ctx.fillRect(x, y, 1, 1);
    }));
    return `url("${canvas.toDataURL()}")`;
  }

  // `rows` with `over` laid on them, its top left at (x, y); '.' in `over` leaves what is under.
  function laid(rows, over, x, y) {
    return rows.map((row, ry) => {
      const top = over[ry - y];
      if (top === undefined) return row;
      return [...row].map((k, rx) => (top[rx - x] !== undefined && top[rx - x] !== '.' ? top[rx - x] : k)).join('');
    });
  }

  // The strip `w` by `h`: its lit edge and stripe, the well colour under them, and the port's
  // opening in the middle, rounded down to a whole pixel.
  function port(w, h) {
    const rows = Array.from({ length: h }, (_, y) => (EDGE[y] ?? 'w').repeat(w));
    return laid(rows, NOTCH, Math.floor((w - NOTCH[0].length) / 2), 0);
  }

  // The adapter with CART on its label in the lightest of its colours.
  function adapter() {
    const ink = CartridgePixel.letters('CART').map(row => row.replace(/#/g, 'W'));
    return laid(ADAPTER, ink, ...ADAPTER_LABEL);
  }

  const int = (el, name, fallback) => {
    const v = Math.round(+(el.getAttribute(name) ?? fallback));
    return Number.isFinite(v) && v > 0 ? v : fallback;
  };

  customElements.define('cartridge-link', class extends CartridgePixel.Element {
    static observedAttributes = ['piece', 'role', 'length', 'width', 'height', 'ring', 'side'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    // The piece's rows and its own colours.
    sprite() {
      const join = this.getAttribute('role') === 'join';
      switch (this.getAttribute('piece')) {
        case 'cable':
          return [Array(int(this, 'length', 20)).fill(CABLE[0]), GREY];
        case 'port':
          return [port(int(this, 'width', 160), int(this, 'height', 36)), {}];
        case 'adapter':
          return [adapter(), SMOKE];
        case 'arc': {
          const rows = ARCS[Math.min(2, int(this, 'ring', 0) || 0)] ?? ARCS[0];
          return [this.getAttribute('side') === 'left' ? CartridgePixel.flip(rows) : rows, {}];
        }
        case 'clicks': {
          const right = CartridgePixel.flip(CLICKS_LEFT);
          return [CLICKS_LEFT.map((row, y) => row + right[y]), {}];
        }
        default:
          return [join ? PLUG_JOIN : PLUG_HOST, join ? GREY : PURPLE];
      }
    }

    paint() {
      if (this.getAttribute('piece') === 'adapter' && !CartridgePixel.fontReady(() => this.paint())) return;
      const [rows, own] = this.sprite();
      const art = this.shadowRoot.querySelector('.art');
      art.style.width = `calc(${rows[0].length} * var(--u))`;
      art.style.height = `calc(${rows.length} * var(--u))`;
      art.style.backgroundImage = paintRows(rows, this, own);
    }

    attributeChangedCallback() {
      if (this.isConnected) this.paint();
    }
  });
})();
