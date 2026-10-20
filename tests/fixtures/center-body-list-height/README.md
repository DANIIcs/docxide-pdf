# center-body-list-height

Four synthetic DOCX/immutable Microsoft Word PDF pairs compare textAlignment=center with baseline, with and without a leading space. Body paragraphs use Times New Roman14pt, Symbol list labels and expanded automatic line spacing. This change concerns body-list line/paragraph geometry; it does not claim complete mixed-run glyph alignment. Style inheritance and explicit reset are checked by a unit test.

No production documents, fonts, existing thresholds or baseline values are included or changed.

Run `DOCXIDE_GROUP=center-body-list-height cargo test --test visual_comparison -- --nocapture`.
