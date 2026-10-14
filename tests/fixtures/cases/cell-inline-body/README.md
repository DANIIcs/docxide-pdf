# Inline JPEG in a mixed table-cell paragraph

Five synthetic controls (`cell-inline-body`, `cell-inline-header`,
`cell-inline-nested`, `cell-inline-tabbed`, `cell-inline-exact`) put a 72 × 36 pt JPEG between
text runs in a table cell. Word references were exported by Word
16.0.17932.20190. Cell margins are explicit, isolating the picture from
default table-style differences.

Before the fix no image XObject is emitted. After it one is emitted in
each fixture, the image reserves its line height, and the following body
paragraph is clear of the picture. Body/nested/tabbed following paragraph:
Word 114.8055 pt from page top, converter 91.7609 → 114.6906 pt.
Picture top differs by 0.02 pt. The tabbed control retains an existing
0.6 pt horizontal tab discrepancy; other picture X coordinates differ
by less than 0.001 pt. Ink-region F1 at 100 DPI with a 1 px tolerance is
100% for all five fixtures after the fix (34–62% before).

The exact-spacing control keeps its 18 pt line, baseline and clipping:
following paragraph Word 93.8055 pt, converter 93.6623 pt. The picture's
full PDF object extends above the row, but its visible portion is clipped
to the paragraph box, as in Word. It does not grow the table.

The JPEG is a generated colored rectangle; all text and metadata are synthetic.
