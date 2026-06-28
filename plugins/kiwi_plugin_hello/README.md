# kiwi_plugin_hello

Reference sample plugin for [SPEC-020](https://github.com/pacificnm/kiwi/blob/main/docs/specs/SPEC-020-plugin-framework.md).
Registers a single command palette entry: **Hello Plugin: Greet** (`hello.greet`).

## Build

From the repository root:

```bash
cargo build -p kiwi_plugin_hello
```

The dynamic library is written to `target/debug/libkiwi_plugin_hello.so` (`.dylib` on macOS).

## Install

### Using the kiwi CLI (recommended)

Build the plugin, then create a staging directory with the manifest and library:

```bash
# Build
cargo build -p kiwi_plugin_hello

# Create a staging directory kiwi plugin install can copy from
STAGING=/tmp/hello-plugin
mkdir -p "${STAGING}"
cp plugins/kiwi_plugin_hello/plugin.toml "${STAGING}/"
cp target/debug/libkiwi_plugin_hello.so "${STAGING}/"   # .dylib on macOS

# Install — copies to ~/.config/kiwi/plugins/hello/ and registers it
kiwi plugin install "${STAGING}"
```

Verify the installation:

```bash
kiwi plugin list
kiwi plugin info hello
```

### Manual install

```bash
PLUGIN_DIR="${HOME}/.config/kiwi/plugins/hello"
mkdir -p "${PLUGIN_DIR}"
cp plugins/kiwi_plugin_hello/plugin.toml "${PLUGIN_DIR}/"
cp target/debug/libkiwi_plugin_hello.so "${PLUGIN_DIR}/"
```

Kiwi will auto-register the plugin on next startup.

## Verify

Start Kiwi. The **Logs** tab shows `Plugin 'hello' loaded (1 command(s))`.

Open the command palette (`Ctrl+P`) and search for **hello** — you should see
**Hello Plugin: Greet**.

To inspect status in the Plugin Manager:

- **TUI**: press `9` or search the palette for **"Plugins: Open Manager"**.
- **GUI**: **File → Plugins** menu or enable the **Plugins** panel from **View**.

## Enable / disable

```bash
kiwi plugin disable hello   # takes effect on next restart
kiwi plugin enable  hello   # takes effect on next restart
```

## Uninstall

```bash
kiwi plugin remove hello    # removes from registry (files kept)
rm -rf ~/.config/kiwi/plugins/hello   # optionally delete files
```

## Layout

```text
~/.config/kiwi/plugins/hello/
├── plugin.toml
└── libkiwi_plugin_hello.so
```

Kiwi discovers any native library (`.so` / `.dylib`) in the plugin subdirectory alongside `plugin.toml`.
