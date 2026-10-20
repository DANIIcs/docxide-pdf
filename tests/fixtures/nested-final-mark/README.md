# nested-final-mark

Synthetic DOCX inputs with immutable Microsoft Word PDF references. No private documents.

The group/case layout is required by `tests/common::discover_fixtures`; case names are at most 16 characters for readable, stable case identifiers.

| Case | Control parameters |
|---|---|
| `m-y-33-n1` | `nested-final-mark-modern-y-33-n1` |
| `m-y363-n2` | `nested-final-mark-modern-y363-n2` |
| `sp1-y1` | `nested-final-mark-spaces1-y1` |
| `sp10-y363` | `nested-final-mark-spaces10-y363` |
| `y1-h4-n1` | `nested-final-mark-w5100-y1-h400-n1` |
| `y1-h4-n2` | `nested-final-mark-w5100-y1-h400-n2` |
| `y363-h4-n2` | `nested-final-mark-w5100-y363-h400-n2` |
| `y363-h8-n1` | `nested-final-mark-w5100-y363-h800-n1` |

Run from the repository root with the Word-compatible fonts and `mutool` available:

```sh
DOCXIDE_GROUP=nested-final-mark cargo test --test visual_comparison --test page_geometry --test text_boundary -- --nocapture
```

Check that all cases appear in the output and `tests/output/latest_scores.json`; an empty test run is not validation. Do not regenerate references with the renderer under test.

## Validation

8 cases were actually discovered and measured with mutool 1.23.0. All reference/generated page counts agree; no case was skipped. Minimum SSIM: 0.9354. 271 library tests passed; the separate system-font-dependent `missing_fonts_resolve_as_word_does` test was excluded.

The new cases have explicit visual baselines in `tests/baselines.json`; existing baseline entries are unchanged. The original main text-boundary parser does not recognize MuPDF 1.23 character nodes; its zero-line scores are not accepted as baselines. Text lines are separately audited with the character-node parser fix. Fonts and renderer versions affect visual scores, so use the same Word-compatible fonts when reproducing them.
