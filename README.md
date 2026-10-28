# Svelte for Zed, with svelte-language-server-rs

A Svelte extension for [Zed](https://zed.dev) that replaces the official one:

- **Grammar:** [tree-sitter-htmlx](https://github.com/themixednuts/tree-sitter-htmlx)'s Svelte
  grammar, which parses Svelte 5 (runes, snippets, `{@attach}`, `{@const}`, `{#snippet}`), with
  fixes for real-world parse errors and block braces kept apart from their `#` `:` `@` `/` sigil.
- **Highlighting** that follows VS Code's Svelte grammar: `if`/`else`/`each` as control keywords,
  the sigil as punctuation, `svelte:` as a namespace, `on:` `bind:` `use:` and the other directives,
  and rainbow brackets on `{ }` without colouring the sigil or the markup's `< >`.
- **Language server:** [svelte-language-server-rs](https://github.com/hanielu/svelte-language-server-rs),
  rsvelte's Rust Svelte server with tsrs as its type checker, downloaded for you. Neither needs Node.
- **Semantic token rules** so `const` bindings and readonly properties get your theme's constant
  colour, as in VS Code.

## It must be your only Svelte extension

Zed keeps one language per name: when two extensions both define "Svelte", one silently replaces
the other. This extension's ID is `svelte`, the same as the official Svelte extension, so
installing it as a dev extension uninstalls the official one, and the Extensions page shows that
extension as "Overridden by dev extension". Do not install any other extension that defines a
"Svelte" language. Extensions that only attach to Svelte files (Tailwind CSS, ESLint, UnoCSS) are
fine.

## Install

It is not in Zed's extension registry, so it is installed as a dev extension:

1. Install Rust with [rustup](https://rustup.rs). Zed adds the `wasm32-wasip2` target itself and
   downloads the wasi-sdk it compiles the grammar with. You also need `git`.
2. Clone this repository somewhere it can stay, since Zed links to the folder rather than copying it:

   ```sh
   git clone https://github.com/hanielu/zed-svelte-rs
   ```

3. In Zed, open the Extensions page, click **Install Dev Extension** (or run
   `zed: install dev extension`) and choose the cloned folder.
4. Turn on the server's semantic tokens for Svelte in your `settings.json`:

   ```jsonc
   "languages": {
     "Svelte": { "semantic_tokens": "combined" }
   }
   ```

The first time you open a Svelte file, the extension downloads the server (and tsrs if needed)
into Zed's extension work folder.

## Updating

Dev extensions do not update themselves. To update the grammar, highlighting or this launcher, run
`git pull` in the clone and press the rebuild button next to the extension on the Extensions page.
The server is different: the first time the extension starts it in a Zed session, it checks for
a new svelte-language-server-rs release, and keeps using the last download when offline.

## Which server and tsrs it runs

The server, first match wins:

1. `lsp.svelte-language-server.binary.path` in your settings;
2. `svelte-language-server-rs` on your shell's `PATH`;
3. the newest [release](https://github.com/hanielu/svelte-language-server-rs/releases) for your
   platform (macOS arm64 and x64, Linux arm64 and x64 with glibc, Windows x64).

tsrs: if your shell's `PATH` has `tsrs`, or the project root has `node_modules/.bin/tsrs`, the
server uses that. Otherwise the extension downloads the newest `@tsrs/<platform>` package straight
from the npm registry and puts it last on the server's `PATH`, so a project-pinned tsrs still wins.

The server's own settings go in `lsp.svelte-language-server.settings`, as described in the
[svelte-language-server-rs README](https://github.com/hanielu/svelte-language-server-rs#configuration).
The language server keeps the official extension's ID, `svelte-language-server`, so existing
`lsp.svelte-language-server` and `language_servers` settings keep working.

If you use vtsls for TypeScript, the extension adds `typescript-svelte-plugin` to it as the
official extension does, so `.ts` files can import `.svelte` components. That one step installs an
npm package with Zed's Node.

## Dev extension trade-offs

- **You need Rust installed**, and installing or rebuilding compiles the extension (about 15 s on
  an M-series Mac, longer the first time while Zed fetches its build tools).
- **No automatic updates** for the extension itself; see [Updating](#updating).
- **The extension is a link to your clone**: moving or deleting the folder breaks it.
- **Speed is the same as a published extension.** Zed builds dev extensions without `--release`,
  which would make this extension's code 8.2 MB instead of 397 KB; `Cargo.toml` optimises that
  build, so it comes out at 444 KB. That code only runs when Zed starts the server. The grammar is
  compiled with the same flags Zed uses for published extensions, and the server is a native
  binary either way.

## Going back to the official extension

Uninstall this extension on the Extensions page, then install "Svelte" from the registry.

## License

Apache-2.0, see [LICENSE](LICENSE). Based on [zed-extensions/svelte](https://github.com/zed-extensions/svelte)
(Apache-2.0): the queries were rewritten for tree-sitter-htmlx and the language server launcher was
replaced.
