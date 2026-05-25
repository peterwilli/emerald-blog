# EmeraldBlogJS

Legacy JavaScript bundle source for the Emerald Show blog background scene.

The active browser bundle now lives in `../core-js`, and the importable
background package lives in `../packages/emerald-background`.

## Build

Install dependencies with pnpm:

```sh
pnpm install --frozen-lockfile
```

Build the bundle:

```sh
pnpm build
```

The build runs Parcel against `index.js` and writes `EmeraldBlog.js` and
`EmeraldBlog.js.map` in this directory.

## Updating the Site Asset

The Leptos app no longer serves this bundle. Use the root build instead:

```sh
pnpm build:core-js
```
