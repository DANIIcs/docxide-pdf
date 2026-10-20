# AutoFit run boundaries

Sixteen synthetic DOCX/immutable Microsoft Word PDF pairs vary narrow/wide saved grids, zero/positive right paragraph indents, whole/color-split email runs, and AutoFit/fixed layout. Settings.xml is explicitly linked in compatibility mode15. Horizontal padding is zero to isolate this change from the separate table-edge alignment behavior.

The zero-indent AutoFit split-run case must have the same unbreakable minimum as the whole-run case. Whitespace boundaries and fixed layout retain their existing rules. Small existing sizing differences in some controls are not claimed fixed.

No production documents, fonts, thresholds or existing baselines are included/changed. Run `DOCXIDE_GROUP=autofit-run-boundaries cargo test --test visual_comparison -- --nocapture`.
