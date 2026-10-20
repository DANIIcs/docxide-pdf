# negative-x-table

Synthetic DOCX inputs with immutable Microsoft Word PDF references. No private documents.

The group/case layout is required by `tests/common::discover_fixtures`; case names are at most 16 characters for readable, stable case identifiers.

| Case | Control parameters |
|---|---|
| `x-142y-33n0` | `negative-x-nested-x-142-y-33-w4820-n0` |
| `x-142y-33n4` | `negative-x-nested-x-142-y-33-w4820-n4` |
| `x-142y1n4` | `negative-x-nested-x-142-y1-w4820-n4` |
| `x0y-33n4` | `negative-x-nested-x0-y-33-w4820-n4` |
| `x142y-33n0` | `negative-x-nested-x142-y-33-w4820-n0` |

Run from the repository root with the Word-compatible fonts and `mutool` available:

```sh
DOCXIDE_GROUP=negative-x-table cargo test --test visual_comparison --test page_geometry --test text_boundary -- --nocapture
```

Check that all cases appear in the output and `tests/output/latest_scores.json`; an empty test run is not validation. Do not regenerate references with the renderer under test.

## Validation

5 cases were actually discovered and measured with mutool 1.23.0. All reference/generated page counts agree; no case was skipped. Minimum SSIM: 0.6833. 271 library tests passed; the separate system-font-dependent `missing_fonts_resolve_as_word_does` test was excluded.

The new cases have explicit visual baselines in `tests/baselines.json`; existing baseline entries are unchanged. The original main text-boundary parser does not recognize MuPDF 1.23 character nodes; its zero-line scores are not accepted as baselines. Text lines are separately audited with the character-node parser fix. Fonts and renderer versions affect visual scores, so use the same Word-compatible fonts when reproducing them.

### Positive-X control remains outside this fix

`x142y-33n0` retains a pre-existing SSIM of 0.6833 and is byte-identical to parent PR122. The patch only extends clearance to nonpositive X. It does not change positive-X positioning, rasterization or global test thresholds. In the changed negative-X case, Ink-F1 improves from 49.6765% to 100%; the following-text Y error changes from -16.278 pt to -0.180 pt. The other four controls are byte-identical to the parent.

This branch includes PR122 as a dependency.
