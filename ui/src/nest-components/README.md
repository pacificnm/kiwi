# @nest/components (vendored)

A vendored copy of the Nest framework's `@nest/components` React library
(`core/crates/nest-react-components` in the main `nest` repo). Kiwi is a
separate git repository with no shared npm workspace, so this can't be
consumed via a relative path the way Nest Desktop's `ui/` does — it's
copied in as source instead, aliased via `@nest/components` in
`vite.config.ts`.

## What's vendored

The full component library **plus** each component's `*.demo.tsx` (live
examples) and `<Name>.docs.md` (usage docs). The Components Activity panel
(`workbench/ComponentsPanel.tsx` + `ComponentDetailView.tsx`) is data-driven:
it discovers components by globbing the vendored `*.docs.md`, renders the
matching `*.demo.tsx` in the Preview tab, the markdown in the Documentation
tab, and the demo source in the Code tab. Newly synced components appear
automatically — no manual registry.

Docs live next to their component as `<Name>.docs.md`. In the upstream repo
they instead live under `docs/nest-react-components/<category>/<Name>.md`; the
sync copies each one next to its component.

## Not vendored

`*.test.tsx` / `test/setup.ts` (vitest isn't part of Kiwi's toolchain),
`components/ComponentsApp.tsx` (an orphaned upstream browser), and
`styles.css` (just `--nest-color-*` fallback vars — Kiwi defines these itself
via `nest-theme`). `runtime.css` (LinearProgress keyframes) **is** vendored
and imported from `index.ts`.

## Syncing upstream changes

Re-copy from `core/crates/nest-react-components/src/` in the `nest` repo:

```sh
# from the nest repo root
SRC=core/crates/nest-react-components/src
DST=apps/kiwi/ui/src/nest-components
rsync -a --delete --exclude='*.test.tsx' --exclude='test/' "$SRC/components/" "$DST/components/"
rsync -a "$SRC/context/" "$DST/context/"; rsync -a "$SRC/hooks/" "$DST/hooks/"; rsync -a "$SRC/lib/" "$DST/lib/"
cp "$SRC/index.ts" "$DST/index.ts"; cp "$SRC/runtime.css" "$DST/runtime.css"
# docs: place each upstream doc next to its component
for md in docs/nest-react-components/*/*.md; do
  name=$(basename "$md" .md)
  comp=$(find "$DST/components" -name "$name.tsx" ! -name '*.demo.tsx' | head -1)
  [ -z "$comp" ] && comp=$(find "$DST/components" -name "$name.demo.tsx" | head -1)
  [ -n "$comp" ] && cp "$md" "$(dirname "$comp")/$name.docs.md"
done
```

If upstream adds new runtime dependencies (e.g. `@floating-ui/react`,
`@react-aria/*`), install them in `ui/` too.
