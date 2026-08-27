# Handoff — Observability Fit Check v0.1.0

## What shipped

- A publish-ready Rust `obsfit` binary and typed library. It accepts bounded OTLP JSON, NDJSON, files, or stdin; handles traces, logs, and five OTLP metric point shapes; measures time windows and label-set cardinality; and exits predictably for scripting.
- Transparent heuristic reports in human-readable or stable `obsfit.report.v1` JSON. Estimates cover raw daily ingest, query-load band, growth, headroom, retained disk, CPU, and memory for Grafana LGTM, Highlight, OpenObserve, and SigNoz. The tool provides fit explanations, never a leaderboard.
- `--emit-dir` capacity plans with `report.json`, `budgets.csv`, a plan README, and resource-only Compose overlays to merge with each stack's upstream project.
- A Vite/vanilla TypeScript landing and documentation site in the required botanical field-guide direction. Its browser demo reads ≤5 MB OTLP samples locally, with synthetic, empty, parsing-error, offline, loading, results, JSON-download, keyboard, reduced-motion, light, dark, and 390 px mobile treatments.
- Original `telemetry-herbarium.webp` hero art generated with `/opt/fleet/lib/gen-image.sh`, visually inspected, and optimized to 92,314 bytes. Exact provenance is in `site/public/telemetry-herbarium.provenance.json` and the full system is documented in `.factory/design.md`.
- No runtime CDN, remote font, analytics, tracking, account, upload, storage, or payment. `/privacy` and `/terms` are not required because the product stores no user data and takes no payment; the local-only privacy behavior is explicit in the UI and README.
- MIT license, changelog, package metadata, offline shell cache, static security/cache headers, README run/test/deploy instructions, and a checked-in lockfile.

## Run and verify

```sh
npm ci
npm run typecheck
npm test
npm run build:site        # output: dist/site/index.html

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo package             # verified publish-ready crate; do not publish here
cargo run -- tests/fixtures/mixed-otlp.json --emit-dir /tmp/fit-plan
```

Verified on 2026-08-27:

- `npm test`: 3 browser-analyzer unit tests, 8 Rust tests/doctests, and 6 Playwright tests passed across desktop Chromium and a 390 × 844 viewport.
- Axe integration: 0 serious or critical violations on desktop and mobile.
- Factory `verify-url.sh`: HTTP 200, title present, `lang="en"`, exactly one `<h1>`, `<main>` present, 0 images missing alt, 0 unlabeled buttons, and 0 console/page errors.
- Lighthouse 12.8.2 mobile: Performance **100**, Accessibility **100**, Best Practices **100**, SEO **100**; LCP **1.505 s**, FCP **0.905 s**, TBT **0 ms**, CLS **0**.
- Production payload: initial JS **9.63 KB** raw / **4.03 KB** gzip; CSS **15.91 KB** raw / **4.36 KB** gzip; hero WebP **92.31 KB**. No font payload.
- `npm audit`: 0 vulnerabilities.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and verified `cargo package` pass.

## Known gaps and next steps

- Storage/index factors are deliberately documented heuristics. The 25% success target cannot be proven without a real seven-day replay; use synthetic/redacted traffic, compare daily disk growth, and calibrate when variance exceeds 25%.
- Compose files are resource overlays, not cloned vendor deployments. Upstream service names and minimums can change; inspect `docker compose ... config` and upstream release notes before every trial.
- Browser analysis is a lightweight preview capped at 5 MB. Use the 50 MiB-default CLI for decisions and raise its bound only after confirming the sample is safe and redacted.
- OTLP protobuf, live collector endpoints, session replay payloads, and benchmark rankings remain intentional non-goals for v1.
