import App from "./App.svelte";

// Order matters: tokens define the variables that app.css and every
// component consume. Loading tokens first guarantees they are
// resolvable when the rest of the cascade is parsed. Self-hosted
// fonts (Inter + JetBrains Mono) ship with the bundle so the app
// renders identically offline — a hard requirement for Tauri.
//
// We import the explicit `latin` and `latin-ext` subsets instead of
// the umbrella `400.css` etc. The umbrella files @import every
// subset (cyrillic, greek, vietnamese, etc.) — even though browsers
// only fetch the woff2 they actually need, the @font-face rules
// still inflate CSSOM. The dashboard UI is EN/ES only, so latin +
// latin-ext is the complete needed range.
import "./lib/styles/tokens.css";
import "@fontsource/inter/latin-400.css";
import "@fontsource/inter/latin-ext-400.css";
import "@fontsource/inter/latin-500.css";
import "@fontsource/inter/latin-ext-500.css";
import "@fontsource/inter/latin-600.css";
import "@fontsource/inter/latin-ext-600.css";
import "@fontsource/jetbrains-mono/latin-400.css";
import "@fontsource/jetbrains-mono/latin-ext-400.css";
import "@fontsource/jetbrains-mono/latin-600.css";
import "@fontsource/jetbrains-mono/latin-ext-600.css";
import "./app.css";

// Side-effect import: applies the persisted theme to <html> before
// the first paint, so we never get a "wrong-theme flash" on launch.
import "./lib/stores/theme";

import { mount } from "svelte";

const target = document.getElementById("app");
if (!target) throw new Error("#app mount point not found");

const app = mount(App, { target });

export default app;
