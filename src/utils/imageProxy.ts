/**
 * Rewrites an external image URL to go through the uex-img custom protocol,
 * which proxies the request through the Rust backend to bypass hotlink protection.
 *
 * On Windows (WebView2), custom schemes use http://{scheme}.localhost/
 */
export function proxyImageUrl(url: string): string {
  return url.replace(/^https?:\/\//, "http://uex-img.localhost/");
}
