# Changelog

## Unreleased

- The shelf is plainer: the two corner actions (SELECT CONTROLS, START SETTINGS) are gone from the
  bottom bar, leaving the system's pill; the top bar has no ground or rule, only its icons and
  figures on the background; and the cart the shelf rests on stands 12 % bigger than the ones
  beside it, keeping that size as it goes in.
- Settings is one list, SETTINGS, with no CONFIGS and GENERAL panels: Equalizer, 5 Games, Rumble,
  Labels, Achievements, Date & Time, Network, Bluetooth, Theme, Sleep, About.
- About names @NUKEBOMB3000 as the author of Cartridge.
- The game menu on a short MENU press has three tabs that L1 and R1 turn between: GUIDE (as
  before, now opening for a game with no guide too, where it says where to put one),
  ACHIEVEMENTS (the game's list, unlocked first, with points) and SETTINGS (brightness, blue
  light, volume, screen look, picture size, the Game Boy palette, ghosting, colour correction,
  rumble, then the running core's own options; Up and Down choose, Right steps a value on and
  Left back, each kept at once). Left and Right still page the guide and the achievements
  (`screenshots/game-menu-achievements.png`, `screenshots/game-menu-settings.png`).
- RetroAchievements, softcore only, ported from nickclyde's slot-plus through fragilesilver's
  slot-fragile (rcheevos 12.5): Settings > Achievements takes the player's own account on the
  keyboard screen (Y logs out) and switches the service on or off. A signed-in game is followed
  by its core's memory and unlocks, the game's progress and completion pop up across the top in
  the theme's colours, with their badges; they queue while a game is still booting and are sent
  once back online. The password is kept only until the first sign-in has handed back a token.
  Hardcore is not offered, so saves, states, rewind and fast forward all work.
- The handheld sleeps by itself after ten minutes without a press and powers itself off three
  minutes into any sleep, the lid shut, POWER tapped or idle alike. Settings > Sleep sets both:
  SLEEP AFTER (never, 1, 2, 5, 10, 15, 30 minutes) and POWER OFF AFTER (never, 1, 3, 5, 10, 30,
  60). A sleep the handheld fell into by itself wakes on any button; a shut lid only when it
  opens. No sleep while a link game runs or labels are fetched (`screenshots/sleep-settings.png`).
- Fast Forward and Fast Forward Sound are gone from Settings to make room: it is always 6X, with
  no sound.
- The shelf opens on the cart it was left on when no game is to resume. The cart, its system and its
  place on the shelf are kept in the state file two seconds after the shelf comes to rest, and when
  the handheld goes to sleep or off. If that game is gone from the card, the shelf opens at the same
  place on that system's shelf, or on its last cart if the shelf got shorter, or as it always did if
  the system has no games left.
- The boot logo's cart has a label after Apotris's: CARTRIDGE in bevelled tetromino blocks on a dark
  playfield, over a skyline of settled pieces (`screenshots/boot-logo.png`).
- The autosave writes a game's save every minute but its resume state only every five, each only
  when it changed: a state runs to megabytes, and writing one every minute wore the card for
  little. Ejecting, sleeping, closing the lid and powering off still write both at once.
- Y on the shelf colours the cart: twelve pigments after slot-fragile's (Ruby Red, Sapphire Blue,
  Emerald Green, Gold, FireRed, LeafGreen, Jet Black, Atomic Purple and others) in solid, clear
  or glittered plastic, the cart taking each change at once; kept in `Config/cart_shell.ini`,
  ORIGINAL taking its own colour back.
