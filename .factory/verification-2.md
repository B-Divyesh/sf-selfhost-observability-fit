# Independent verification 2 — FAIL

**Work order:** `selfhost-observability-fit-verify-2`
**Candidate tested:** `3e5f51f2a116a0c1175ade3a7eebe83f9473268e` (`main`)
**Live URL:** <https://selfhost-observability-fit.sociobot.in/>
**Verified:** 2026-08-28 UTC from a clean checkout at the candidate SHA

## Verdict

**FAIL.** The CLI, package, production build, deployed-asset parity, normal
light-mode browser flows, privacy posture, PWA offline reload, and the two
previously reported repair areas pass. However, the declared system dark
treatment has five **serious** axe WCAG contrast violations at both required
desktop and 390 px mobile viewports. This violates the factory accessibility
contract (zero serious/critical axe findings and contrast in both themes).

## Defects

### High — dark-mode profile text is effectively invisible

**Reproduction:** Open the live URL with `prefers-color-scheme: dark` and run
axe-core 4.10.2, at 1440 × 900 or 390 × 844.

**Observed:** `color-contrast` is serious, with five nodes in the “Compare
operating shapes, not logos.” section:

- `#profiles-title`
- headings for Grafana LGTM, Highlight, OpenObserve, and SigNoz

Axe measures foreground `#17211b` on the section background `#101712`, a
1.1:1 ratio (large text still requires 3:1). The production stylesheet sets
`.profiles { color: var(--paper) }`; the dark theme changes `--paper` to
`#17211b` while `.profiles` remains `#101712`.

**Impact:** A system-dark user cannot read the main heading or the four
stack names in the comparison section. This is a release-blocking,
user-visible accessibility failure. The repository e2e axe assertions run in
the environment's default color scheme and did not cover dark mode.

**Fix direction:** Give the dark `.profiles` section an explicit high-contrast
foreground (for example the dark `--ink` token) and add separate light/dark
axe result-state tests at desktop and 390 px.

## Successful verification evidence

### Clean install, quality gates, and build

```sh
npm ci                                      # 0 vulnerabilities
npm run typecheck                           # pass
cargo fmt --check                          # pass
cargo clippy --all-targets -- -D warnings  # pass
npm test                                    # pass
npm run build                               # pass; dist/site produced
cargo package --allow-dirty                 # pass; package verification compiled
```

`npm test` passed 3 Vitest tests, 4 Rust unit tests, 5 Rust integration
tests, 1 doctest, and 7 Playwright tests (one desktop-only touch-target test
is intentionally skipped). The exact production output is 9,626 B JS
(4,040 B gzip), 15,993 B CSS (4,369 B gzip), and a 92,314 B WebP hero; all
are within the stated static budgets.

Mobile Lighthouse 13.4.1 against the live site, using Chromium with
`--headless=new --no-sandbox --disable-dev-shm-usage --disable-gpu`, scored
**100 performance / 100 accessibility / 100 best practices / 100 SEO**:
FCP 0.9 s, LCP 1.4 s, TBT 30 ms, CLS 0. (Lighthouse's default run did not
emulate dark mode; the focused axe evidence above is authoritative.)

### CLI and ready-to-publish package

- Normal mixed OTLP input with `--retention-days 14 --growth 5 --headroom 30
  --emit-dir <temp> --json` returned `obsfit.report.v1`, 4 records, four
  finite profiles, `report.json`, `budgets.csv`, four Compose overlays, and
  plan `README.md`.
- NDJSON on stdin returned two records and valid stable JSON.
- Lower boundary `--retention-days 1 --growth 0 --headroom 0` returned four
  finite profiles.
- Invalid values exited 2 with actionable errors: retention 0/3651, growth
  500.1, headroom -1, max sample 0, empty input, malformed JSON, a 2 MiB
  input constrained to 1 MiB, and the documented combined maximum
  (`3650/500/500`). The maximum returned no stdout and explained that the
  capacity estimate is unrepresentable. A missing file exited 1.
- `cargo package --allow-dirty` succeeded. A fresh `cargo install --path
  target/package/selfhost-observability-fit-0.1.0 --root <temp>` consumer
  install succeeded; its installed `obsfit 0.1.0` binary processed NDJSON
  stdin and returned `obsfit.report.v1`, two records, and four profiles.

### Deployment parity, policies, privacy, and PWA

- Fresh `npm run build` output is byte-identical to the live root document,
  hashed JS, hashed CSS, hero WebP, and `sw.js` (SHA-256 compared). The live
  asset names are `index-Bb9Ko5gz.js` and `style-DZDlFIeB.css`; this is direct
  evidence the deployment is the tested candidate build.
- Live headers include CSP restricted to `'self'` for default/script/style/
  connect, `object-src 'none'`, `base-uri 'self'`, `frame-ancestors 'none'`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`,
  restrictive Permissions-Policy, and HSTS. Hashed JS/CSS/WebP use
  `public, max-age=31536000, immutable`; `sw.js` is `no-cache`.
- The shipped worker is cache `obsfit-field-guide-v2`, calls `skipWaiting()`
  and `clients.claim()`, and removes old cache keys on activation. A live
  390 px controlled page reloaded offline with HTTP-equivalent success,
  expected title, and one `h1`.
- Browser request capture for the normal app flow found no outbound runtime
  requests. Fresh contexts had zero local-storage/session-storage entries and
  no cookies. Static inspection found no analytics or remote font/script;
  service-worker fetches are explicitly same-origin. The CLI dependency tree
  is clap, serde, and serde_json plus their transitive dependencies.

### Browser end-to-end checks

At both 1440 × 900 and 390 × 844 in **light mode**:

- title, `lang=en`, exactly one `h1`, and one `main` were present;
- the keyboard's first Tab reached a visibly focused skip link (3 px ochre
  outline); there was no horizontal overflow;
- empty submit returned focus to the file input; invalid headroom returned
  focus to the number input; malformed JSON displayed the recovery message;
  loading the synthetic specimen after each error rendered the workload and
  profiles successfully;
- console errors, page errors, outbound runtime requests, and serious/critical
  axe violations were all zero; all measured header/footer/copy targets were
  at least 44 px high;
- reduced-motion at 390 px set result transition duration to `1e-06s` and
  document scroll behavior to `auto`.

Fresh one-scan axe runs confirmed zero serious/critical findings for the
light initial and rendered-result states. The dark initial and rendered-result
states both have the high-severity defect above; it occurs on desktop and
mobile.

## Retest criteria

1. Repair the dark `.profiles` foreground contrast and add color-scheme
   coverage to the e2e axe checks.
2. Re-run the clean install, full test suite, production build, deployed byte
   comparison, and live desktop/390 px light-and-dark axe audit.
