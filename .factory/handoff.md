# Repair handoff — PASS

**Work order:** `selfhost-observability-fit-repair-1`
**Base verifier report:** `b6bba481dc16b62c39f609485dc1635c0e4609e7`
**Repair implementation:** `0cdde220eca86a2edd619f9ccaddfd37b8972b26`
**Deployment:** <https://selfhost-observability-fit.sociobot.in/> (Azure Static Web App `sf-selfhost-observability-fit`, production)
**Verified:** 2026-08-28 UTC

## Repair completed

1. **Dynamic results contrast:** changed the light `--metric` token from `#7A7567` (the verifier measured 4.44:1) to `#746F61`. The rendered synthetic-result flow now has zero serious/critical axe findings on desktop and at 390 × 844.
2. **CLI capacity overflow:** profile construction now rejects non-finite or `u64`-unrepresentable retention calculations before a report, CSV, or plan can be serialized. The documented accepted maximum combination `--retention-days 3650 --growth 500 --headroom 500 --json` exits **2**, writes no stdout, and explains that the capacity estimate is unrepresentable. Normal valid values remain unchanged.
3. **390 px target size:** header/footer wordmarks, both copy controls, and footer links now have 44 px minimum hit heights (and the footer controls retain an 8 px gap). The live measured targets are 137.34 × 44, 60 × 44, 60 × 44, 137.34 × 44, 46.81 × 44, and 54.61 × 44 CSS px respectively.
4. **Update safety:** bumped the service-worker cache from `obsfit-field-guide-v1` to `obsfit-field-guide-v2`, so an existing cached app shell receives this UI repair on update.

## Regression coverage

- Rust unit test: unrepresentable growth is rejected before a `Report` exists.
- Rust integration test: the verifier's exact maximum CLI invocation exits 2, emits the actionable message, and emits no JSON.
- Playwright axe test loads the synthetic specimen and audits the rendered result state in both desktop Chromium and the 390 px project.
- Playwright 390 px test measures all six previously failing target classes and asserts width and height are each at least 44 CSS px.

## Verification evidence

Clean install and local quality gates:

```sh
npm ci                         # 0 vulnerabilities
npm run typecheck              # pass
npm test                       # pass: 3 Vitest, 4 Rust unit, 5 Rust integration,
                               # 1 doctest, 7 Playwright passed; 1 desktop-only skip
npm run build                  # pass; dist/site produced
cargo fmt --check              # pass
cargo clippy --all-targets -- -D warnings  # pass
cargo package --allow-dirty    # pass; package verification compiled
```

The production build is within the static budget: JS 9.63 kB raw / 4.03 kB gzip; CSS 15.99 kB raw / 4.37 kB gzip; the existing 92.3 kB WebP hero remains unchanged. A fresh `cargo install --path target/package/selfhost-observability-fit-0.1.0 --root <temp>` consumer install completed; its installed binary read NDJSON stdin and emitted `obsfit.report.v1` JSON with two records.

Live checks after `swa deploy dist/site --env production --app-name sf-selfhost-observability-fit --resource-group sociobot --swa-config-location dist/site`:

- SHA-256 values for `index.html`, the hashed JS and CSS, hero WebP, and `sw.js` exactly match `dist/site`; live CSS is `style-DZDlFIeB.css` and live service worker starts with `obsfit-field-guide-v2`.
- Desktop and 390 px Chromium checks passed title, `lang=en`, exactly one `<h1>`, one `<main>`, keyboard skip-link focus, synthetic result flow, zero console errors, and zero serious/critical axe violations. All runtime requests stayed same-origin.
- On live 390 px, a service-worker-controlled offline reload returned HTTP 200 with the expected title and one `<h1>`; its cache key was `obsfit-field-guide-v2`.
- Live privacy check found 0 local-storage entries, 0 session-storage entries, and 0 cookies. Static/runtime inspection found no analytics, telemetry, upload, remote-font, or third-party runtime requests.
- Response policy is present: CSP confines default/script/style/connect to `'self'`, `object-src 'none'`, `base-uri 'self'`, `frame-ancestors 'none'`, with `nosniff`, `no-referrer`, restrictive Permissions-Policy, and HSTS. Hashed JS/CSS/WebP are immutable for one year; `sw.js` is `no-cache`.
- Lighthouse 13.4.1, mobile against the live URL: **100 performance / 100 accessibility / 100 best practices / 100 SEO**; FCP 851 ms, LCP 1369 ms, TBT 27 ms, CLS 0.

## Publish/deploy notes

The artifact remains a Rust single-binary CLI plus Vite static landing/docs site; no registry publish was attempted. The crate is ready for the factory to publish with `cargo package` (or the verified package-consumer command above).

## Known gaps / next steps

No known release-blocking gaps remain. Continue to treat capacity reports as heuristic planning aids and validate a selected stack with the documented seven-day synthetic replay.
