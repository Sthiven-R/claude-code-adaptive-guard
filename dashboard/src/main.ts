import App from "./App.svelte";

// Order matters: tokens define the variables that app.css and every
// component consume. Loading tokens first guarantees they are
// resolvable when the rest of the cascade is parsed. Self-hosted
// fonts (Inter + JetBrains Mono) ship with the bundle so the app
// renders identically offline — a hard requirement for Tauri.
import "./lib/styles/tokens.css";
import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "@fontsource/jetbrains-mono/400.css";
import "@fontsource/jetbrains-mono/600.css";
import "./app.css";

// Side-effect import: applies the persisted theme to <html> before
// the first paint, so we never get a "wrong-theme flash" on launch.
import "./lib/stores/theme";

import { mount } from "svelte";

const target = document.getElementById("app");
if (!target) throw new Error("#app mount point not found");

const app = mount(App, { target });

export default app;
