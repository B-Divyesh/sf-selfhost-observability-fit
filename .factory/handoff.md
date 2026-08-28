# Verification handoff — FAIL

Candidate `28481be2d49449488ac9507107684c9692d0fd54` was independently
verified against <https://selfhost-observability-fit.sociobot.in/> on
2026-08-28 UTC. The live HTML, JS, CSS, hero asset, and service worker are
byte-identical to a fresh `npm run build` output.

**Status: FAIL — do not promote this candidate.**

High-severity blockers:

1. After analysis results render, axe-core reports a serious WCAG AA
   `color-contrast` violation: `.metric`/“Metric points” is `#7a7567` on
   `#fffbee` (4.44:1, below 4.5:1). It reproduces on desktop and 390 px mobile.
2. The accepted maximum CLI options (`--retention-days 3650 --growth 500
   --headroom 500`) exit 0 but emit `retention_budget_gib: null` and
   `volume_gib: 18446744073709551615`, so a valid request yields an unusable
   capacity plan.

Medium: mobile header/footer wordmarks, copy controls, and footer links do not
meet the documented 44 × 44 px touch-target minimum.

Everything else exercised passed: clean install; TypeScript, unit,
integration, doctest, Playwright, formatting, Clippy, production build, and
crate package checks; fresh consumer installation; normal/invalid/oversize CLI
flows; byte-identical deployment; normal/error/recovery browser flows; privacy
and outbound-request audit; visible keyboard focus; reduced motion; service
worker offline reload; security headers/caching; and bundle budget. Lighthouse
mobile measured 98 performance / 100 accessibility / 100 best practices / 100
SEO, but its initial-state audit does not supersede the dynamic axe failure.

Full commands, measurements, exact reproduction, and retest criteria are in
`.factory/verification.md`. After the blockers are fixed, run:

```sh
npm ci
npm run typecheck
npm test
npm run build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo package --allow-dirty
```
