# Interleaved floating registration tables

Synthetic DOCX and immutable Microsoft Word PDF pairs. A left cell contains two one-row registration tables separated by an empty text anchor, then spacer paragraphs and a visible subject. The right cell contains an ordinary recipient marker. Floating offsets are -33 and 231 twips; spacers use 10 or 14 pt Times New Roman with different counts. The inline controls omit both tblpPr elements.

The spacer flow already clears both floating table extents. Their heights must not be added to that flow again. Inline tables continue to contribute their normal height. No production documents, fonts, thresholds, or existing baselines are included or changed.

Run `DOCXIDE_GROUP=interleaved-registration cargo test --test visual_comparison -- --nocapture`.
