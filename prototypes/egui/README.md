# Second Life `egui` Mockup

Quick, throwaway UI prototypes of Second Life viewer screens built in
[egui](https://github.com/emilk/egui). The goal is to iterate on layout & flow
fast, render in the browser, and preview/drive it from Claude via the Chrome MCP
tools, **not** to reproduce the pixel-exact XUI skin.

Reference for the real screens lives at `indra/newview/skins/default/xui/` (e.g.
`panel_login.xml`, `menu_login.xml`). Colors come from `colors.xml` and
translations are harvested straight from the localized XUI files.

## Layout

```
src/
  main.rs        native + WASM entry points (one file, cfg-split)
  app.rs         eframe App shell: screen / language / theme / radius / font pickers
  theme.rs       semantic color palettes (5 themes) + twill sizing/radius tokens
  theme/fonts.rs embedded font faces + UI/Mono font pickers
  data.rs        mock data layer: provider traits + JSON-fixture-backed MockData
  i18n.rs        Fluent runtime: static loader + tr() / tr_args()
  bin/harvest.rs build-time tool: XUI .xml -> i18n/<locale>/*.ftl
  screens/
    login.rs     login screen + menu bar, modeled on panel_login / menu_login
fixtures/
  login.json     LLSD-shaped mock payload the login screen renders against
assets/
  fonts/         TTF/OTF UI fonts (converted from Alchemy's woff2), embedded
i18n/
  <locale>/*.ftl generated per-locale messages (en is the fallback)
index.html       trunk web harness (canvas + wasm bootstrap)
Trunk.toml       dev server config (127.0.0.1:8080, live reload)
```

## Toolchain (one-time)

```sh
brew install rustup trunk
rustup default stable
rustup target add wasm32-unknown-unknown
```

`rustup` is keg-only on Homebrew; if `rustup`/`cargo` aren't found, prepend:
`export PATH="$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"`.

## Run

**Web (the preview loop):**
```sh
trunk serve        # http://127.0.0.1:8080  (live-reloads on save)
```
Edit any `.rs` → trunk rebuilds the WASM → the browser tab refreshes itself.
Claude then navigates Chrome (MCP) to `127.0.0.1:8080` and screenshots to see
the result.

**Native (fastest iteration, no browser):**
```sh
cargo run          # opens a 1024x768 desktop window
```

## Theming (semantic tokens + `twill`)

`theme.rs` is the design system. Colors are shadcn-style **semantic tokens**
(`background`, `foreground`, `primary`, `border`, `input`, `ring`, ...) grouped
into a `Palette`. Five palettes are hand-translated from the skins Alchemy ships
(`default`, `alchemy`, `gemini`, `heretic`, `ionic`). Sizes, spacing, and radius
come from [twill](https://ferrismind.github.io/twill) tokens, so screens carry
no raw pixel literals (`theme::space(Spacing::S7)`, `theme::field_w()`, etc.).
twill's `Color` is Tailwind-only, so SL's exact brand colors are carried through
twill's `ColorValue`. egui pins to 0.33 for twill 0.2's egui backend.

The top bar switches **theme**, **corner radius** (None/Small/Medium/Large/Full),
**UI font**, and **mono font** live. The active theme and radius are stashed in
the egui context and read back via `theme::active(ctx)` / `theme::active_radius(ctx)`,
so no state is threaded through screen functions.

### Fonts

egui only reads TTF/OTF, so Alchemy's `.woff2` UI faces were converted to TTF
once and the curated set lives in `assets/fonts/`, embedded at build time
(`theme/fonts.rs`). The default UI font is Inter and the default mono is Cascadia
Code, matching Alchemy's `SansSerifBase` / `MonospaceBase`.

## i18n (Fluent, harvested from XUI)

UI strings reuse Second Life's existing translations. `bin/harvest.rs` reads the
localized XUI skin tree directly and emits Fluent `.ftl` per locale; the runtime
(`i18n.rs`) embeds them with a compile-time static loader (WASM-safe), with
English as the fallback. The top-bar switcher changes locale live.

```sh
cargo run --features harvest --bin harvest   # regenerate i18n/<locale>/*.ftl
```

Each curated key in `harvest.rs`'s `ENTRIES` points at a source element
(`file` + `name` + attribute/text); the harvester pulls that string for every
locale that has it. `[BRACKET]` placeholders become Fluent `{ $var }`. Add a
screen's strings by extending `ENTRIES` and re-running the harvester.

## Adding a screen

1. Add a fixture under `fixtures/` (shape it like the real LLSD payload).
2. Add a provider trait + mock impl in `src/data.rs`.
3. Add the screen's strings to `ENTRIES` in `bin/harvest.rs`, run the harvester.
4. Add `src/screens/<name>.rs` and a `Screen` variant in `src/screens/mod.rs`,
   rendering against the provider and `tr(lang, ..)`.
5. The top-bar picker switches between screens automatically.

## Hot-reload options

- **WASM (`trunk serve`)**: recompile + auto browser refresh. Mock state resets
  on reload, which is fine since it re-seeds from fixtures. This is the default.
- **Native state-preserving**: for true hot-swap that keeps live UI state, add
  [`hot-lib-reloader`](https://github.com/rksm/hot-lib-reloader-rs) and move the
  screen `update` fns into a `dylib`. Not set up here yet; reach for it only if
  reset-on-reload becomes annoying.
