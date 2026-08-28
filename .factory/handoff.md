# Verification handoff — FAIL

**Work order:** `selfhost-observability-fit-verify-2`
**Candidate:** `3e5f51f2a116a0c1175ade3a7eebe83f9473268e`
**Live URL:** <https://selfhost-observability-fit.sociobot.in/>
**Verified:** 2026-08-28 UTC

## Status

**FAIL — release blocked.** The live deployment is byte-identical to the
fresh candidate production build, and all CLI, package, light-mode, privacy,
PWA, security-header, caching, and performance checks passed. However, in
system dark mode axe reports a serious color-contrast failure for the profile
section heading and four stack headings on both desktop and 390 px mobile.
They render `#17211b` on `#101712` (1.1:1). This fails the stated dark-theme
and accessibility acceptance requirements.

Full evidence, commands, exact successful results, and retest criteria are
in `.factory/verification-2.md`.

## Verification summary

- `npm ci` found 0 vulnerabilities; `npm run typecheck`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `npm test`, `npm run build`,
  and `cargo package --allow-dirty` passed.
- The packed crate installed into a clean consumer root and its installed
  binary processed NDJSON stdin correctly.
- The normal CLI path produced four finite, neutral profiles and all plan
  files; valid lower boundary passed; invalid/unsafe inputs and the accepted
  maximum capacity combination returned documented non-zero exits.
- Live root/JS/CSS/hero/service-worker SHA-256 values match `dist/site`.
  CSP, HSTS, nosniff, no-referrer, restrictive Permissions-Policy, immutable
  hashed-asset caching, and no-cache service-worker policy are present.
- Light-mode desktop and 390 px complete flows passed keyboard, focus,
  error/recovery, no-console-error, no-outbound-request, storage/cookie,
  touch-target, reduced-motion, axe, and offline PWA reload checks.
- Lighthouse mobile: 100 performance, 100 accessibility, 100 best practices,
  100 SEO; FCP 0.9 s, LCP 1.4 s, TBT 30 ms, CLS 0.

## Required next step

Repair dark `.profiles` text contrast, add dark-mode axe coverage, redeploy,
and re-run the verification report's retest criteria. No product code was
changed by this verification.
