# tsap-oo-web

Play 十胡 against the [tsap-oo](..) bots in the browser.  The whole game runs
client-side as WebAssembly; there is no server.

## Build

You need the `wasm32-unknown-unknown` target and `wasm-pack`:

```console
rustup target add wasm32-unknown-unknown   # once
cargo install wasm-pack                    # once

wasm-pack build --release --target web     # writes ./pkg/
```

Notes:

- `.cargo/config.toml` clears a global `-Ctarget-cpu=native` for the wasm build.
  Left in place, that flag (harmless on native builds, meaningless for wasm)
  corrupts the module's target features and `wasm-bindgen` then fails with
  `failed to find intrinsics to enable "clone_ref"`.
- `getrandom = { features = ["wasm_js"] }` in `Cargo.toml` names getrandom's
  browser backend so the wasm target compiles — we never call it (the RNG is
  seeded from JS), but the crate still has to name a backend.

## Play

Serve this directory over HTTP — ES modules and wasm won't load from `file://`:

```console
python3 -m http.server
# open http://localhost:8000/
```

You sit south; the bots take the other three seats.  When a public card is
yours to claim, the buttons under your hand offer 碰, each possible 吃, and
過; when it is your discard, click a card.  Wins and kongs are compulsory and
happen by themselves, and so does keeping a general (將帥) you draw.

Generals cannot be discarded, so they are dimmed in your hand.  A 碰 or 吃
that would leave you nothing to discard is shown as a disabled button;
running out of discards anyway (相公) ends the round and costs you ten 胡 to
each bot.

## Deploy

`pkg/`, `index.html`, `app.js`, and `style.css` are all static — push them to
GitHub Pages or any static host.

## Test

The game flow is native-testable without a browser:

```console
cargo test
```
