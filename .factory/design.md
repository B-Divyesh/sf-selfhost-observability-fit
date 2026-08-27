# Visual thesis — botanical field guide

## Direction and rationale

Observability Fit Check is presented as a **botanical field guide for telemetry habitats**. A field guide does not crown a “best” specimen; it records traits, conditions, and fit. That makes the metaphor useful rather than ornamental: OTLP signals are the specimen, cardinality is branching density, retention is the growing bed, and each stack profile is a habitat card with clearly stated tolerances.

The interface borrows the quiet authority of a pressed-plant folio: warm paper, inked rules, compact specimen labels, hand-numbered annotations, and one original cyanotype-like telemetry plant. It deliberately avoids generic dark dashboards, neon gradients, and faux terminal chrome.

## Tokens

Light treatment (the canonical field-guide page):

- `paper #F3EEDC` — warm specimen-sheet background.
- `paper-raised #FFFBEF` — elevated controls and result sheets.
- `ink #16261D` — primary copy, 13.4:1 on paper.
- `ink-muted #526157` — annotations, 5.7:1 on paper.
- `moss #275D43` — primary action and healthy fit; white text is 7.5:1.
- `fern #4D7A57` — secondary botanical marks.
- `ochre #9A5B18` — caution and heuristic labels.
- `rust #8A342B` — error and constrained fit.
- `rule #B8B19B` — non-text separators only.

Dark treatment follows the user’s system preference:

- `night-paper #17211B`, `night-raised #202C24`, `night-ink #F1EBD9`, `night-muted #B8C5B9`, `night-moss #9BC7A5`, `night-ochre #E6B56E`, `night-rust #F09A8E`, `night-rule #526157`.

No state is conveyed by color alone: every fit state has a word and symbol.

## Typography

- Display and prose: Georgia, `Times New Roman`, serif. Its field-note character is available locally with no font transfer.
- Data, commands, labels: `ui-monospace`, SFMono-Regular, Consolas, monospace. Tabular numerals make capacity comparisons scan cleanly.
- Scale: 14 / 16 / 20 / 28 / clamp(40–68) px; body never below 16 px. Reading measure is capped near 68 characters.

## Spacing and layout

An 8 px base rhythm: 4 px for hairline relationships, then 8, 16, 24, 32, 48, 64, and 96 px. The desktop page uses an offset 12-column specimen grid; the hero copy occupies seven columns and the illustration five. At 760 px, the folio becomes one column, comparison tables become labelled stacked entries, and decorative annotations are reduced. Interactive targets are at least 44 px with an 8 px minimum gap.

## Interaction grammar

- Primary action: a dark moss “Inspect sample” control, with a leaf/arrow mark and plain verb.
- Local sample drop zone: a dashed specimen tray that also exposes an ordinary labelled file input for keyboard and assistive technology.
- Results: revealed like a field-note sheet, ordered from workload evidence to capacity budget to vendor-neutral habitat cards.
- Stack rows disclose assumptions in place; no modal dialogs or hidden comparison criteria.
- Errors, empty input, offline state, and parsing progress stay adjacent to the input with a clear next action.

## Motion policy

Motion is restrained and physical: the result sheet rises 8 px and fades over 220 ms; measurement bars grow from their baseline over 260 ms; controls depress by 1 px. Nothing loops. With `prefers-reduced-motion: reduce`, transforms and transitions are removed and all state changes are immediate. Content hierarchy, borders, labels, and spacing preserve depth without motion.

## Asset plan and provenance

- `site/public/telemetry-herbarium.webp`: original raster hero generated for this product with the factory image deployment, then locally converted to WebP. Prompt: “Botanical field guide plate on warm uncoated paper: one imaginary telemetry plant growing from a small server rack root system; three distinct branches represented only through forms—rounded trace seedpods, tiny log leaves, circular metric berries—fine engraved ink linework with restrained moss green and muted ochre watercolor washes, subtle paper grain, scientific specimen composition, generous clear negative space, no lettering, no numbers, no logos, no UI, no border, no watermark.” Deployment and generation parameters are preserved beside the source during generation; the shipped WebP is an original project asset. Intended license: MIT with the repository.
- Small leaf, ruler, and signal marks are hand-authored in CSS or inline SVG and are MIT-licensed with the source.
- No stock art, icon library, third-party font, or CDN asset is used.

