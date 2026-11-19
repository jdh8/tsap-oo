# tsap-oo-tauri

A desktop client for [tsap-oo](..), built on [Tauri](https://tauri.app).  The
window shows the browser client's own page ([`web/`](../web): `index.html`,
`app.js`, `style.css`, embedded at build time), so there is one UI to
maintain.  Only the engine differs: `app.js` finds `window.__TAURI__` and
calls the commands in `src/main.rs` instead of loading the wasm.

What native buys is opponents that search.  The three bots are
`ExpectBot`s that play by the single-player DP of
[`doc/expectation.md`](../doc/expectation.md) with 14 concealed cards or fewer,
and before by a graph two claims deep to 17 cards and one beyond, its leaves
valued by the pattern tables of [`doc/eval.md`](../doc/eval.md).  A decision
takes some 60 ms of native code on average, 1.4 s at worst, on an engine
thread, so the window never stalls on it.  The gate is not a setting: no other
gate has a verdict.

The hint is the same `ExpectBot`'s, as in the browser: its marks are the
expected points a move gives up.

Settings are the page's: they live in the webview's `localStorage`, apart
from any browser's.

## Build and run

```console
cargo run --release
```

From the repository root, `cargo desktop` is the same command.

No Tauri CLI and no wasm build are needed: `cargo` embeds the three files
`tauri.conf.json` lists.  A change to `web/` shows after the next build.

On Linux Tauri links WebKitGTK 4.1.  On Fedora:

```console
sudo dnf install webkit2gtk4.1-devel gtk3-devel libsoup3-devel
```

## Adding an engine call

Three places, all spelled alike: the `WebGame` method in `web/src/lib.rs`, a
`#[tauri::command]` in `src/main.rs` (and its name in `generate_handler!`),
and one line in each half of `loadEngine()` in `web/app.js`.

## Test

```console
cargo test
```

The test deals a game and follows the hint through a whole round over the
commands, against the searching opponents.  The page itself is not exercised.
