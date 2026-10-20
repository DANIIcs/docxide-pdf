# Rectangular contour clearance

20 synthetic DOCX/immutable Microsoft Word PDF pairs vary Tight/Through, Square controls, linked compatibility14/15,12/14pt and single/1.5 line spacing. The rectangle spans the column; paragraphs have no incoming gap. Natural-font line-step clearance applies only to eligible rectangular contour wrapping; Square keeps its existing path.

This branch depends on PR118 (including PR110): leading picture anchors must already keep their correct paragraph layout. Each intended fixture is checked separately for page count and the After-anchor page/Y, because image-dominated raster scores can hide misplaced text.

No production documents/fonts or threshold/existing-baseline changes. Run `DOCXIDE_GROUP=rect-contour-clearance cargo test --test visual_comparison -- --nocapture`.
