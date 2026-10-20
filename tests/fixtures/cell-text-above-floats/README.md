# cell-text-above-floats

Synthetic DOCX and immutable Microsoft Word PDF pairs. No production documents or fonts. Existing thresholds and baselines are unchanged.

Run `DOCXIDE_GROUP=cell-text-above-floats cargo test --test visual_comparison -- --nocapture`.

Four cases vary one/two positive-offset floating tables and30/60pt top gaps beneath a caps/letter-spaced heading. A minimum-height outer row isolates positions from row-height sizing. All four candidate pages match Word at100% tolerant ink F1. Additional local diagnostics include long headings and insufficient top gaps: these do not activate the new path. A unit test verifies width, height and manual-line-break guards.
