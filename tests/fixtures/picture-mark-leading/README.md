# picture-mark-leading

Synthetic DOCX inputs with immutable Microsoft Word PDF references. No private documents.

The group/case layout is required by `tests/common::discover_fixtures`; case names are at most 16 characters for readable, stable case identifiers.

| Case | Control parameters |
|---|---|
| `c14-l480` | `picture-mark-leading-compat14-line480` |
| `c15-l480` | `picture-mark-leading-compat15-line480` |
| `s22-l360` | `picture-mark-leading-mark22-line360-text0` |
| `s28-l480` | `picture-mark-leading-mark28-line480-text0` |
| `same-l-l480` | `picture-mark-leading-same-line-line480` |
| `three-ls-l480` | `picture-mark-leading-three-lines-line480` |

Run from the repository root with the Word-compatible fonts and `mutool` available:

```sh
DOCXIDE_GROUP=picture-mark-leading cargo test --test visual_comparison --test page_geometry --test text_boundary -- --nocapture
```

Check that all cases appear in the output and `tests/output/latest_scores.json`; an empty test run is not validation. Do not regenerate references with the renderer under test.

## Validation

6 cases were actually discovered and measured with mutool 1.23.0. All reference/generated page counts agree; no case was skipped. Minimum SSIM: 0.5925. 271 library tests passed; the separate system-font-dependent `missing_fonts_resolve_as_word_does` test was excluded.

The new cases have explicit visual baselines in `tests/baselines.json`; existing baseline entries are unchanged. The original main text-boundary parser does not recognize MuPDF 1.23 character nodes; its zero-line scores are not accepted as baselines. Text lines are separately audited with the character-node parser fix. Fonts and renderer versions affect visual scores, so use the same Word-compatible fonts when reproducing them.

### Raster interpolation is a separate known limitation

Five image-heavy controls have SSIM below the global 0.75 display threshold (minimum 0.5925). The low-resolution source PNG is enlarged; Word preserves hard pixel edges whereas current main interpolates them. This patch does not change image decoding or interpolation. No global threshold was lowered.

Compared with exact main fe7b8bea, the legacy following-text error drops from 6.186–32.043 pt to at most 0.164 pt; image bounds agree with Word within 0.020 pt. Ink-F1 improves from 49–61% to about 99.80% in the changed controls. The modern compatibility-15 control is byte-identical to main. The recorded visual baselines retain, rather than conceal, the interpolation limitation.
