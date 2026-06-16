# plotkit WASM demo

Renders plotkit charts in the browser with the `plotkit-render-wasm` Canvas2D backend.

## Build

Requires [`wasm-pack`](https://rustwasm.github.io/wasm-pack/):

```bash
wasm-pack build crates/plotkit-render-wasm --target web --out-dir ../../web/pkg
```

This emits `web/pkg/plotkit_render_wasm.js` + `.wasm`, which `index.html` imports.

## Serve

Any static file server works (ES modules require HTTP, not `file://`):

```bash
python -m http.server --directory web 8080
# open http://localhost:8080
```

The page calls the exported `render_demo(canvas, kind)` function, where `kind`
is `"line"`, `"scatter"`, `"bar"`, or `"hist"`.
