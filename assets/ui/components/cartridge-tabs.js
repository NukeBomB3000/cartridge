/*
  <cartridge-tabs>: a page's tabs, drawn as a row of panel titles: each tab the dark title tab of an
  <cartridge-panel> with its slanted end, on the bright strip, the stripe line under the row. The tab
  shown is drawn as an active panel's title; the others are dark, their titles dimmed. The tabs
  are the page's own links, put inside it, each to the part of the page it shows, so with scripts
  off they are a row of links to parts that are all shown, one under the other.

    <cartridge-tabs label="cartridge" hash>
      <a href="#home">HOME</a>
      <a href="#install">INSTALL</a>
    </cartridge-tabs>
    <section id="home">…</section>
    <section id="install">…</section>

  label  what the tabs are, said to a screen reader (aria-label).
  hash   the URL's hash follows the tab shown (none for the first), and the tab follows the hash:
         a link to #install, the back button, or a link to something inside a part, which then
         scrolls to it, all show the right tab.

  It is a tablist as ARIA has it: each link becomes a tab and the part it names its tabpanel,
  every part but the shown one hidden. Left and Right move to the tab before or after (round from
  the ends), Home and End to the first and last, and the tab moved to is shown at once; a click,
  Enter or Space shows the tab pressed. Each change sends `cartridge-tab` (detail: index, id, tab,
  panel), from the first tab shown on; select(index or id) shows one from a script, and `index` is
  the one shown.

  Each tab is 12 art pixels tall, as a panel's title tab: its outline in the frame colour, its
  title Pixel Code one face pixel to an art pixel on a line of 10, capitals on rows 3 to 9, 4
  pixels in from the outline and 3 before the slant, which is 13 wide. The tabs sit side by side
  and wrap into more rows when the page is narrower than all of them, each row on its own strip
  and stripe line, round them the panel's edge line with its corner pixels left out. The tab
  shown has the tab colour and a cream title; the others the dark colour and a dim title, cream
  under the pointer. Keyboard focus underlines the title in cream. Areas, each a div that ::part()
  reaches: bar. How the links are drawn is the component's, marked important so the page's own
  rules for links cannot reach them.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // A tab's height in art pixels, its outline rows included, and its slanted end's width.
  const H = 12;
  const SLANT = 13;

  // A tab, a 9-slice: the outline down its left, the column that stretches (the outline at top
  // and bottom, `fill` between), then the slanted end as <cartridge-panel> draws it: one pixel
  // further right each row, a frame pixel and a stripe pixel, nothing after them, where the
  // strip shows.
  function tab(fill) {
    return Array.from({ length: H }, (_, y) => {
      const inside = y === 0 || y === H - 1 ? 'A' : fill;
      return 'A' + inside + inside.repeat(y) + 'As' + '.'.repeat(SLANT - y - 2);
    });
  }

  // The strip behind a row of tabs, a row of it at a time: the bright strip as tall as a tab,
  // then the stripe line under it.
  const STRIP = [...Array(H).fill('H'), 's'];

  // The edge line round the rows: the panel's stripe line, its corner pixel left out.
  const EDGE = [
    '.s',
    's.',
  ];

  // The fill of a tab shown and of one not.
  const FILL = { on: 't', off: 'k' };

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: block; min-width: 0; text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      /* the rows, a pixel apart for the stripe line under each; the strip's sprite repeats down
         the rows from the top, inside the edge line */
      .bar {
        display: flex;
        flex-wrap: wrap;
        row-gap: var(--u);
        border: var(--u) solid transparent;
        border-image-source: var(--edge);
        border-image-slice: 1;
        border-image-repeat: stretch;
        background-image: var(--strip);
        background-size: var(--u) calc(${H + 1} * var(--u));
        background-clip: padding-box;
        background-origin: padding-box;
        image-rendering: pixelated;
      }

      /* each tab the page's own link, as tall as a tab and as wide as its title: the outline and
         the slant drawn round it, the title on line 1 of the tab's 12 rows */
      ::slotted(a), ::slotted(button) {
        all: unset !important;
        box-sizing: border-box !important;
        display: block !important;
        height: calc(${H} * var(--u)) !important;
        padding: 0 calc(3 * var(--u)) 0 calc(4 * var(--u)) !important;
        border-style: solid !important;
        border-color: transparent !important;
        border-width: var(--u) calc(${SLANT} * var(--u)) var(--u) var(--u) !important;
        border-image-source: var(--tab-off) !important;
        border-image-slice: 1 ${SLANT} 1 1 fill !important;
        border-image-repeat: stretch !important;
        image-rendering: pixelated !important;
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font) !important;
        white-space: nowrap !important;
        color: var(--oz-dim) !important;
        cursor: pointer !important;
        outline: none !important;
      }
      ::slotted(a:hover), ::slotted(button:hover) { color: var(--oz-cream) !important; }
      ::slotted([aria-selected="true"]) {
        border-image-source: var(--tab-on) !important;
        color: var(--oz-cream) !important;
        cursor: default !important;
      }
      /* keyboard focus: the title underlined in cream along the tab's bottom edge, a row clear of its letters */
      ::slotted(:focus-visible) {
        text-decoration: underline !important;
        text-decoration-color: var(--oz-cream) !important;
        text-decoration-thickness: var(--u) !important;
        text-underline-offset: var(--u) !important;
      }
    </style>
    <div class="bar" part="bar"><slot></slot></div>`;

  customElements.define('cartridge-tabs', class extends CartridgePixel.Element {
    static observedAttributes = ['label'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
      this.tabs = [];
      this.index = -1;
      this.shadowRoot.querySelector('slot').addEventListener('slotchange', () => this.ready());
      this.addEventListener('click', event => this.pressed(event));
      this.addEventListener('keydown', event => this.key(event));
      this.hashed = () => this.followHash();
    }

    connectedCallback() {
      super.connectedCallback();
      this.setAttribute('role', 'tablist');
      window.addEventListener('hashchange', this.hashed);
      this.ready();
    }

    disconnectedCallback() {
      super.disconnectedCallback();
      window.removeEventListener('hashchange', this.hashed);
    }

    paint() {
      const { sprite, mirror } = CartridgePixel;
      const style = this.shadowRoot.querySelector('.bar').style;
      style.setProperty('--edge', sprite(mirror(EDGE), this));
      style.setProperty('--strip', sprite(STRIP, this));
      style.setProperty('--tab-on', sprite(tab(FILL.on), this));
      style.setProperty('--tab-off', sprite(tab(FILL.off), this));
    }

    attributeChangedCallback(name, _old, value) {
      if (name !== 'label') return;
      if (value) this.setAttribute('aria-label', value);
      else this.removeAttribute('aria-label');
    }

    // The tabs and their parts, once the page has them all: the parts come after the tabs.
    ready() {
      if (!this.isConnected) return;
      if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => this.ready(), { once: true });
        return;
      }
      const tabs = this.shadowRoot.querySelector('slot').assignedElements()
        .filter(el => el.matches('a, button'));
      if (tabs.length === this.tabs.length && tabs.every((tab, i) => tab === this.tabs[i])) return;
      this.tabs = tabs;
      for (const tab of tabs) {
        const id = this.target(tab);
        const panel = id && document.getElementById(id);
        tab.setAttribute('role', 'tab');
        if (!tab.id) tab.id = `${id || 'tab'}-tab`;
        if (!panel) continue;
        tab.setAttribute('aria-controls', id);
        panel.setAttribute('role', 'tabpanel');
        panel.setAttribute('aria-labelledby', tab.id);
      }
      const chosen = tabs.findIndex(tab => tab.getAttribute('aria-selected') === 'true');
      if (!(this.hasAttribute('hash') && this.followHash())) this.select(Math.max(0, chosen));
    }

    // The id of the part a tab shows: its aria-controls, else its link's hash.
    target(tab) {
      const controls = tab.getAttribute('aria-controls');
      if (controls) return controls;
      const href = tab.getAttribute('href') ?? '';
      return href.startsWith('#') ? decodeURIComponent(href.slice(1)) : '';
    }

    panel(i) {
      const id = this.target(this.tabs[i]);
      return (id && document.getElementById(id)) || null;
    }

    // Shows tab `which`, an index or a part's id. `focus` moves the focus to it; `history` is
    // 'push' or 'replace' for how the URL's hash follows, with the hash attribute.
    select(which, { focus = false, history = null } = {}) {
      const index = typeof which === 'number'
        ? which
        : this.tabs.findIndex(tab => this.target(tab) === which);
      if (index < 0 || index >= this.tabs.length) return;
      const changed = index !== this.index;
      this.tabs.forEach((tab, i) => {
        const on = i === index;
        tab.setAttribute('aria-selected', String(on));
        tab.tabIndex = on ? 0 : -1;
        const panel = this.panel(i);
        if (panel) panel.hidden = !on;
      });
      this.index = index;
      if (focus) this.tabs[index].focus();
      if (history && this.hasAttribute('hash')) {
        const hash = index === 0 ? '' : `#${encodeURIComponent(this.target(this.tabs[index]))}`;
        if (hash !== location.hash) {
          window.history[history === 'push' ? 'pushState' : 'replaceState'](null, '', hash || location.pathname + location.search);
        }
      }
      if (changed) {
        this.dispatchEvent(new CustomEvent('cartridge-tab', {
          bubbles: true,
          detail: { index, id: this.target(this.tabs[index]), tab: this.tabs[index], panel: this.panel(index) },
        }));
      }
    }

    // Shows the tab the URL's hash names, its part or something inside it, and scrolls to what it
    // names: for a whole part, back up to the tabs if they are above the window. False when the
    // hash names nothing in a part.
    followHash() {
      const id = decodeURIComponent(location.hash.slice(1));
      const named = id && document.getElementById(id);
      const index = named
        ? this.tabs.findIndex((_, i) => this.panel(i)?.contains(named))
        : -1;
      if (index < 0) {
        if (!id && this.tabs.length) {
          this.select(0);
          return true;
        }
        return false;
      }
      this.select(index);
      requestAnimationFrame(() => {
        if (named !== this.panel(index)) named.scrollIntoView();
        else if (this.getBoundingClientRect().top < 0) this.scrollIntoView();
      });
      return true;
    }

    pressed(event) {
      const tab = event.target.closest?.('[role="tab"]');
      const index = this.tabs.indexOf(tab);
      if (index < 0 || event.button || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      event.preventDefault();
      this.select(index, { history: 'push' });
    }

    key(event) {
      const at = this.tabs.indexOf(event.target);
      if (at < 0) return;
      const count = this.tabs.length;
      const to = { ArrowLeft: at - 1, ArrowRight: at + 1, Home: 0, End: count - 1 }[event.key];
      if (to !== undefined) {
        event.preventDefault();
        this.select((to + count) % count, { focus: true, history: 'replace' });
      } else if (event.key === ' ') {
        event.preventDefault();
        this.select(at, { history: 'push' });
      }
    }
  });
})();
