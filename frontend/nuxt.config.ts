// https://nuxt.com/docs/api/configuration/nuxt-config
import settings from "./configuration/settings.json";

export default defineNuxtConfig({
    // these may be overwritten by env vars
    appConfig: {
        apiURL: settings.apiURL
    }
})
