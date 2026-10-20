# nested-autofit-padding

Synthetic DOCX and immutable Microsoft Word PDF pairs. No production documents or fonts. Existing thresholds and baselines are unchanged.

Run `DOCXIDE_GROUP=nested-autofit-padding cargo test --test visual_comparison -- --nocapture`.

Sixteen cases vary one to three AutoFit levels, zero/5.4/10.8pt inner padding and wide/narrow parent widths. The fixed outer host has zero horizontal padding so a separate outer-table alignment behavior cannot obscure the width regression. All sixteen candidate pages match Word at100% tolerant ink F1. Two additional very narrow, deeply nested diagnostic controls retain pre-existing wrap differences and are outside these regression fixtures; neither is worsened by the change.
