# @nest/components (vendored)

A vendored copy of the Nest framework's `@nest/components` React library
(`core/crates/nest-react-components` in the main `nest` repo). Kiwi is a
separate git repository with no shared npm workspace, so this can't be
consumed via a relative path the way Nest Desktop's `ui/` does — it's
copied in as source instead, aliased via `@nest/components` in
`vite.config.ts`.

Not vendored (dropped as dead weight or out of scope for this port):
`components/ComponentsApp.tsx` (an orphaned, unused browser — Nest Desktop
actually uses its own `NestUIBrowser.tsx`; Kiwi's equivalent is the
Components Activity panel), `*.demo.tsx`/`*.docs.md` (unused by any
browser in the source library either), `*.test.tsx`/`test/setup.ts`
(vitest isn't part of Kiwi's toolchain), and `styles.css` (just
`--nest-color-*` fallback variables — Kiwi already defines these itself
via `nest-theme`).

To pick up upstream changes, re-copy the relevant files from
`core/crates/nest-react-components/src/` in the `nest` repo and re-check
this list against its current contents.
