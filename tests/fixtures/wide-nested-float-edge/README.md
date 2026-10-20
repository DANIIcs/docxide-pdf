# wide-nested-float-edge

Synthetic DOCX and immutable Microsoft Word PDF pairs; linked settings select compatibility mode. No production documents or fonts. Existing thresholds and baselines are unchanged.

Run `DOCXIDE_GROUP=wide-nested-float-edge cargo test --test visual_comparison -- --nocapture`.

Eight full-overflow cases vary compatibility14/15 and host/child padding; four narrow modern controls verify unchanged positions. The new path requires a laid-out float whose width exceeds the full host cell, not merely its text area. Partially overflowing borderline cases retain their existing behavior.