- Game Boy and Game Boy Color games run on Gambatte, built here from its source: on the RG SP it
  takes 40 to 56 % less of the CPU than mGBA for the same game, so the battery lasts. A Game Boy
  only game runs on the original Game Boy, where the palettes (SELECT+L2/R2) colour it; without
  one it is coloured as a Game Boy Color would. mGBA stays a press of X away on the shelf, on a
  Game Boy board of its own (DMG-BEAN-02's look), and link play asks for it.
- Nintendo DS games, from `Games/NDS`, on DSperate, built here from its source with frameskip
  (Auto while the sound runs short): both screens in one picture, the top one large. R2
  fast-forwards as on the other systems and L2 swaps the screens; there is no stylus. SELECT+MENU in a DS game opens a pause menu after DSperate's own: save
  and load a state, the save states, OPTIONS (screen layout, the gap between the screens and
  frameskip, each change taken at once and kept in `Config/core_options.ini`), resume and quit.
- The core picker opens a PlayStation disc on the console's own mainboard, PU-18's look: the
  game's title on the CPU, the RAM and the BIOS above the sockets, the CD drive's connector
  below. A game named as one on another shelf, 007 on the Game Boy Color and on the
  PlayStation, no longer opens on the other one's board.
- PlayStation games can run on SwanStation (the libretro DuckStation) as well as PCSX-ReARMed:
  X on the shelf picks it, given `System/swanstation_libretro.so` and a real BIOS in BIOS/.
  It draws with the GPU: cores that ask for OpenGL ES get a GLES 3 context of their own on the
  emulator's thread and a framebuffer whose frames are read back, so rewind, states and the
  looks take them as any; while one runs the GPU is held at its top clock (648 MHz), which
  took SwanStation from 47 to a full 60 frames a second on the RG SP. PCSX stays the default.
- A game holds the CPU to what its system needs, so the handheld runs cooler and longer: 720 MHz
  for the Game Boy, Game Boy Color, Game Gear, Master System, NES and PC Engine, 1008 MHz for
  the Mega Drive, 1200 MHz for the GBA, the whole 1512 MHz for the SNES and the PlayStation.
  Fast forward, rewind and the menus get it all. Measured on the RG SP: a Game Boy core needs
  under half a core even at 480 MHz.
- The CRT looks follow the game's own lines: a scanline to each, a PlayStation picture of 480
  lines shown as the fields a TV drew it in, the beam soft across and widening where bright,
  and the aperture grille only as strong as the screen has pixels for. PlayStation games no
  longer come out under a coarse, coloured mesh.
- SELECT+B gives the TV systems ghosting too: each frame shown blended with the last, as a
  TV's phosphor lingered, so flicker drawn as transparency shows. Off until switched on, kept
  apart from the handhelds' LCD ghosting.
- Theme: nine more colours (Black, Indigo, Sky, Mint, Olive, Rose, Coral, Brown, Wine), all in
  order round the colour wheel after the neutrals, and five
  backgrounds after real handhelds: Game Boy (its dot matrix), DS (the DS Lite menu's lines),
  3DS (stripes), PSP (the XMB's waves, drifting round seamlessly) and Switch (its grid of rounded
  tiles).
- A 5-band equalizer (150 Hz, 400 Hz, 1 kHz, 2.5 kHz, 6 kHz, ±12 dB) in Settings, in Colour
  Correction's place (SELECT+A still switches that): one setting for the speaker and one for
  headphones, presets Flat, Speaker, Bass and Clear on Y, and a test sound on A, the card's
  `System/eq-test.wav` (16-bit stereo, 48 kHz) or a built-in one.
- The game guide takes its colours from Settings > Theme > Colour: the page in the theme's
  darkest tone, the bands a panel's, headings and tables in its lighter tones.
- The boot logo is a Game Boy Advance cart with Cartridge on its label (`task bootlogos:cart`).
- Brightness has 20 steps instead of 9, smallest at the dim end, and its lowest step keeps the
  backlight on, as dim as it goes; only the closed lid turns it off. A brightness saved on the
  old scale is carried over to the nearest new step.
- omega-zero is now Cartridge: the program is `System/cartridge`, its log `cartridge.log`, its
  settings `Config/cartridge.state` (taken over from `Config/omega-zero.state` at the first
  start), with a new logo on the boot screen. Copy the new `System/` over the old one; the old
  `System/omega-zero` can be deleted.
- Game guides: a short MENU press in a game opens its walkthrough over the paused game, from
  `GUIDES/<system>/<game>.txt` (or `.html`/`.htm`), matched by name with brackets and case left
  out; several guides go in a folder named as the game, X steps through them. L1/R1 or
  Left/Right turn a page, Up/Down scroll and keep scrolling while held, L2/R2 make the text
  smaller or larger (8 sizes), B or MENU close it. A double MENU press still opens the save
  states. Hard-wrapped TXT prose is joined back into paragraphs (a word split by a hyphen made
  whole), tables and ASCII art keep their columns in a fixed-width face shrunk to fit across,
  decorated and underlined lines become headings; HTML shows its headings, paragraphs, lists,
  tables and `<pre>` blocks and drops scripts and styles. The place in each guide and the size
  are kept in `Config/guides.txt` and survive restarts and size changes. A game without a guide
  says so with a toast. Text is set in DejaVu Sans and DejaVu Sans Mono.
- Rumble works again: the ANBERNIC-keys driver drops the motor's effect when another handle on
  its node closes, so it is armed again at the start of each buzz. Each buzz runs at least
  100 ms, so a game's taps are felt, and at most 150 ms.
- Rumble in Settings is OFF, LOW, MED or HIGH. The motor knows only on and off, so LOW and MED
  switch it on for 35 and 60 % of each 20 ms.
- The launcher rumbles too: a tick as the shelf moves on, a thump as a cart seats, as it comes
  out and as the handheld powers off.
- PlayStation games get a DualShock, the pad that rumbles; the d-pad also pushes its left stick,
  so a game in analog mode plays on a handheld without one.
- RG CubeXX: both analog sticks steer as the d-pad, on the shelf, in the menus and in games,
  pressing past half their reach and letting go under a quarter so a stick held near the edge
  does not chatter. Real analog values for the cores are still to come.
- RG CubeXX: on its square screen the shelf and the open cart stand in the middle between the
  bars instead of down by the footer. 640x480 and 720x480 are unchanged.
- Overlays for 720x720 can be drawn per picture size as for the other panels:
  `Overlays/GB/720x720-actual.png` and so on. At Aspect a Game Boy game fills the square's
  width, so it has no room for one.
- Five Games: 5 GAMES in Settings picks up to five games right on the shelf, across every
  system (A marks a cart, START keeps them, B gives up), and from then on the handheld shows
  only those, on one shelf of their own, across restarts. The pill counts the games picked; a
  sixth, or START with none, is refused with a toast saying so. A on 5 GAMES again changes
  them, the games kept coming up marked; left or right on it switches Five Games off.
  It is for playing a few games rather than scrolling through hundreds. It takes LCD
  Ghosting's row, which SELECT+B still switches in a game.
- Holding POWER in a game saves it where it is and powers off at once, with no menu; on the
  shelf and in the menus it opens the power menu as before. In a link session it is refused.
- The shelf names the selected game over the bar across the bottom, its file's name without
  its tags in brackets, "Legend of Zelda, The" said as "The Legend of Zelda".
- CONTROLS names every hotkey: X and Y held for turbo, SELECT+START for the card repair and, on
  the Game Boy, SELECT+L2 R2 for its palette.
- A card the kernel made read-only after an error no longer keeps omega-zero from starting:
  its log goes to /tmp instead, and omega-zero comes up to offer SELECT+START to repair it.
- A curved CRT for the television's consoles, after the CRT on SELECT+X: the picture bowed
  out to rounded dark corners, its scanlines following the bow.
- Game Gear and Master System games, on Genesis Plus GX as the Mega Drive's: `Games/GG` takes
  `.gg` and `Games/SMS` `.sms`, with Labels, Overlays, Saves and States beside them and labels
  from ScreenScraper. Each stands on the shelf as its own black cart, the Game Gear's wide with
  its top corners cut, the Master System's tall with grip ridges. A Game Gear game is laid out as
  a Game Boy game is, 3x at its actual size over the body, with the handheld looks on SELECT+X,
  LCD ghosting on SELECT+B and the picture size on L1 R1; a Master System game as a television's
  console, twice its lines, with scanlines and CRT. 1 and 2 are B and A, X and Y turbo.
- An overlay can be drawn for one picture size: `Overlays/<system>/640x480-actual.png` or
  `-aspect.png` comes before `640x480.png`.
- A Game Boy game at its actual size is its 160x144 window at 3x on the top edge over the body,
  with the console's wordmark below, as a GBA game stands, rather than the whole GBA frame.
- Simpletex is jdgleaver's simpletex_lcd, and Transparent zfast_lcd under an unlit LCD after
  Pixel Transparency; LCD ghosting on mGBA fades each frame into the next.

- Game Boy only games can be drawn in any of mGBA's 48 palettes: SELECT+R2 and SELECT+L2 step
  through them in a game, from Auto, the Game Boy Color's own colouring as before, through
  Grayscale, DMG Green and GB Pocket to the Super Game Boy's. A game opened in a palette takes
  another at once; the step to or from Auto takes at the next start, as the toast says. The
  choice is kept. On the Game Boy's controls page it takes the line of L1 R1, which still
  change the picture size.
- Games start sooner: resting on a cart on the shelf for half a second reads its ROM ahead into
  memory, so inserting it no longer waits on the card. A PlayStation disc is left alone.
- When the card turns read-only, a toast says so every 10 seconds, and SELECT+START repairs it:
  omega-zero hands the card to repair.sh, which unmounts it, runs fsck.fat or fsck.exfat until
  it comes back clean and reboots; the next boot says whether it worked. It waits for 20 %
  battery unless the handheld is charging. SELECT+START is only taken while the card is
  read-only. fsck.exfat is built from exfatprogs, its source shipped beside it.
- If omega-zero stops responding with the lid closed, a watcher apart from it powers the handheld
  off a minute after omega-zero would have, unloading the Wi-Fi driver and syncing first.
- The speaker no longer buzzes while nothing plays: omega-zero lets go of the audio device after
  3 seconds of silence and takes it back when sound starts, retrying a device the BaseOS startup
  sound still holds.
- The card is written less: the resume state only when it changed, the boot partition only when
  the boot logo changes, and temporary files a power cut left half-written are cleared at boot.
- Game sound is resampled to 48 kHz through a windowed sinc instead of linearly to 32768 Hz, so
  high frequencies no longer fold back as grit; mGBA benefits most.
- A Game Boy cart's clock is kept in its .sav after the save RAM, as other emulators write it,
  so Pokemon Gold, Silver and Crystal keep their time.
- Loading a state while a GBA game is saving to a 128 KB flash chip, as Pokemon Ruby, Sapphire
  and Emerald and Super Mario Advance 4 have, no longer writes the save into the wrong half of the
  chip: mGBA keeps the chip's bank through the load (slot's patch).
- Pokemon Pinball: Ruby & Sapphire rumbles: mGBA answers the game's Game Boy Player check.
- The boot goes from the bootloader's logo straight to the shelf: the NOM ENGINEER terminal and
  its fades, some twelve seconds, are no longer played. A start that failed still says so.
- The GBA and Game Boy games blend each frame with the last, as the LCDs they were drawn for
  blurred them, so a game that flickers a sprite to draw it see-through, as F-Zero and the Boktai
  games do, shows it as it was meant to be seen. mGBA mixes the two frames and gpSP its frame
  mixing. It is on by default: LCD Ghosting, under Colour Correction in CONFIGS, turns it off, as
  does SELECT+B in a game.
- The RG SP and the RG34XX SP calibrate their panels as omega-zero starts: the display engine
  takes a measured table of each panel's colours, its black and white points among them, and
  applies it after everything is drawn, to the games and the menus alike, at no cost a frame.
  The tables are inside omega-zero; nothing new goes on the card.
- On the RG SP, the RG34XX SP and the RG34XX, the panel runs at 120 Hz, which takes about half a
  frame off the wait between a frame being drawn and being shown. omega-zero only asks for it
  where BaseOS names one of those handhelds and the device tree names its panel, reads it back
  once set, and goes back to 60 Hz if it does not read back. A game still runs at its own speed:
  at 120 Hz omega-zero keeps its own clock, ticking a game once a frame of its own and showing
  each frame at the next refresh. The panel is set again when the lid opens; one that refused it
  or lost it is written down in `Config/omega-zero.state` as `high_refresh=0` and kept at 60 Hz
  from then on, and setting that by hand keeps any panel at 60 Hz.
- SELECT+Y goes round a Game Boy, Game Boy Color or GBA game's picture sizes, as on the other
  systems, kept per system: as it was always drawn, its own shape as large as the screen holds (a
  GBA game 640x427 on a 640x480 screen, a Game Boy game 533x480), and stretched over the whole
  screen. L1 and R1 on the Game Boys still stretch and unstretch.
- A handheld game has four hotkeys under SELECT: A the colour correction, B the LCD's ghosting,
  X the screen look and Y the picture size.
- Three more handheld looks on SELECT+X, each one pass: Simpletex, pixels printed over grained
  paper; Transparent, light pixels letting a pale LCD background through with dark ones casting
  a shadow on it; and, on the Game Boy only, DMG, its four greens over a faint grid.
- A handheld's look Off draws the picture with nothing over it; the LCD mask no longer stayed
  over it at 3x.
- A game ScreenScraper knows but has no scan of the label for wears its box's front instead, cut
  to its label, rather than its name; a PlayStation disc tries the face ScreenScraper draws of it
  first.
- SELECT+X in a game steps through screen looks for its system, kept per system in
  `Config/shader.ini`: off, LCD, grid and dot for the Game Boys and the GBA, ported from slot's
  ports of the libretro shaders, and off, scanlines and CRT for the television's consoles.
- The picture size SELECT+Y (or L1 and R1 on the Game Boys) picks is kept per system in
  `Config/video_mode.ini`, not per game.
- PlayStation labels whose discs ScreenScraper numbers are fetched again. Their names carry the
  disc's number in brackets, `support-texture(us)[1]`, which curl read as a range and refused, so
  the sync counted them as having no scan.
- Mega Drive and Genesis games play from `Games/MD`, as `.md`, `.gen` or `.bin` images, on
  Genesis Plus GX (free for non-commercial use only), which ships with its source in
  `System/licenses/`. A, B and C are on Y, B and A, as on most handhelds, and the core picks a
  three or six button pad for each game; a six button game finds X, Y and Z on L1, X and R1.
  The picture is twice its 320 pixel lines, 640x448, or 4:3 or stretched on SELECT+Y. Titles
  and regions come from the cart's header, labels from ScreenScraper.
- A Mega Drive game stands on the shelf as the black Mega Drive cart of Japan and Europe, 112 by
  130 mm with its top corners rolled round, and an American one as the squarer Genesis cart, 110
  by 135 mm with grip ridges across its top; its label from ScreenScraper is taken from the
  region of the cart it stands as. `cart_shell.ini` takes `megadrive` and `genesis` for either
  shape.
- NES games play from `Games/NES`, as `.nes` iNES images, on FCEUmm (GPL-2.0), which ships with
  its source in `System/licenses/`. The picture shows the 224 lines a television did and is laid
  out as the SNES's; held X and Y are turbo A and B, as on the handhelds. Labels come from
  ScreenScraper, an image asked about with its iNES header and without.
- An NES game stands on the shelf as the grey Game Pak, 120 by 133 mm, and a Japanese one as a
  Famicom cart, 108 by 71 mm, in a colour of its game's own as the real ones came; its label
  from ScreenScraper is taken from the region of the cart it stands as. `cart_shell.ini` takes
  `nes` and `famicom` for either shape.
- PC Engine games play from `Games/PCE`, HuCards as `.pce` files, on Beetle PCE Fast (Mednafen's
  pce_fast, GPL-2.0), which ships with its source in `System/licenses/`. The picture is laid out
  as the SNES's, twice its size, 4:3 or stretched with SELECT+Y, and labels come from
  ScreenScraper.
- A PC Engine game stands on the shelf as a HuCard, 54 by 85 mm, its label across most of its
  face: pale grey for a Japanese game, a black TurboChip for an American one, its label from
  ScreenScraper taken from the same region as the card.
- SELECT+Y goes round a television's console's picture sizes, and its system keeps the one
  it was left in: integer, its actual size at the largest whole scale; keep aspect, a television's
  4:3 rather than the SNES pixels' 8:7, filling a 640x480 screen; and stretch, over the whole
  screen. A PlayStation game starts in 4:3, as it always has, its picture changing size between
  menus and play. The handhelds' SELECT+Y is still the colour correction.

## omega-zero 1.2.0

Supafaust runs SNES games, a game that takes the console down can no longer trap it at boot, and
labels take a quarter of the room and no longer hold the boot.

- SNES games run on Supafaust. Chosen with X, Supafaust took omega-zero down on a game's first
  frame and left the console on its boot screen: it forgets the defaults of its own options once
  a game has loaded, and read an empty one back. omega-zero now answers every option a core
  declares, with the core's own default where omega-zero sets none, as any libretro frontend does.
- A game whose core takes omega-zero down as it starts is no longer opened again at every boot.
  The next boot leaves it on the shelf, selected, rather than resume it, so the console can no
  longer get stuck on its boot screen over one game. That boot says so after the opening, on its
  terminal: which game went down, on which system and core, and how to try it another way. A, B
  or START goes on to the shelf once its prompt is up.
- When omega-zero goes down, the card keeps what it last said in `crash/omega-zero.log` and its
  whole memory as it went down in `crash/omega-zero.core.gz`, for a debugger. It starts again once
  the core is written, some seconds later. A crash within ten minutes of the last keeps the first
  core, and none is written when it would leave the card less than 512 MB.
- Labels take about a quarter of the room on the card. Once a label is cut to its cart, the
  label itself is shrunk to the smallest size that still covers every face its system draws it
  on, since no face is drawn bigger than that; until now each one was kept 600 pixels wide. Every
  cart looks as it did. Labels already on the card, or copied there by hand, are shrunk once in
  the background after the boot.
- Labels fetches each label at about two thirds of the width it did, so a sync downloads about
  half as much.
- The boot no longer reads any label before the shelf comes up. Every cart starts in its
  generated label and is drawn in its own on threads that stand back from the shelf and the
  game, the carts on screen first, a few a frame, while the opening plays.

## omega-zero 1.1.0

The SNES and the PlayStation join the Game Boys: their games run, and they stand on shelves of
their own, SNES carts and PlayStation discs modelled like the Game Boy carts.

- SNES games go in `Games/SNES` (`.sfc`, `.smc`). A game from the USA stands as the American SNES
  cart; one from Japan, Europe or anywhere else stands as the Super Famicom cart, as it was sold.
  The region comes from the ROM's header, a tag in the file name such as `(USA)` or `(Japan)`
  wins over it, and `cart_shell.ini` can set a cart's shape to `ntsc` or `sfc`.
- Two SNES cores, Snes9x and Supafaust, chosen per cart with X, as for the GBA's two. Snes9x is
  the default.
- PlayStation games go in `Games/PS1` as `.cue` with their `.bin`, `.chd`, `.pbp` or `.m3u`, also
  one folder down. They run on PCSX ReARMed, which carries its own BIOS; a real one in `BIOS/` is
  used when it is there.
- A PlayStation game stands on its shelf as its disc. A game on several discs is one entry, its
  discs stacked one behind another with a gap between them, and the whole stack goes down into
  the handheld. Files named `(Disc 1)`, `(Disc 2)` and so on are put together as one game, with
  a playlist made for them under `Config/playlists/PS1/`. In a game, SELECT+MENU opens DISC to
  put another disc in, and the disc in the drive is the one there when the game is resumed.
- In a SNES or PlayStation game, X, Y, L1 and R1 go to the game. A PlayStation game takes L2 and
  R2 too: fast-forward is SELECT+R2 held, and it has no rewind. The SNES keeps L2 rewind and R2
  fast-forward.
- The SNES picture is drawn at twice its size; the PlayStation's fills the screen at 4:3. A
  game made for 50 Hz runs at 50 Hz.
- Labels fetches SNES labels, and the label of each PlayStation disc.
- Snes9x is free for non-commercial use only; Supafaust and PCSX ReARMed are GPL-2.0. Their
  licences and source are in `System/licenses/`.

## omega-zero 1.0.1

The boot no longer waits on the labels: each one is cut to its cart once, as it is fetched.

- A card with hundreds of labels boots as quickly as one with none. Each label is cut to its
  cart's face once, right after Labels fetches it, and kept in `Labels/<system>/.cut/`; a boot
  draws the carts from those small cuts, on every core. Until now each boot decoded every
  full-size label before its first frame, and with a few hundred of them the boot logo stayed up
  long enough to look frozen. Labels already on the card, or copied there by hand, are cut once
  in the background after the boot, the carts on screen first, and wear a generated label until
  then.

## omega-zero 1.0.0

Labels fetch in the build you download: omega-zero now has its own ScreenScraper key.

- Labels, in Settings, fetches the scanned label of every cart that has none in the release
  build too. Until now a release carried no ScreenScraper developer key, so the screen said it
  could not reach ScreenScraper. You still sign in with your own ScreenScraper account.

## omega-zero 0.0.4

The slot under the shelf is gone: carts go down into the handheld, and the game comes on like a
tube TV.

- The slot under the shelf is gone: a cart goes into the handheld itself. From the middle of the
  shelf it rises a little, then is pulled down out of the bottom of the screen, and the game
  comes on as a tube TV does: a dot, a line across, then the picture opening white and fading to
  its colours. Leaving a game plays it backwards, and a cart the core turns away goes back out
  the same way under its warning sign.
- The carts stand in the middle between the top bar and the bottom bar, at their usual sizes. A
  Game Boy cart no longer grows on its way in.
- The shelf's system, GBA, GB or GBC, is in a pill between the bottom bar's buttons, always. It
  rises in as L1 or R1 changes shelf, and a letter jumped to with Up or Down shows in it for two
  seconds.
- Config/theme.txt's `recess` and `opening` coloured only the slot and are no longer read.

## omega-zero 0.0.3

Settings in the style of Mega Man Zero's menus, with Network, Bluetooth, Theme and a POWER menu;
every screen drawn in the UI kit's 8/16-bit pixel art; carts as low-poly models drawn the way a
PlayStation drew 3D; and an opening after Mega Man X5's boot terminal at power-on.

- Everything around the shelf is on the menus' 2-pixel grid: the slot's edges and its scoop, the
  slot's name in the menus' type and the theme's dim colour, the top bar's clock, figure and
  "ssh" (drawn bold the kit's way), and a generated label's title, now always Pixel Code at its
  own size. A cart the core turns away comes out under a pixel-art warning sign in the theme's
  colour. The background drifts two pixels at a time, a refused cart shakes a whole pixel at a
  time, and the boot screen fades out in four steps.
- The save states, MENU ×2 in a game, are drawn in the menus' pixel art. The shelf's top bar
  stands over the screenshot in place of a dark strip of their own; a SAVE STATES panel at the
  top in the middle says how long ago the save shown was made, with a dot for each save under
  it, the one shown lit; and the bar along the bottom has B RETURN and A LOAD. Nothing over the
  screenshot is see-through any more.
- Save states only save and load: Y no longer deletes the save shown, and X no longer undoes the
  last save or load.
- `<omega-hints>` joins the UI kit: a screen's keys on one line of the kit, on a ground of their
  own, for where the bar's hints would sit on a picture.
- The link screen (SELECT + MENU in a game) is drawn in the menus' kit: the theme's backdrop, a
  LINK panel with ROLE (HOST or JOIN, Left and Right swap it) and MODE (CABLE or WIRELESS,
  SELECT switches it where the game can), what is happening as a note under them, and the
  bottom bar with RETURN and LINK, SELECT MODE and ◀▶ SWAP between them. The plug, its cable,
  the port, the wireless adapter, its radio arcs and the clicks of a plug going home are pixel
  art on the 2-pixel grid, the plug still the purple of the real cable. A link that fails lifts
  the plug out and shakes it a pixel either way instead of tilting it, and the arcs pulse in four
  steps.
- The carts are low-poly models drawn the way a PlayStation drew 3D: a face and a side for
  every cut of the outline, each lit on its own, corners that land on the 2-pixel grid as they
  turn, textures mapped without perspective, and colour in 15 bits through the PlayStation's
  dither. The one you are on sways twenty degrees either way, seen from closer than before, so
  its side and the step under a GBA cart's grip show. The cart going into the slot and the core
  picker's lid are the same model. A cart is one draw instead of up to seventeen.
- The volume, brightness, blue light and rewind show as a gauge drawn like the menus: the
  level's name in a tab, then a cell for each step, lit as far as the level reaches, and its
  icon, on the 2-pixel grid in the colours of Settings > Theme, with no dark band behind it. It
  stands at the top of the screen, on the shelf just under the top bar, and fades out in four
  steps. What just happened shows the same way, a small panel titled SAVE STATE, LINK or COLOUR
  over "State saved", "Link ended", "Correction on" and the rest.
- Fast forward's badge in the corner is a key cap holding ▶▶ and the speed, ×2 to ×6, lit
  while it is latched. A link's badge shows a link, lit while hosting and dark when joined, and
  broken once the other handheld is lost.
- The core picker is pixel art on the 2-pixel grid: the open cart in its own plastic, the board,
  the ROM chip marked with the game's title, the sockets and the chip, with outlines and flat
  colours. The title and the cores' names are in Pixel Code at its own size and spacing. The
  chip hops between the sockets a pixel at a time, level, and the shadows under it and under
  the lid are checker dither.
- About is a window like Settings, drawn in the menus' kit in place of the sticker. OMEGA-ZERO
  lists the version, the build (its commit, with -DIRTY after it for a build with changes not
  committed), the author, @oauramos, the handheld as BaseOS names it, and how full its
  battery is and whether it is charging. CREDITS under it says what omega-zero is made with and
  on, a line each. The barcode, the ANBERNIC wordmark and the made-up rating lines are gone.
- A bar along the bottom of the shelf: SELECT CONTROLS on the left, START SETTINGS on the
  right. The slot sits a bar higher to make room for it.
- Game Boy carts stand at 80% of their size on the shelf and grow to full size as they go into
  the slot.
- The handheld starts on an old PC's BIOS screen, after Award's, in place of the logo: the
  OmegaZero BIOS, "An Anbernic Ally", oauramos Software Inc., the H700, a memory test, BaseOS's
  Plug and Play Extension and Press START to enter SETTINGS, with the OmegaZero logo in 16
  colours where Award put its Energy Star. It is set in the opening's letters, where its
  terminal prints. `task bootlogos` renders it; `task bootlogos:logo` renders the old logo.
- An opening after Mega Man X5's NOM ENGINEER WORK SYSTEM boot screen, told about the handheld
  it runs on. omega-zero's first frames show the boot screen where the handheld drew it, and it
  fades to black. Then a terminal fades in, in X5's wide mint letters and its beeps: the model,
  the memory counted up, the H700, the Mali-G31, the battery and its charge, a login as root,
  `>omega -boot -a` typed out and three files read. After ten seconds it fades out and the shelf
  fades in. Any button cuts it short, and that press does nothing on the shelf. Once the
  shelf shows through, a press ends the opening at once and works as usual. POWER and the lid
  end it at once. A boot back into a game, or to the clock, cuts as before.
- START on the shelf opens Settings, a window in the style of Mega Man Zero's menus, in place of
  the old quick menu. CONFIGS holds Fast Forward (2X, 3X, 4X or 6X), Fast Forward Sound, Colour
  Correction, Rumble and Labels; GENERAL holds Date & Time, Network, Theme and About.
- A row that Left and Right change shows ◀ ▶ while it is lit. A row that opens a screen ends
  in ›.
- A tap of SELECT on the shelf opens CONTROLS: what every button does, on the shelf and in a
  game, on one panel. SELECT held with another button works as before.
- The Wi-Fi screen is now Network, under Settings. Its NETWORK panel switches Wi-Fi, Network
  Time and SSH. AVAILABLE NETWORKS shows each network's signal, a padlock on those with a
  passphrase, and CONNECTED, JOINING or SAVED. Y forgets a saved network.
- Network looks for networks only when X is pressed, once each time, and the bar says X SCAN.
  Until then it lists the network joined and the saved ones, with a note to press X. It no
  longer scans every 12 seconds while it is open, which cost battery and, on the RG35XX SP,
  came before reboots that sometimes hung.
- Wi-Fi, Network Time and SSH are each ON or OFF, and Y on any of them opens its panel. Wi-Fi's
  shows the network joined, its signal, the handheld's address, `<name>.local` and the radio's
  MAC address. Network Time's shows whether a time server has set the clock, and the date, time
  and zone. SSH's shows how to log in from a computer: the address, `<name>.local`, user `root`,
  the password and port 22. SFTP uses the same login.
- SSH ON starts the SSH server and OFF stops it, leaving a login already made to carry on; the
  card remembers which.
- The first time omega-zero runs on a card whose `baseos.conf` sets no `ssh_password`, it gives
  root a password of six random digits, at once and in `baseos.conf`, so BaseOS keeps it. A
  password you set there yourself is kept.
- Cart Labels is no longer on the Wi-Fi screen: it is Labels, in Settings.
- Theme, Date & Time and Labels are panels over the background, and the passphrase
  keyboard is a window.
- In the menus the bottom bar says B RETURN and A CONFIRM. A screen's own keys sit in the middle
  of it: Y INFO and Y FORGET on Network, START FETCH and Y SIGN OUT on Labels. Between its two
  ends there is nothing but those keys, over the background.
- Every move between screens slides, over a background that keeps moving. Going into a screen,
  the one you leave goes out to the right and the new one comes in from the left; coming back is
  the mirror. Leaving the shelf, the top bar goes up, the carts sideways and the slot down.
  Settings, from the shelf, slides the other way round: the carts go out to the left as it comes
  in from the right, and it goes back out to the right. Inside Settings every screen slides the
  usual way.
- The menus, both bars and the slot take their colours from Settings > Theme > Colour.
  Night is grey; Amber is Mega Man Zero's own colours.
- Theme is what Background was: the background's pattern, its motion and its colour. It also
  picks the font of the bottom bar's big words, Bold or Thin.
- Restart no longer freezes the console after Settings > Network has been open. The RTL8821CS's
  Wi-Fi driver hung the kernel's shutdown when it had just gone into its power saving, as it does
  after scanning for networks, so the console stayed frozen until it was reset. omega-zero now
  lets go of the driver itself before a restart or power off: wpa_supplicant and the DHCP client
  stop, the interfaces go down, and the device is unbound from its driver.
- The top bar lights Wi-Fi and ssh within a second of a network being joined, and puts them out
  as quickly when it is lost. After boot they stayed dark until a menu was opened.
- Bluetooth, under Network in Settings, on handhelds that have it. Its screen switches it on and
  off and, while it is on, lists the devices paired, then those a look finds. A pairs with a new
  device and connects it, connects a paired one, or disconnects one connected; Y forgets a
  paired one. Settings names the device connected. The card remembers whether Bluetooth is on,
  bringing it up at boot, and the top bar lights its icon while it is. Pairing asks nothing on
  screen, which most gamepads and headphones take. Game sound over Bluetooth headphones and
  Bluetooth gamepads as controllers are still to come.
- The Bluetooth screen looks for devices only when asked: it opens on the paired devices and
  "Press X to look for devices.", and X SCAN looks for 12 seconds, saying "Looking for devices…"
  while the devices it finds join the list, which keeps them until the screen is left. Devices
  are listed by name: one found that gives none, most often a beacon nobody can pair with, is
  left out rather than shown by its address.
- Holding POWER opens POWER, a panel like the other menus over the background, with RESTART and
  POWER OFF and B RETURN and A CONFIRM on the bar; B goes back to where you were, a game
  included. It comes down from the top as the screen under it drops away, and goes back up. After
  a choice, in place of the black screen, a RESTART or POWER OFF panel comes down the same way and
  says "Restarting…" or "Turning off…" until the handheld goes down, and the bar sinks away.

### For developers

- `assets/ui/flow.html` walks every new screen with the slides the device draws, and is the
  reference they are built to. Keys stand in for the buttons, slow motion runs the slides five
  times slower, and the URL's hash picks the screen, colour, panel size and any moment of a
  slide: `flow.html#screen=network&from=settings&t=0.5&colour=amber&shot=1`.
- The storybook has three new components: `<omega-row>`, `<omega-list-row>` and `<omega-bar>`.
- And four for pages, which the site is now drawn with: `<omega-button>`, `<omega-screen>`, `<omega-card>`, `<omega-code>`.
- The kit is in Rust too, in `crates/omega-ui/src/kit/` (palette, pixel, modal, panel, action,
  row, list_row, bar, text, layout). `tests/suite/kit/` holds it to Chrome's screenshots of the
  HTML kit pixel for pixel; `make-refs.sh` takes them again.
- Each menu screen's input and drawing is a file of its own in `crates/omega-zero/src/screens/`,
  its faces are made in `frontend/sync_*.rs`, and `nav.rs` slides between them.
- `card/System/bluetooth.sh` drives the radio for omega-zero (up, down, status, scan on or off,
  pair, connect, disconnect, forget) and prints each device as a tab-separated line;
  `card/test-bluetooth.sh` tests it against fakes of BlueZ's tools.
- The daemons wifi.sh and bluetooth.sh leave running, and a dropbear omega-zero starts again,
  no longer hold the framebuffer open or share omega-zero's session and signals.
- Now that root's password is random, `task deploy:wifi` and `task log:wifi` log in with
  `CONSOLE_PASSWORD` or, without it, the first line of a git-ignored `.console-password` at the
  top of the repo.

## omega-zero 0.0.2

- A cart leaning half off the screen is drawn whole on every GPU: some drivers dropped it for a
  frame. Turning carts go through a shader of their own and everything else through the one it
  always had, so the shelf costs no more to draw than before carts turned.

### For developers

- `task deploy:wifi` builds omega-zero, copies it onto a console on the home network over SSH
  and reboots into it; `task log:wifi` prints its log. `PROFILE=device-dev` builds in seconds.
- Each crate's tests build as one binary, and the suite runs in about 42 s instead of 130 s.

## omega-zero 0.0.1

The first release, for the RG35XX SP and the other H700 handhelds BaseOS runs on.

### Every panel, natively

- omega-zero lays itself out for the panel it finds, from the model BaseOS names: 640x480 on
  the RG35XX SP instead of the RG SP's 720x480 picture shrunk to fit.
- On a 640x480 panel, GBA games fill the width (640x427) through a sharp scaler, over the
  console's body carrying the GAME BOY ADVANCE logo. Game Boy games stay at 3x through the LCD
  mask.
- Your own overlays: a `.png` the size of the screen in `Overlays/GBA`, `Overlays/GB` or
  `Overlays/GBC`, laid over the game. One that would cover the game is ignored. Name one
  `640x480-actual.png`, `640x480-aspect.png` or `640x480-stretch.png` for one picture size
  only; `640x480.png` serves every size without one of its own.
- The boot logo is only ever replaced with one the size of the device's own. Logos ship for
  640x480, 720x480 and 720x720 panels.
- Rumble works on devices whose motor is the PMIC's `moto` pin rather than a force feedback
  node.
- Hosting a link works on devices whose Wi-Fi driver gives no second interface.
- A panel mounted on its side is turned in software (the RG28XX).
- `System/probe.sh` writes a hardware report, also from an empty `RUN_PROBE` file at boot, for
  device reports.

### Wi-Fi, SSH and labels

- Wi-Fi in the quick menu: switch it on, pick a network and type its passphrase on an
  on-screen keyboard. Networks are remembered in `Config/wifi.conf` and joined again at boot.
- Once joined, the Wi-Fi screen shows the address and name to reach the handheld over SSH and
  SFTP.
- Network Time keeps the clock right through BaseOS whenever Wi-Fi is joined.
- A link takes the radio over from home Wi-Fi and gives it back when it ends.
- Handhelds without Wi-Fi don't show the row.
- Cart Labels, on the Wi-Fi screen: sign in with your ScreenScraper account and fetch the
  scanned label of every cart that has none, from the ROM's own region first. Labels go on the
  shelf as they arrive. Yours are never replaced.

### A new look

- Every word on screen is in Pixel Code, a pixel font set on its own grid so it stays sharp.
- A top bar over the shelf: Wi-Fi, Bluetooth and ssh on the left, lit while they are up; the
  clock in the middle; the battery on the right.
- The shelf's background is yours to pick under START > Background: a grid, raised plates
  (riveted or not), dotted lines, a dithered grid, bits or a plain colour, drifting or still, in
  eight colours, every pattern in the same 2x2 pixels as the words over it. A wallpaper on the
  card still works.
- The slot below the shelf is bigger: a cart slides into it at full size instead of shrinking.
- The quick menu scrolls when its rows outgrow the screen.
- On the shelf START opens the quick menu and X the core picker; MENU opens nothing there.
- The carts on the shelf are solid. The one you are on floats and sways ten degrees either way,
  showing its edge, and every cart leans into a scroll. It squares up as it slides into the
  slot or opens for the core picker.
- The boot logo is the OMEGA ZERO title in pixel art, edged in white, over the shelf's grid.
  Its source is `assets/logo/omega-zero-logo.html`; `task bootlogos` renders it for every
  panel, and `STYLE=smooth` renders the drawn-straight version instead.
