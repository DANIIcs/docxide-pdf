# nested-float-align

Synthetic DOCX and immutable Microsoft Word PDF pairs; linked settings select compatibility mode. No production documents or fonts. Existing thresholds and baselines are unchanged.

Run `DOCXIDE_GROUP=nested-float-align cargo test --test visual_comparison -- --nocapture`.

Eight modern-compatibility cases vary center/right positioning and host/child padding. Inline and explicit-left controls retain their old path. Additional legacy controls still expose a pre-existing host-table alignment discrepancy; this PR does not claim complete legacy table geometry.
