// explicitly get the setResponseStatus from h3
import { setResponseStatus } from "h3";
import { Readable } from "node:stream";
import type { H3Event } from "h3";

export async function proxyBackend(event: H3Event, path: string) {
    // construct the URL
    const url = new URL(useRuntimeConfig().apiURL);
    url.pathname = path;

    // get request parameters
    const method = getMethod(event);
    const headers = getProxyRequestHeaders(event);

    // only send the body if the method is appropriate and if it exists
    let body;
    if (method == "GET" || method == "HEAD") {
        body = undefined;
    } else {
        body = await readRawBody(event, false);
    }

    console.log("proxying to:", url.toString());

    const response = await fetch(url, {
        method,
        headers,
        body,
        redirect: "manual",
        credentials: "include",
        mode: "cors",
    });

    // TODO: if response is 401 Unauthorized, try refreshing the auth token

    // TODO: if response is a redirect, rewrite target url to point at this proxy

    // set all of the appropriate response options
    setResponseStatus(event, response.status);
    const responseHeaders: Record<string, string> = {};
    response.headers.forEach((val, key) => {
        responseHeaders[key] = val;
    });
    setResponseHeaders(event, responseHeaders);
    if (response.body) {
        return sendStream(event, Readable.fromWeb(response.body as any));
    }
    return sendNoContent(event, response.status);
}
