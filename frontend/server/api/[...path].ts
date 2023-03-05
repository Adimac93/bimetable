import { proxyBackend } from "../utils/proxy";

export default defineEventHandler(async (event) => {
    return proxyBackend(event, event.path!.slice(5));
});
