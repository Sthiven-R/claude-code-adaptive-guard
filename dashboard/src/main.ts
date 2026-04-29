import App from "./App.svelte";
import "./app.css";
import { mount } from "svelte";

const target = document.getElementById("app");
if (!target) throw new Error("#app mount point not found");

const app = mount(App, { target });

export default app;
