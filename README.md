# Observability Fit Check

Pick a plausible self-hosted observability stack from telemetry you already understand. `obsfit` reads a bounded, synthetic or redacted OTLP JSON sample, measures its signal mix and attribute cardinality, then estimates daily ingest, retained disk, and comparable resource envelopes for Grafana LGTM, SigNoz, OpenObserve, and Highlight.

It is a planning aid, not a benchmark or vendor ranking. Every estimate is labelled as heuristic and the report tells you what to validate during a one-week replay.

## Who it is for

Solo operators and small platform teams who want to rule stacks in or out before sending production telemetry or spending days on trial deployments.

## Install

Build the single Rust binary from a checkout:

```sh
cargo install --path .
obsfit --help
```

The factory can publish the ready-to-package crate after release review; workers do not publish registry packages.

## Usage

Export a **synthetic or redacted** OTLP JSON sample covering at least 10 minutes, then run:

```sh
obsfit sample/otlp.json --retention-days 14 --emit-dir ./fit-plan
```

NDJSON is accepted too, and `-` reads stdin:

```sh
telemetry-redactor export --format otlp-json | obsfit - --json > fit.json
```

The command exits `0` after a valid analysis, `2` for invalid arguments or unsafe input (empty, malformed, or over the size limit), and `1` for other I/O failures. It never sends telemetry or usage data anywhere.

```text
Usage: obsfit [OPTIONS] <INPUT>

Arguments:
  <INPUT>  OTLP JSON/NDJSON file, or - for stdin

Options:
      --retention-days <DAYS>  Retention window to model [default: 14]
      --growth <PERCENT>       Expected daily volume growth [default: 0]
      --headroom <PERCENT>     Capacity headroom [default: 30]
      --max-sample-mib <MIB>   Refuse larger samples [default: 50]
      --emit-dir <DIR>         Write report.json, budgets.csv, and Compose overlays
      --json                   Print only stable JSON to stdout
  -h, --help                   Print help
  -V, --version                Print version
```

`--emit-dir` writes:

- `report.json` — full machine-readable evidence, assumptions, and profiles.
- `budgets.csv` — daily and retention disk budgets for spreadsheets or CI.
- `compose.<stack>.yaml` — a resource overlay to merge with that stack’s upstream Compose file; it is intentionally not a replacement deployment.
- `README.md` — exact merge guidance and the assumptions attached to the files.

### Supported OTLP shapes

The analyzer walks standard OTLP/JSON `resourceSpans`, `resourceMetrics`, and `resourceLogs`, including their scope collections and nanosecond timestamps. It also accepts NDJSON containing multiple OTLP envelopes. Unknown fields are ignored. Protobuf and live endpoints are intentionally out of scope for v1; convert protobuf to JSON first so the sample remains inspectable and redaction can be verified.

### Reading the result

Storage is calculated from observed bytes per record, a signal-specific indexing/compression factor, daily growth, retention, replicas, and user-selected headroom. Resource profiles use transparent workload bands, not secret benchmark scores. Run the generated plan against synthetic replay traffic for seven days and compare actual disk growth; adjust the observed ratio if it differs by more than 25%.

## Website and local demo

The static field guide explains the model and includes a browser-only demo. Files stay in the browser and are not uploaded.

```sh
npm ci
npm run dev
npm test
npm run build:site   # writes dist/site/index.html
```

`npm run build` is an alias for the factory build and produces the same `dist/site` output.

## Develop the CLI

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo package --allow-dirty
```

The documented examples are covered by integration tests and sample fixtures. Rust 1.85 or newer is supported.

## Privacy and security

Use synthetic or redacted telemetry. Attribute values can contain credentials, customer identifiers, and request data; inspect the sample before analysis. The CLI is offline and stateless. The website has no analytics, cookies, remote fonts, third-party scripts, storage, upload endpoint, accounts, or payment.

## License

MIT. See [LICENSE](LICENSE).
