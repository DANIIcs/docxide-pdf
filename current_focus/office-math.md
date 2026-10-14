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
- Radicals: the surd is the first variant at least body + gap + rule tall.
  Inline, a taller surd shares its surplus above and below the body (the
  gap grows by half); in display the bar keeps the display gap and the
  surplus all goes below (display √(a/b): bar 17.55pt up at 12pt, the
  half-surplus rule says 18.13). Word lowers the surd at most
  RadicalVerticalGap (the text-style one, in display too) below its designed
  place, so over short contents the bar sits higher (√x: 10.0pt up at 12pt,
  not 9.45), and a surd that then no longer reaches below the body gives way
  to the next variant (√y, √g, √p take the 8.91pt surd). Probes `mp3_rad`
  (12, 24pt) and `mp7_rad` (10, 16, 36, 72pt; 14 bodies inline, 5 display):
  every variant agrees, bars within ±0.26pt in display; all 36 `mp5_delim`
  delimiter probes now agree.
- In display math a fraction never shrinks its parts: a nested a/b/c keeps
  12pt letters at 12pt (text-style shifts inside). Inline, the parts drop a
  script level (pendulum's π/2 at 11.5 of 16pt).
- `m:d` delimiters are centred on the math axis even at base size (Word draws
  `(x)` 0.5pt up at 12pt). With `m:grow` on (default) they take the first
  variant at least δ − 3/8 em tall, δ = TeX's axis-symmetric cover height
  2·max(asc − axis, desc + axis), and always one size past a delimiter inside
  them: all 36 probes (`mp5_delim`: x, y, b², x+y, inline and display
  fractions, nested fractions, delimiters and radicals) agree for a
  shortfall in [4.47, 4.58) pt at 12pt. A
  `m:sepChr` without a value draws no separator.
- A typed space in a math run is the spacing: nothing is added beside it
  (the zone ", " is comma + 0.22em space).
- Display n-ary operators take the first variant at least 1.5 ×
  DisplayOperatorMinHeight: all 20 operators probed take their 1.91–2.15em
  variant, never the ∫ family's 1.35em one (the bar lies in (1.35, 1.91] em).
  Inline operators keep the base glyph and put limits beside it whatever
  `limLoc` says.
- Matrices: columns an em apart; rows step like math-font lines (14.16pt at
  12pt for plain rows), taller ink pushing them apart; the grid is measured
  and centred on the axis by its ink (heading-to-row distances match Word to
  0.05pt). Display cells keep display style (a cell's a/b shifts 9.12/7.92).
- Display operators' limits drop from the operator's ink like a box's
  scripts (display ∫: 13.7pt up, 11.3pt down at 12pt); inline operators'
  limits sit like a letter's (inline ∑: 4.8 up, 2.8 down).
- Group characters (`m:groupChr`): a brace or arrow keeps its designed place
  (a ⏟ hangs below the baseline) unless the content comes within the
  stretch-stack gap: x+y+z's descenders push the brace 1.44pt down, a+b+c's
  leave it. With `m:vertJc` on the character's side (default top, so any
  pos="top" brace without vertJc) the content shrinks to a script-size limit
  of the character.
- `m:borderBox`: an overbar's gap and rule all round (border 1.94pt off 12pt
  content, 0.72pt rule).
- A display-math paragraph takes its own `m:oMathParaPr/m:jc` (centred by
  default) over the paragraph's `w:jc`: Word centres a `w:jc="both"` cell's
  equation.

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
- A zone too long for its line breaks after a top-level binary or relation
  operator, whatever `m:brkBin` says (with brkBin before or no mathPr alike:
  "… + 𝑎5𝑥5 +" / "𝑎6𝑥6 + …"; a broken minus isn't repeated). Probe
  `mp6_break`: our breaks match Word's on all three lines.

## Open

- Radicals inline over tall contents (b, H, 1, ℓ, x+1): Word's surd sits
  0.3–0.6pt lower than the half-surplus rule at every size from 10 to 72pt
  (an absolute offset, not a fraction of the em; the drop limit shows the
  same). Not found in the MATH constants; snapping is ±0.25pt.
- groupChr with the content as limit: Word's content sits 7pt under a ⏞
  (ours 1pt); a box-reaching-baseline clamp made the probe document worse.
- Accents over tall bases, eqArr alignment points (`&`), display-math wrap
  (`m:wrapIndent`): not yet measured. The parts of a broken inline zone keep
  the whole zone's height.
- `m:nor` text draws in the math font, upright; Word uses the run's font.
