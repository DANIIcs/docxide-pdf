# Paragraph indents beside a floating table

Six synthetic DOCX and immutable Microsoft Word PDF pairs. Four positive cases vary floating registration-table width (3000/4200 twips) and recipient right indent (0/240 twips); the left indent is5100twips. A long paragraph crosses the float boundary to exercise initial and per-line geometry. Two inline-table controls retain normal paragraph indents.

For table floats, intersect the margin-relative paragraph box with the available strip instead of adding the left indent to the table edge. Image float behavior is unchanged. No production documents, fonts, existing baseline values or thresholds are included or changed.

Run `DOCXIDE_GROUP=table-wrap-indent cargo test --test visual_comparison -- --nocapture`.
