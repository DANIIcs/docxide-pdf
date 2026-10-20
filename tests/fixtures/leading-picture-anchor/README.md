# leading-picture-anchor

Synthetic DOCX inputs with immutable Microsoft Word PDF references. No private documents.

The group/case layout is required by `tests/common::discover_fixtures`; case names are at most 16 characters for readable, stable case identifiers.

| Case | Control parameters |
|---|---|
| `l-p0-y80-b0` | `leading-anchor-prefix0-offset80-behind0` |
| `l-p0-y80-b1` | `leading-anchor-prefix0-offset80-behind1` |
| `l-p330-y80-b0` | `leading-anchor-prefix330-offset80-behind0` |
| `l-p330-y80-b1` | `leading-anchor-prefix330-offset80-behind1` |
| `s-p300-sq` | `split-picture-prefix300-square` |
| `s-p300-ti` | `split-picture-prefix300-tight` |
| `s-p330-sq` | `split-picture-prefix330-square` |
| `s-p330-ti` | `split-picture-prefix330-tight` |
| `t-p0-y80-b0` | `trailing-anchor-prefix0-offset80-behind0` |
| `t-p0-y80-b1` | `trailing-anchor-prefix0-offset80-behind1` |
| `t-p330-y80-b0` | `trailing-anchor-prefix330-offset80-behind0` |
| `t-p330-y80-b1` | `trailing-anchor-prefix330-offset80-behind1` |

Run from the repository root with the Word-compatible fonts and `mutool` available:

```sh
DOCXIDE_GROUP=leading-picture-anchor cargo test --test visual_comparison --test page_geometry --test text_boundary -- --nocapture
```

Check that all cases appear in the output and `tests/output/latest_scores.json`; an empty test run is not validation. Do not regenerate references with the renderer under test.

## Validation

12 cases were actually discovered and measured with mutool 1.23.0. All reference/generated page counts agree; no case was skipped. Minimum SSIM: 0.9823. 272 library tests passed; the separate system-font-dependent `missing_fonts_resolve_as_word_does` test was excluded.

The new cases have explicit visual baselines in `tests/baselines.json`; existing baseline entries are unchanged. The original main text-boundary parser does not recognize MuPDF 1.23 character nodes; its zero-line scores are not accepted as baselines. Text lines are separately audited with the character-node parser fix. Fonts and renderer versions affect visual scores, so use the same Word-compatible fonts when reproducing them.
