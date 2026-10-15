# Empty margin frames before an introductory table

All documents are freshly generated public controls, with no customer data. Explicit paragraph defaults and zero table cell margins isolate the vertical frame behavior.

In compatibility mode 15, one or two empty margin-anchored auto-height wrapping paragraphs reserve 12 or 24pt before the preceding full-width inline table. Their section-break mark contributes no extra frame line. The corresponding mode-14 layouts do not reserve this area.

The matrix covers no frame, one empty frame paragraph, two empty paragraphs and ASCII-space-only paragraphs in both compatibility modes. Three mode-15 reproducers improve to 100% ink F1. All mode-14 cases and the no-frame case are byte-identical.

Four additional negative controls are byte-identical: off-page frame, hidden paragraph marks, visible body text before the frame, and a long prefix whose anchor could be on a later page. The latter controls document this fix's conservative first-page scope; they do not claim to fix all later-page frame behavior.
