# First-page header when a continuous section's first paragraph moves

Three synthetic controls use different `FIRST HEADER` and `DEFAULT HEADER`
variants, titlePg and a continuous second section. Word references were
exported by Word 16.0.17932.20190.

- `cont-first-text-overflow`: first paragraph needs more room than remains,
  so page 2 is the new section's first page: FIRST / FIRST.
- `cont-first-empty-overflow`: its first empty paragraph moves to page 2:
  FIRST / FIRST. Before the fix the converter emits FIRST / DEFAULT.
- `cont-first-first-fits`: first paragraph fits on page 1; a later page break
  opens page 2 with the default header: FIRST / DEFAULT (unchanged).

The correction is scoped to single-column continuous sections beginning
with an ordinary paragraph, with no active float exclusion. Tables, lifted
frames and drawing paragraphs retain their existing handling. The pending
origin is consumed once, and a changed cursor/page/section does not reuse
the first-page variant. All document text/metadata is synthetic.
