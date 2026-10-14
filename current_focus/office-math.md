# Office Math (OMML) layout

Equations used to be flattened into text runs (`a/b`, `√(…)`, `(…)`), so
fractions, radicals, matrices and big operators drew wrong and lines kept the
text's height. Now each `m:oMath` zone is laid out as a box by `src/math/`,
driven by the math font's OpenType MATH table (Cambria Math, face 1 of
`fonts/Cambria.ttc`). Word's math engine is where that table comes from, so
its constants hold. The rules below are Word's, measured from its exports.

Evidence: Word online probes (python-docx documents with OMML injected,
exported with `tools/word_export.py`), the pendulum fixture's online
reference, and the external corpus's one-construct-per-document math test
documents (local print exports: 0.24pt position grid, otherwise the same
geometry). Online positions snap to 0.25pt, so single readings carry ±0.125pt.

## Pipeline

- `docx/math.rs`: OMML → `model::MathZone` tree (one zone per `m:oMath`;
  `display` when its parent is `m:oMathPara`), and `m:mathPr` →
  `model::MathSettings`. A zone becomes one `Run` with `math: Some(zone)`; its
  `text` is the linear form (extraction, fallback), `formula` the spoken /Alt.
- `pdf/fonts.rs::register_math_font`: before any font is embedded, every zone
  is laid out (`math::layout`) and the math font is embedded with the glyphs
  the boxes draw, by glyph id (`FontContext::extra_glyphs`; script alternates
  and stretched variants have no character). ToUnicode maps each id to the
  character it stands for. No font with a MATH table: zones draw their text.
- `pdf/layout.rs`: a zone is one `WordChunk` (`math: Some(box)`), drawn glyph
  by glyph with `Tm` inside the Formula marked content; rules go with the
  line's decorations.

## Rules

### Sizes and glyphs
- Script sizes: base × ScriptPercentScaleDown (73%), scriptscript × 60%,
  **floored to the half point**: 9→6.5/5, 10→7/6, 11→8/6.5, 12→8.5/7,
  13→9/7.5, 14→10/8, 16→11.5/9.5, 20→14.5/12.
- Script-level glyphs are the `ssty` alternates (1st for script, 2nd for
  scriptscript): widths match Word's to 0.01pt.
- Letters are drawn as Mathematical Alphanumeric Symbols. No `m:sty`: Latin
  and lowercase Greek italic (𝑇 U+1D447, 𝜋, 𝜗), digits and capital Greek
  upright; `b` → bold, `bi` → bold italic, `p` → as typed; `m:scr` picks the
  alphabet (Letterlike holes: ℝ, ℒ, ℎ…). A hyphen-minus draws as − (U+2212).

### Spacing
- TeX's table with thin 3/18, medium 4/18, thick 5/18 em: `a+b` 2.67pt each
  side of + at 12pt, `=` 3.33pt. A binary sign at the start, after an
  operator, relation, opening or punctuation is unary (no space).
- An italic glyph followed by an upright one keeps its italic correction
  (𝑇 then `=`: 4.44 + 0.35 = 4.79, Word 4.75).
- A function name is an operator: thin space before an ordinary argument,
  none before a delimiter (`sin x`, `sin(x)`).
- After a script: SpaceAfterScript.

### Structures
- Fractions: MATH fraction shifts and gaps; display numerator 9.05 / denominator
  7.95 at 12pt (Word 9.05 / 7.95); inline parts at script size, 7.0 / 6.0.
  No padding: the bar is exactly as wide as the wider part.
- Scripts: superscript at advance + italic correction, subscript at the
  advance; MATH shift rules.
- `m:d` delimiters are centred on the math axis even at base size (Word draws
  `(x)` 0.5pt up at 12pt), stretched to cover the content when `m:grow` is on
  (default).
- Display n-ary operators take the first variant at least 1.5 ×
  DisplayOperatorMinHeight: all 20 operators probed take their 1.91–2.15em
  variant, never the ∫ family's 1.35em one (the bar lies in (1.35, 1.91] em).
  Inline operators keep the base glyph and put limits beside it whatever
  `limLoc` says.
- Matrices: columns an em apart, rows a math line (Cambria Math hhea box)
  apart, centred on the axis.

### Line heights
- A line holding math is sized as if the zone were text in the math font's
  hhea box (Cambria Math: 0.95em above, 0.222em below; its win metrics are
  several ems and never used). A display-math paragraph's line comes out
  exactly so (12pt `x=1`: 11.4 + 2.67).
- Ink above that ascent less MathLeading (0.146em), and ink below the
  descent, stretch the line **unscaled** by the line spacing (OpenType MATH's
  MathLeading rule; Calibri and Arial, 12 and 24pt, single and 1.5: an inline
  a/b line grows 1.75–2.0pt above and 2.5–3.5pt below either way). Exact
  spacing never stretches.

### Line breaking
- An inline zone is one box. It is glued to text it touches without a space:
  pendulum's "cambiando la lunghezza 𝑙 del filo" moves "𝑙 del" to the next
  line together (nothing may break before "del").

## Open

- Radicals: the standard placement (gap grows by half the surd's surplus)
  matches Word for b, H and fractions, but over short contents (x, a) Word's
  overbar sits 0.6pt (12pt) to 1.4pt (24pt) higher. Probe `mp3_rad_*`.
- Display ∫ limits, groupChr braces (Word places the assembly at the content
  baseline), accents over tall bases, borderBox padding, eqArr alignment
  points (`&`), `m:brkBin` line breaking inside a zone: not yet measured.
- `m:nor` text draws in the math font, upright; Word uses the run's font.
