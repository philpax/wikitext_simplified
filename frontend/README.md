# Wikitext Simplified Frontend

A web-based visualization tool for parsing and exploring MediaWiki wikitext. This frontend wraps the [wikitext_simplified](https://github.com/philpax/wikitext_simplified) Rust library compiled to WebAssembly.

## Features

- **Live Parsing**: Wikitext is parsed in real-time as you type (with 300ms debounce)
- **AST Tree View**: Collapsible tree visualization of the parsed abstract syntax tree
- **HTML Preview**: Rendered preview of the wikitext with template placeholders
- **Source Highlighting**: Hover over tree nodes to highlight the corresponding source text
- **Click to Select**: Click tree nodes to select the text range in the editor
- **Example Snippets**: Pre-built examples demonstrating various wikitext features

## Development

### Prerequisites

- Node.js 20+
- Rust toolchain with `wasm32-unknown-unknown` target
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)

### Setup

From the repository root:

```bash
# Build WASM and install dependencies
python build-frontend.py

# Start development server
python build-frontend.py --dev
```

Or manually:

```bash
# Build WASM bindings
wasm-pack build wikitext-wasm --target web --out-dir ../frontend/src/wasm

# Install dependencies
cd frontend
npm install

# Start dev server
npm run dev
```

### Production Build

```bash
python build-frontend.py --build
```

The production build will be in `frontend/dist/`.

## Deployment

The frontend auto-deploys to [philpax.me/experimental/wikitext](https://philpax.me/experimental/wikitext/) on pushes to `main` via GitHub Actions.
