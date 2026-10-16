A synthetic 500pt page has a 160pt first-page-only header, a body paragraph,
and a table row with a 350pt minimum height. The row is taller than the first
page's body area but fits on the next page. Word places both `Minimum` and
`Second` on page 2, leaving `Before` on page 1.

`first-header-small-control` reduces the header to one short line;
`first-header-no-minimum-control` removes the row minimum. Both controls keep
all three markers on page 1. Reference PDFs were exported using Microsoft
Word on Windows. The inputs were generated from minimal OOXML and contain no
customer documents, certificates, or personal metadata.
