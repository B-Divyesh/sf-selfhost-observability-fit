# Independent verification — FAIL

**Work order:** `selfhost-observability-fit-verify-1`
**Candidate:** `28481be2d49449488ac9507107684c9692d0fd54` (`main`)
**Live URL:** <https://selfhost-observability-fit.sociobot.in/>
**Verified:** 2026-08-28 UTC from a clean checkout at the candidate SHA

## Verdict

**FAIL.** The normal workflow, package, deployment, privacy posture, and most
quality gates work, but two high-severity defects violate the acceptance
contract:

1. The rendered results have an axe **serious** WCAG AA contrast failure on
   desktop and at the required 390 px mobile viewport. This contradicts the
   required zero serious/critical findings and the 4.5:1 text-contrast floor.
2. The CLI accepts its documented maximum numeric values but creates an
   unusable capacity plan (`null` budgets and a max-`u64` disk volume). A
   sizing utility must reject an unrepresentable estimate or return finite,
   actionable values.

## Defects

### High — dynamic results fail WCAG AA contrast

- **Reproduction:** Load the synthetic specimen, select **Inspect sample**,
  then run axe-core 4.10.3 against the live page.
- **Observed:** `color-contrast` / serious on `.metric` (`Metric points`) in
  the result legend: foreground `#7a7567` on `#fffbee` is **4.44:1** at 11 px;
  axe requires 4.5:1. The same violation occurs at desktop and 390 × 844.
- **Impact:** Result content has a release-blocking accessibility failure.
  The repository Playwright axe test only audits the initial page, so it did
  not cover the dynamically rendered result state.

### High — allowed maximum CLI values overflow capacity estimates

- **Reproduction:**

  ```sh
  obsfit tests/fixtures/mixed-otlp.json \
    --retention-days 3650 --growth 500 --headroom 500 --json
  ```

- **Observed:** Exit status is 0, but every profile has
  `retention_budget_gib: null` and `volume_gib: 18446744073709551615`.
  Human output likewise reports `18446744073709551615GiB`. The argument
  parser explicitly accepts all three values.
- **Impact:** Valid CLI input can silently produce a non-actionable plan.
  Detect non-finite/overflowed estimates before serialization and return the
  documented invalid-input exit code (or constrain the accepted range).

### Medium — several mobile touch targets miss the documented 44 px minimum

At 390 px, the visible header/footer wordmarks measure 137 × 34 px, each
copy button measures 60 × 36 px, and the footer Source/Contact links measure
47 × 15 px and 55 × 15 px. The labelled file control is intentionally 1 px
because its large label is the target and is not counted as a defect. Increase
the interactive hit areas for the other controls.

## Evidence collected

### Clean install, tests, build, and packaging

- `npm ci`: completed; `npm audit` reported 0 vulnerabilities.
- `npm run typecheck`: passed.
- `npm test`: passed — 3 Vitest browser-analyzer tests, 3 Rust unit tests,
  4 Rust integration tests, 1 doctest, and 6 Playwright tests across desktop
  Chromium and a 390 × 844 viewport.
- `npm run build`: passed and produced `dist/site`.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo package --allow-dirty`: passed. Package verification compiled the
  crate successfully.
- The generated `.crate` was extracted and installed into a fresh temporary
  consumer root with `cargo install --path`; the installed `obsfit 0.1.0`
  read NDJSON from stdin and emitted `obsfit.report.v1` JSON.

### CLI end-to-end coverage

- A normal mixed trace/log/metric fixture produced four profiles and all
  expected plan files: `report.json`, `budgets.csv`, plan `README.md`, and
  four Compose overlays.
- Stdin NDJSON completed successfully with two log records.
- Invalid argument boundaries `--retention-days 0`, `3651`, `--growth 500.1`,
  `--headroom -1`, and `--max-sample-mib 0` each exited 2.
- A missing input file exited 1. Stdin exceeding `--max-sample-mib 1` exited
  2 with the safety-limit message.
- The normal lower boundary (`--retention-days 1 --growth 0 --headroom 0`)
  worked. The accepted combined upper boundary is the high-severity failure
  described above.

### Live deployment, browser, and PWA checks

- The live document and all deployed candidate assets are byte-identical to
  the fresh production build: `index.html`, hashed JS, hashed CSS, hero WebP,
  and `sw.js` have matching SHA-256 values. The live root returned HTTP 200.
- The live page has a title, `lang="en"`, one `h1`, and a `main` landmark.
  Desktop and 390 px runs had no console errors, page errors, horizontal
  overflow, or outbound runtime requests. A Tab press reaches a visibly
  focused skip link (3 px ochre outline); normal result, invalid headroom
  (focus returns to the number input), malformed JSON, synthetic recovery,
  and browser 5 MB-limit error paths were exercised.
- Reduced-motion mode sets result animation duration to `1e-06s` and document
  scroll behavior to `auto`.
- Service-worker registration became controlling after reload. After an online
  cache pass, offline reload at 390 px returned HTTP 200 with the correct
  title and one `h1`. The shipped worker uses a versioned cache plus
  `skipWaiting()` and `clients.claim()` for updates.
- Privacy inspection found no runtime analytics, upload, storage, remote font,
  or third-party script/request. The static scan found no browser network or
  storage API beyond same-origin service-worker caching; the Rust dependency
  tree contains only CLI/serialization dependencies.

### Response policies, cache policy, and budget

- Live CSP is `default-src 'self'`; it restricts `connect-src` to self and
  also sets `object-src 'none'`, `base-uri 'self'`, and
  `frame-ancestors 'none'`. `X-Content-Type-Options: nosniff`,
  `Referrer-Policy: no-referrer`, a restrictive Permissions-Policy, and HSTS
  were present.
- Hashed JS/CSS and the WebP have `Cache-Control: public, max-age=31536000,
  immutable`; `sw.js` has `Cache-Control: no-cache`.
- Production payloads are within budget: JS 9,626 B raw / 4,030 B gzip; CSS
  15,910 B raw / 4,360 B gzip; hero image 92,314 B; no font payload.
- Independent Lighthouse 13.4.1 mobile run: Performance 98, Accessibility
  100, Best Practices 100, SEO 100; FCP 0.9 s, LCP 1.4 s, TBT 160 ms, CLS 0.
  Lighthouse loaded only the initial state; the separate axe run above covers
  the dynamic result state and is authoritative for the reported defect.

## Retest criteria

1. Fix the metric legend contrast and add an automated axe assertion after a
   sample result is rendered at both viewports.
2. Reject or safely represent any non-finite/overflowed growth-retention
   calculation, with a non-zero invalid-input exit code and test coverage of
   the accepted maximum values.
3. Expand the listed mobile hit areas to at least 44 × 44 CSS px.
4. Re-run the complete verification suite and live byte/headers comparison.
