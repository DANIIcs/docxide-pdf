# short-cell-widows

Ten synthetic DOCX/immutable Microsoft Word PDF pairs use linked settings.xml to select compatibility mode14 or15. A two/three-line table-cell paragraph has one/two lines of remaining space. Tests vary widowControl and explicit keepLines/keepNext. Legacy Word splits short paragraphs one line at a time; modern Word respects widow/keep properties. Longer-paragraph splitting retains its existing behavior and is outside this change.

No production documents, fonts, existing thresholds or baseline values are included or changed.

Run `DOCXIDE_GROUP=short-cell-widows cargo test --test visual_comparison -- --nocapture`.
