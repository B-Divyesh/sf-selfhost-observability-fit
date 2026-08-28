# Independent verification 3 — PASS

**Work order:** `selfhost-observability-fit-verify-3`
**Candidate tested:** `3ab875841df3414ca0a4ef9072ac7ca1c43b3210` (`main`)
**Live URL:** <https://selfhost-observability-fit.sociobot.in/>
**Verified:** 2026-08-28 UTC from a clean checkout at the candidate SHA

## Verdict

**PASS.** The candidate fulfills the researched job: a bounded local OTLP JSON/NDJSON sizing utility with transparent comparable capacity profiles and local Compose-plan files. The fresh production build is byte-identical to the live deployment. Local gates, CLI/package checks, live browser, privacy, accessibility, PWA, policy, and budget checks passed.

## Defects by severity

- **Critical:** none.
- **High:** none.
- **Medium:** none.
- **Low:** none.

The two earlier blockers were retested. The combined accepted capacity maximum now exits 2 without JSON and reports an unrepresentable estimate. The former dark profile contrast failure has zero serious/critical axe findings at both desktop and 390 px mobile in light and dark modes.

## Clean checkout and gates

The worktree began clean at the candidate SHA. All passed:

```sh
npm ci                              # 0 vulnerabilities
npm run typecheck
cargo fmt --check
cargo clippy --all-targets -- -D warnings
npm test
npm run build                       # writes dist/site
cargo package --allow-dirty
```

`npm test` ran browser-analyzer unit tests, Rust unit/integration tests and the documented-example doctest, plus Playwright desktop and 390 x 844 projects. The exact production build is 9,626 B JS (4,040 B gzip), 16,010 B CSS (4,372 B gzip), one 92,314 B original WebP, and no font payload: within all static budgets.

## CLI and package end-to-end

- A release build analyzed `tests/fixtures/mixed-otlp.json` with 14-day retention, 5% growth, 30% headroom, `--json`, and `--emit-dir`. The stable `obsfit.report.v1` report had 4 records and 4 finite profiles; the emitted plan had `report.json`, `budgets.csv`, `README.md`, and four Compose files.
- Two OTLP log envelopes through stdin as NDJSON produced a four-profile, 2-record report.
- Independent boundary probes covered retention `0`/`3651`, growth `500.1`/`NaN`, headroom `-1`, max sample `0`, combined maxima (`3650`, `500`, `500`), empty/malformed input, missing input, and stdin exceeding 1 MiB. All returned documented nonzero statuses: 2 for invalid/unsafe input, 1 for the missing file.
- The packaged crate was extracted to a clean consumer root and installed with `cargo install --path`; installed `obsfit 0.1.0` analyzed the NDJSON stdin case to `obsfit.report.v1` with four profiles.

## Live deployment, browser, accessibility, and privacy

SHA-256 comparisons show live `index.html`, `assets/index-Bb9Ko5gz.js`, `assets/style-JbBA02Qn.css`, `telemetry-herbarium.webp`, and `sw.js` match the fresh candidate build byte-for-byte. Root response was HTTP 200.

Independent Playwright audits ran 1440 x 900 and 390 x 844 in light and dark schemes. Each had `lang=en`, one title, one `h1`, one `main`, no horizontal overflow, and a first-Tab visible skip link with solid 3 px focus. Normal synthetic analysis rendered all four shapes. Empty submit focused the file input; invalid 201% headroom focused its input; malformed JSON and a >5 MB browser sample gave actionable errors and recovered through the synthetic sample. There were zero console/page errors, cookies, local/session storage, or cross-origin runtime requests. Axe-core 4.10.2 reported zero serious or critical violations after results rendered. Inspected mobile wordmarks, copy buttons, and footer links were at least 44 px high.

At 390 px reduced motion used a `1e-06s` result transition and `auto` scroll. After an online cache pass, the controlling service worker supported a successful offline reload with the expected title and one `h1`. Static and request inspection found no analytics, remote fonts, third-party scripts, uploads, browser storage, or telemetry.

## Policies and performance

Responses have self-only CSP for default/script/style/connect, with `object-src 'none'`, `base-uri 'self'`, `frame-ancestors 'none'`, HSTS, `nosniff`, `no-referrer`, and restrictive Permissions-Policy. Hashed JS/CSS/WebP are `public, max-age=31536000, immutable`; byte-identical `sw.js` is `Cache-Control: no-cache`.

Lighthouse 13.4.1 mobile against production: **100 Performance, 100 Accessibility, 100 Best Practices, 100 SEO**; FCP 0.9 s, LCP 1.4 s, TBT 30 ms, CLS 0.

## Reproduction

```sh
npm ci && npm test && npm run build
cargo fmt --check && cargo clippy --all-targets -- -D warnings
cargo package --allow-dirty
cargo run -- tests/fixtures/mixed-otlp.json --retention-days 14 --emit-dir ./fit-plan --json
```

No known release-blocking gaps remain. Use only synthetic or redacted samples and validate a selected profile with the documented seven-day replay.
