# @emerald-show/emerald-background

Importable browser package for the Emerald Show background scene.

```js
import { initEmeraldBackground } from "@emerald-show/emerald-background";

initEmeraldBackground();
```

The default options target `.emerald-bg`, load `/dflat.glb`, and update
`#copyright-year`.

Small screens (up to 768px) and touch-first devices use a still image of the
emerald scene (`public/emerald-background-static.png`) instead of initializing
WebGL. The site's glass panels retain their backdrop blur over that image.

