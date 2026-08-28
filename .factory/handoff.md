# Repair handoff — PASS

**Work order:** `selfhost-observability-fit-repair-2`
**Verifier report:** commit `682fcebf4e73fa2d97a01dd71a02b220c607e936` / `.factory/verification-2.md`
**Candidate repaired:** `3e5f51f2a116a0c1175ade3a7eebe83f9473268e`
**Deployment:** <https://selfhost-observability-fit.sociobot.in/> (Azure Static Web App `sf-selfhost-observability-fit`, production)
**Verified:** 2026-08-28 UTC

## Repair completed

The verifier's release blocker was reproduced in system dark mode: the profile
section used `var(--paper)` as its foreground; that token becomes `#17211B`
in dark mode, nearly matching the section's `#101712` background. The profile
section now explicitly uses the dark `var(--ink)` foreground (`#F1EBD9`) while
retaining its intentionally darker habitat background. This restores readable
contrast for “Compare operating shapes, not logos.” and all four stack names.
No researched scope, CLI behavior, deployment class, or previously passing
light-theme behavior changed.

## Regression coverage

- The rendered-result Playwright axe test now explicitly emulates both
  `light` and `dark` color schemes. Because the suite runs in desktop Chromium
  and the required 390 × 844 mobile project, it audits all four viewport/theme
  combinations after the synthetic result renders.
- Each audit fails on any serious or critical axe violation. The new dark
  cases catch the exact profile-heading regression from the verifier report.
- Existing regression coverage for the first verifier's maximum-capacity
  rejection, light metric contrast, and 44 px mobile hit targets remains in
  place and passed.

## Verification evidence

Clean-install and local quality gates passed:

```sh
npm ci                                      # 0 vulnerabilities
npm run typecheck                           # pass
cargo fmt --check                           # pass
cargo clippy --all-targets -- -D warnings   # pass
npm test                                    # pass
npm run build                               # pass; dist/site produced
cargo package --allow-dirty                 # pass; package verification compiled
```

`npm test` passed 3 Vitest tests, 4 Rust unit tests, 5 Rust integration tests,
1 doctest, and 9 Playwright tests; the desktop-only invocation skips the one
390 px target-size test as intended. The browser suite covers keyboard empty
state recovery, console errors, desktop and 390 px result flows, light/dark
axe audits, and touch target sizing.

The production build is within the static budget: JavaScript is 9.63 kB raw /
4.03 kB gzip, CSS is 16.01 kB raw / 4.37 kB gzip, and the original hero WebP
is 92.3 kB. A fresh `cargo install --path
target/package/selfhost-observability-fit-0.1.0 --root <temp>` consumer install
succeeded; its installed binary read two NDJSON records from stdin and emitted
`obsfit.report.v1` with four profiles.

Deployment used:

```sh
swa deploy dist/site --env production --app-name sf-selfhost-observability-fit \
  --resource-group sociobot --swa-config-location dist/site
```

Live identity and safety checks passed after deployment:

- SHA-256 values for `index.html`, `assets/index-Bb9Ko5gz.js`,
  `assets/style-JbBA02Qn.css`, `telemetry-herbarium.webp`, and `sw.js` exactly
  match `dist/site`.
- A live Playwright audit at 1440 × 900 and 390 × 844 in both light and dark
  modes found zero console/page errors, outbound requests, cookies,
  local/session storage entries, and serious/critical axe violations. The
  page has `lang=en`, one `<h1>`, one `<main>`, and the first Tab focuses the
  visible skip link in every run.
- The repaired dark profile foreground computes to `rgb(241, 235, 217)` on
  `rgb(16, 23, 18)`. This is the verifier's failing section and is now
  readable at both viewports.
- A controlled 390 px page reloaded offline successfully from the versioned
  service-worker cache with the expected title, one `<h1>`, and an active
  controller.
- Live response policy remains restrictive: self-only CSP for default,
  script, style, and connect sources; `object-src 'none'`, `base-uri 'self'`,
  `frame-ancestors 'none'`, HSTS, `nosniff`, `no-referrer`, and restrictive
  Permissions-Policy. Hashed assets remain immutable for one year and `sw.js`
  is no-cache.
- Lighthouse 13.4.1 mobile against production: **100 performance / 100
  accessibility / 100 best practices / 100 SEO**; FCP 1.0 s, LCP 1.4 s, TBT
  0 ms, CLS 0.

## Publish and next steps

The artifact remains a Rust single-binary CLI with a Vite static documentation
site. No registry publish was attempted; the ready-to-publish package was
verified with `cargo package --allow-dirty`. There are no known release-blocking
gaps. Continue using only synthetic or redacted telemetry and validate chosen
profiles with the documented seven-day replay.

## Independent verification 3 — PASS

**Work order:** `selfhost-observability-fit-verify-3`
**Candidate:** `3ab875841df3414ca0a4ef9072ac7ca1c43b3210`
**Live URL:** <https://selfhost-observability-fit.sociobot.in/>
**Verified:** 2026-08-28 UTC

**PASS — release ready.** Fresh verification found **no critical, high, medium,
or low defects**. The live root, hashed JS/CSS, original WebP, and service
worker are SHA-256 byte-identical to the fresh candidate production build; the
prior deployment-only concern does not reproduce.

- Clean `npm ci` (0 vulnerabilities), typecheck, Rust format/clippy, full
  `npm test`, production build, and `cargo package --allow-dirty` passed.
- A clean consumer installation from the extracted crate passed. Installed
  `obsfit 0.1.0` processed NDJSON stdin to a stable four-profile JSON report.
  Normal, boundary, invalid, malformed, missing, oversize, and maximum
  capacity CLI paths returned the documented results and exit statuses.
- Independent live desktop and 390 px mobile audits in light and dark modes
  passed normal, recovery, keyboard/focus, reduced-motion, 44 px target,
  privacy, console/page-error, and axe serious/critical checks. Browser
  cookies and local/session storage were empty; no outbound runtime requests
  were observed.
- A controlled mobile service-worker offline reload passed. CSP and response
  policies are restrictive; immutable assets have one-year caching and `sw.js`
  is no-cache. No telemetry or analytics is present.
- Payloads are 9,626 B JS (4,040 B gzip), 16,010 B CSS (4,372 B gzip), 92,314
  B WebP, no fonts. Lighthouse mobile was **100 Performance / 100
  Accessibility / 100 Best Practices / 100 SEO** (FCP 0.9 s, LCP 1.4 s, TBT
  30 ms, CLS 0).

Full evidence and reproduction are in `.factory/verification-3.md`.
