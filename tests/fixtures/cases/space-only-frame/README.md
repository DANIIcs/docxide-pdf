# ASCII-space-only auto-height wrapping frame

A synthetic wrapping frame contains one ordinary ASCII space. Word gives it
no body-flow line, just as in the `empty-frame-control` fixture. The following
paragraph starts at 75.8055 pt from the page top in Word; the converter moves
it from 91.7609 pt before the fix to 75.6623 pt after it (0.143 pt error).
References were exported by Microsoft Word 16.0.17932.20190.

Only space-only auto-height wrapping frames are changed. Ordinary space-only
paragraphs, NBSP, explicit breaks/tabs, fields, drawings, borders, nonzero frame
heights and text-below frames keep their existing handling. Word experiments
also show differences for several of those controls; they need separate fixes.
No private corpus content or metadata is present.
