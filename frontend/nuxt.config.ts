// https://nuxt.com/docs/api/configuration/nuxt-config
import settings from "./configuration/settings.json";

export default defineNuxtConfig({
    // these may be overwritten by env vars
    runtimeConfig: {
        apiURL: settings.apiURL ?? "http://127.0.0.1:3001",
    },
    css: ["assets/global.scss", "assets/forms.scss"],
});
