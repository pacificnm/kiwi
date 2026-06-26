# kiwi_plugin_hello

Reference sample plugin for [SPEC-020](https://github.com/pacificnm/kiwi/blob/main/docs/specs/SPEC-020-plugin-framework.md).
Registers a single command palette entry: **Hello Plugin: Greet** (`hello.greet`).

## Build

From the repository root:

```bash
cargo build -p kiwi_plugin_hello
```

The dynamic library is written to `target/debug/libkiwi_plugin_hello.so` (`.dylib` on macOS).

## Install for local testing

```bash
PLUGIN_DIR="${HOME}/.config/kiwi/plugins/hello"
mkdir -p "${PLUGIN_DIR}"
cp plugins/kiwi_plugin_hello/plugin.toml "${PLUGIN_DIR}/"
cp target/debug/libkiwi_plugin_hello.so "${PLUGIN_DIR}/"
```

Start Kiwi, open the Logs tab for a `Plugin 'hello' loaded` message, then press `Ctrl+P` and search for `hello`.

## Layout

```text
~/.config/kiwi/plugins/hello/
├── plugin.toml
└── libkiwi_plugin_hello.so
```

Kiwi discovers any native library (`.so` / `.dylib`) in the plugin subdirectory alongside `plugin.toml`.
