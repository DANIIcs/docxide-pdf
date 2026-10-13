//! Right-to-left ordering of laid-out lines: the Unicode bidi algorithm
//! (UAX #9) as Word applies it, measured on Word's online PDF export
//! (probes 2026-10-13: arabic_rice plus 7 probe documents).
//!
//! - Every run is an embedding: a `w:rtl` run at the lowest odd level at or
//!   above the paragraph's, any other run at the lowest even one. So a plain
//!   run in a `w:bidi` paragraph reads "A2 <Arabic> ." left to right (only
//!   right-aligned), and an rtl run's neutrals turn right to left even in an
//!   LTR paragraph (". <Arabic>" with the full stop on the left).
//! - Each line is resolved on its own, from its first character, looking
//!   ahead into the rest of the paragraph: a dash that starts a line sits at
//!   its left end (line-start context), one that ends a line takes the
//!   direction of the text after it. A manual break ends the lookahead.
//! - No bracket pairing (N0) and, in plain runs, no mirroring: Word draws
//!   "(قوس)" between Arabic words with its brackets unmirrored.
//! - In plain runs a neutral cluster between right-to-left letters is laid
//!   out left to right as one unit ("منه, يتناوله" keeps ", " after the
//!   comma's own word), except whitespace straight after a letter, which
//!   stays right to left. Hyphen-minus (ES) clusters reverse normally. A
//!   number next to a cluster is a unit of its own (" (5) " draws ") 5 (").
//!
//! Lines are reordered after line breaking. Chunks are stored in visual
//! order with their new offsets; `TextLine::draw_order` keeps the logical
//! order for drawing, so text extraction and tags read logically, and
//! right-to-left chunks place their glyphs from the right (`show_rtl`).

use std::collections::HashMap;

use crate::fonts::FontEntry;

use super::layout::{TextLine, WordChunk};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Class {
    L,
    R,
    AL,
    EN,
    ES,
    ET,
    AN,
    CS,
    NSM,
    WS,
    ON,
}

/// Bidi_Class of `ch` for the scripts documents mix with right-to-left text.
// ponytail: hand-written ranges (Hebrew, Arabic, Syriac, Thaana, NKo, digits,
// common punctuation); the `unicode-bidi` crate has the full table if a
// script beyond these needs it.
pub(super) fn class(ch: char) -> Class {
    use Class::*;
    match ch {
        '0'..='9' | '\u{06F0}'..='\u{06F9}' | '\u{00B2}' | '\u{00B3}' | '\u{00B9}' => EN,
        '+' | '-' | '\u{2212}' | '\u{FB29}' => ES,
        '#' | '$' | '%' | '\u{00A2}'..='\u{00A5}' | '\u{00B0}' | '\u{00B1}' => ET,
        '\u{0609}' | '\u{060A}' | '\u{066A}' | '\u{2030}'..='\u{2034}' => ET,
        '\u{20A0}'..='\u{20CF}' => ET,
        ',' | '.' | '/' | ':' | '\u{00A0}' | '\u{060C}' | '\u{202F}' | '\u{2044}' => CS,
        ' ' | '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{205F}' | '\u{3000}' => WS,
        '\u{200E}' => L,
        '\u{200F}' => R,
        '\u{0300}'..='\u{036F}'
        | '\u{0591}'..='\u{05BD}'
        | '\u{05BF}'
        | '\u{05C1}'..='\u{05C2}'
        | '\u{05C4}'..='\u{05C5}'
        | '\u{05C7}'
        | '\u{0610}'..='\u{061A}'
        | '\u{064B}'..='\u{065F}'
        | '\u{0670}'
        | '\u{06D6}'..='\u{06DC}'
        | '\u{06DF}'..='\u{06E4}'
        | '\u{06E7}'..='\u{06E8}'
        | '\u{06EA}'..='\u{06ED}'
        | '\u{0711}'
        | '\u{0730}'..='\u{074A}'
        | '\u{07A6}'..='\u{07B0}'
        | '\u{07EB}'..='\u{07F3}'
        | '\u{08D3}'..='\u{08E1}'
        | '\u{08E3}'..='\u{08FF}'
        | '\u{FB1E}' => NSM,
        '\u{0600}'..='\u{0605}'
        | '\u{0660}'..='\u{0669}'
        | '\u{066B}'
        | '\u{066C}'
        | '\u{06DD}' => AN,
        '\u{06DE}' | '\u{06E9}' | '\u{FD3E}' | '\u{FD3F}' => ON,
        '\u{0590}'..='\u{05FF}' | '\u{07C0}'..='\u{07FF}' | '\u{FB1D}'..='\u{FB4F}' => R,
        '\u{0600}'..='\u{07BF}' | '\u{08A0}'..='\u{08FF}' | '\u{FB50}'..='\u{FDFF}' => AL,
        '\u{FE70}'..='\u{FEFE}' => AL,
        c if c.is_alphanumeric() => L,
        _ => ON,
    }
}

/// The glyph a right-to-left level draws for a mirrored character.
fn mirrored(ch: char) -> char {
    match ch {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        '‹' => '›',
        '›' => '‹',
        c => c,
    }
}

/// Resolved embedding levels of one window of text, and which characters
/// are Word's left-to-right clusters inside right-to-left text.
/// `emb` is each character's run embedding (see the module docs); the
/// window starts a line, so its start is compared with `base` (X10).
pub(super) fn resolve(cls: &[Class], emb: &[u8], base: u8) -> (Vec<u8>, Vec<bool>) {
    use Class::*;
    let n = cls.len();
    let mut levels = vec![0u8; n];
    let mut cluster = vec![false; n];
    let dir = |level: u8| if level % 2 == 1 { R } else { L };
    let mut start = 0;
    while start < n {
        let e = emb[start];
        let end = (start..n).find(|&i| emb[i] != e).unwrap_or(n);
        let before = if start == 0 { base } else { emb[start - 1] };
        let after = if end == n { base } else { emb[end] };
        let (sos, eos) = (dir(e.max(before)), dir(e.max(after)));
        let mut t: Vec<Class> = cls[start..end].to_vec();
        let m = t.len();
        // W1
        for i in 0..m {
            if t[i] == NSM {
                t[i] = if i == 0 { sos } else { t[i - 1] };
            }
        }
        // W2, W3
        let mut last_strong = sos;
        for c in t.iter_mut() {
            match *c {
                L | R | AL => last_strong = *c,
                EN if last_strong == AL => *c = AN,
                _ => {}
            }
        }
        for c in t.iter_mut() {
            if *c == AL {
                *c = R;
            }
        }
        // W4
        for i in 1..m.saturating_sub(1) {
            let (p, nx) = (t[i - 1], t[i + 1]);
            if (t[i] == ES && p == EN && nx == EN)
                || (t[i] == CS && p == nx && matches!(p, EN | AN))
            {
                t[i] = p;
            }
        }
        // W5
        let mut i = 0;
        while i < m {
            if t[i] == ET {
                let j = (i..m).find(|&j| t[j] != ET).unwrap_or(m);
                if (i > 0 && t[i - 1] == EN) || (j < m && t[j] == EN) {
                    t[i..j].fill(EN);
                }
                i = j;
            } else {
                i += 1;
            }
        }
        // W6, W7
        let mut last_strong = sos;
        for c in t.iter_mut() {
            match *c {
                ES | ET | CS => *c = ON,
                L | R => last_strong = *c,
                EN if last_strong == L => *c = L,
                _ => {}
            }
        }
        // N1, N2
        let strong = |c: Class| match c {
            L => Some(L),
            R | EN | AN => Some(R),
            _ => None,
        };
        let mut i = 0;
        while i < m {
            if matches!(t[i], ON | WS) {
                let j = (i..m).find(|&j| !matches!(t[j], ON | WS)).unwrap_or(m);
                let prev = if i == 0 { Some(sos) } else { strong(t[i - 1]) };
                let next = if j == m { Some(eos) } else { strong(t[j]) };
                let to = if prev == next && prev.is_some() {
                    prev.unwrap()
                } else {
                    dir(e)
                };
                // Word: in a plain run, the neutrals between right-to-left
                // letters form a left-to-right cluster from their first
                // non-space character on (not hyphen-minus).
                let mut in_cluster = false;
                for k in i..j {
                    let orig = cls[start + k];
                    if e % 2 == 0 && to == R {
                        in_cluster |= matches!(orig, ON | CS | ET);
                        cluster[start + k] = in_cluster && orig != ES;
                    }
                    t[k] = to;
                }
                i = j;
            } else {
                i += 1;
            }
        }
        // I1, I2
        for k in 0..m {
            let lv = match (e % 2, t[k]) {
                _ if cluster[start + k] => e + 2,
                (0, R) => e + 1,
                (0, AN | EN) => e + 2,
                (1, L | EN | AN) => e + 1,
                _ => e,
            };
            levels[start + k] = lv;
        }
        start = end;
    }
    (levels, cluster)
}

/// Visual order (L2) of items with `levels`. Reversing at a level above
/// one, two items both at that level don't join when only one is a
/// cluster (see module docs): a cluster and a number stay separate units.
pub(super) fn visual_order(levels: &[u8], cluster: &[bool]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..levels.len()).collect();
    let max = levels.iter().copied().max().unwrap_or(0);
    let lowest_odd = levels.iter().copied().filter(|l| l % 2 == 1).min();
    let Some(lowest_odd) = lowest_odd.or((max > 0).then_some(1)) else {
        return order;
    };
    for k in (lowest_odd..=max).rev() {
        let mut i = 0;
        while i < order.len() {
            if levels[order[i]] < k {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < order.len()
                && levels[order[j]] >= k
                && !(k >= 2
                    && levels[order[j]] == k
                    && levels[order[j - 1]] == k
                    && cluster[order[j]] != cluster[order[j - 1]])
            {
                j += 1;
            }
            order[i..j].reverse();
            i = j;
        }
    }
    order
}

/// One character of a paragraph window: its class, run embedding, and
/// where it came from on its line (`None` for the space between chunks).
struct Unit {
    cls: Class,
    emb: u8,
    chunk: Option<(usize, usize)>,
}

/// The characters of `line`'s chunks with a whitespace unit for every gap
/// between them.
fn line_units(line: &TextLine, rtl_of: &dyn Fn(&WordChunk) -> bool, base: u8) -> Vec<Unit> {
    let emb = |rtl: bool| match (rtl, base % 2 == 1) {
        (true, _) => base | 1,
        (false, false) => base,
        (false, true) => base + 1,
    };
    let mut units = Vec::new();
    let mut prev_end: Option<f32> = None;
    for (ci, c) in line.chunks.iter().enumerate() {
        let e = emb(rtl_of(c));
        if prev_end.is_some_and(|end| c.x_offset > end + 0.01) {
            // The space keeps the embedding of the run it was typed in.
            units.push(Unit {
                cls: Class::WS,
                emb: emb(c.gap_rtl),
                chunk: None,
            });
        }
        if c.text.is_empty() {
            // A picture or check box: an object replacement character.
            units.push(Unit {
                cls: Class::ON,
                emb: e,
                chunk: Some((ci, 0)),
            });
        }
        for (k, ch) in c.text.chars().enumerate() {
            units.push(Unit {
                cls: class(ch),
                emb: e,
                chunk: Some((ci, k)),
            });
        }
        prev_end = Some(c.x_offset + c.width);
    }
    if line.trailing_space > 0.0 {
        units.push(Unit {
            cls: Class::WS,
            emb: emb(line.trailing_space_rtl),
            chunk: None,
        });
    }
    units
}

fn is_rtl_text(c: Class) -> bool {
    matches!(c, Class::R | Class::AL | Class::AN)
}

/// Reorder `lines` (one paragraph, logical order) for display: see the
/// module docs. `rtl_runs` tells whether a chunk came from a `w:rtl` run;
/// `bidi` is the paragraph's `w:bidi`.
pub(super) fn reorder_lines(
    lines: &mut [TextLine],
    bidi: bool,
    fonts: &HashMap<String, FontEntry>,
) {
    let base = u8::from(bidi);
    let rtl_of = |c: &WordChunk| c.rtl;
    let any_rtl = lines.iter().any(|l| {
        l.chunks
            .iter()
            .any(|c| c.rtl || c.text.chars().any(|ch| is_rtl_text(class(ch))))
    });
    if !any_rtl {
        return;
    }
    let per_line: Vec<Vec<Unit>> = lines.iter().map(|l| line_units(l, &rtl_of, base)).collect();
    for li in 0..lines.len() {
        // ponytail: tab and two-region lines keep logical order; their
        // pieces are placed by tab stops and float gaps, not by flow
        if lines[li].justify_from > 0 || lines[li].right_region.is_some() {
            continue;
        }
        // The line, then the rest of its paragraph (or manual-break segment)
        // up to the first strong character, which settles trailing neutrals.
        let mut cls: Vec<Class> = per_line[li].iter().map(|u| u.cls).collect();
        let mut emb: Vec<u8> = per_line[li].iter().map(|u| u.emb).collect();
        let own = cls.len();
        if !lines[li].ends_with_break {
            'ahead: for next in &per_line[li + 1..] {
                cls.push(Class::WS);
                emb.push(next.first().map_or(base, |u| u.emb));
                for u in next {
                    cls.push(u.cls);
                    emb.push(u.emb);
                    if matches!(
                        u.cls,
                        Class::L | Class::R | Class::AL | Class::EN | Class::AN
                    ) {
                        break 'ahead;
                    }
                }
            }
        }
        // No L1: Word resolves the space a line wraps at with the text after
        // it (see `reorder_line`).
        let (mut levels, mut cluster) = resolve(&cls, &emb, base);
        levels.truncate(own);
        cluster.truncate(own);
        let ltr_island = bidi && !lines[li].trailing_space_rtl;
        reorder_line(
            &mut lines[li],
            &per_line[li],
            &levels,
            &cluster,
            ltr_island,
            fonts,
        );
    }
}

/// A piece of the line: a run of one chunk's characters at one level, or a
/// gap between chunks.
struct Piece {
    chunk: Option<usize>,
    chars: std::ops::Range<usize>,
    level: u8,
    cluster: bool,
    gap: f32,
}

fn reorder_line(
    line: &mut TextLine,
    units: &[Unit],
    levels: &[u8],
    cluster: &[bool],
    keep_left_wrap_space: bool,
    fonts: &HashMap<String, FontEntry>,
) {
    if levels.iter().all(|&l| l == 0) {
        return;
    }
    // Group units into pieces, splitting chunks where the level changes.
    let mut pieces: Vec<Piece> = Vec::new();
    let mut gap_starts = Vec::new();
    for (k, u) in units.iter().enumerate() {
        match u.chunk {
            None => {
                gap_starts.push(pieces.len());
                pieces.push(Piece {
                    chunk: None,
                    chars: 0..0,
                    level: levels[k],
                    cluster: cluster[k],
                    gap: 0.0,
                });
            }
            Some((ci, ch)) => {
                if let Some(p) = pieces.last_mut()
                    && p.chunk == Some(ci)
                    && p.level == levels[k]
                    && p.cluster == cluster[k]
                {
                    p.chars.end = ch + 1;
                } else {
                    pieces.push(Piece {
                        chunk: Some(ci),
                        chars: ch..ch + 1,
                        level: levels[k],
                        cluster: cluster[k],
                        gap: 0.0,
                    });
                }
            }
        }
    }
    // Gap widths, from the logical offsets; the last gap of a wrapped line
    // is the space it wrapped at.
    for &gi in &gap_starts {
        let prev = pieces[..gi].iter().rev().find_map(|p| p.chunk);
        let next = pieces[gi + 1..].iter().find_map(|p| p.chunk);
        pieces[gi].gap = match (prev, next) {
            (Some(a), Some(b)) => {
                let (a, b) = (&line.chunks[a], &line.chunks[b]);
                (b.x_offset - (a.x_offset + a.width)).max(0.0)
            }
            _ => line.trailing_space,
        };
    }
    let piece_levels: Vec<u8> = pieces.iter().map(|p| p.level).collect();
    let piece_cluster: Vec<bool> = pieces.iter().map(|p| p.cluster).collect();
    let mut order = visual_order(&piece_levels, &piece_cluster);
    // The wrap space hangs at either end of a line, as a trailing space does
    // in left-to-right text: arabic_rice's all-Arabic first line starts its
    // ink at the margin, while a line opening with a left-to-right "-" keeps
    // the space between the dash and the Arabic (2.4pt in Word's export).
    // Except at the left end when it comes from a plain run in a w:bidi
    // paragraph, a left-to-right embedding: Word probes start such
    // justified and left-aligned lines 3.3pt in, and the same lines from
    // w:rtl runs at the margin.
    let is_wrap_space = |i: &usize| pieces[*i].chunk.is_none() && *i == pieces.len() - 1;
    if !keep_left_wrap_space && order.first().is_some_and(is_wrap_space) {
        order.remove(0);
    }
    if order.last().is_some_and(is_wrap_space) {
        order.pop();
    }

    // New chunks in visual order; `logical` keys them for the draw order.
    let old = std::mem::take(&mut line.chunks);
    let char_counts: Vec<usize> = old.iter().map(|c| c.text.chars().count()).collect();
    let mut placed: Vec<((usize, usize), WordChunk)> = Vec::new();
    let mut x = 0.0f32;
    for &pi in &order {
        let p = &pieces[pi];
        let Some(ci) = p.chunk else {
            x += p.gap;
            continue;
        };
        let src = &old[ci];
        let whole = p.chars.start == 0 && p.chars.end >= char_counts[ci];
        let mut chunk = if whole {
            src.clone()
        } else {
            split_chunk(src, p.chars.clone(), fonts)
        };
        if !whole && p.chars.end < char_counts[ci] {
            chunk.space_after = false;
        }
        chunk.x_offset = x;
        if p.level % 2 == 1 {
            chunk.rtl_glyphs = true;
            if chunk.rtl {
                let shown: String = chunk.text.chars().map(mirrored).collect();
                // Extraction reads the brackets as written, as in Word's PDF.
                if shown != chunk.text {
                    let source = std::mem::replace(&mut chunk.text, shown);
                    chunk
                        .actual_text
                        .get_or_insert_with(|| crate::docx::arabic::unshape(&source));
                }
            }
        }
        x += chunk.width;
        placed.push(((ci, p.chars.start), chunk));
    }
    let mut keys: Vec<(usize, (usize, usize))> = placed
        .iter()
        .enumerate()
        .map(|(i, (k, _))| (i, *k))
        .collect();
    keys.sort_by_key(|&(_, k)| k);
    line.draw_order = keys.into_iter().map(|(i, _)| i).collect();
    line.chunks = placed.into_iter().map(|(_, c)| c).collect();
    // A wrap space that landed inside the line widens it.
    line.total_width = x;
}

/// The characters `range` of `chunk` as a chunk of their own, its width
/// the share their measured width takes of the whole (kerning across the
/// split stays in the total).
fn split_chunk(
    chunk: &WordChunk,
    range: std::ops::Range<usize>,
    fonts: &HashMap<String, FontEntry>,
) -> WordChunk {
    let text: String = chunk
        .text
        .chars()
        .skip(range.start)
        .take(range.len())
        .collect();
    let entry = fonts.values().find(|e| e.pdf_name == chunk.pdf_font);
    let measure = |s: &str| match entry {
        Some(e) => {
            e.word_width(s, chunk.font_size, chunk.kern) * chunk.text_scale / 100.0
                + chunk.char_spacing * s.chars().count() as f32
        }
        None => s.chars().count() as f32,
    };
    let total = measure(&chunk.text);
    let mut piece = chunk.clone();
    piece.width = if total > 0.0 {
        chunk.width * measure(&text) / total
    } else {
        0.0
    };
    piece.actual_text = None;
    piece.text = text;
    piece
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Visual left-to-right order of `text` (plain runs, or one `w:rtl`
    /// run), with right-to-left letters written as A, B, C… in the input
    /// for readability: they are classed AL here.
    fn visual(text: &str, rtl: bool, base: u8) -> String {
        let chars: Vec<char> = text.chars().collect();
        let cls: Vec<Class> = chars
            .iter()
            .map(|&c| {
                if c.is_ascii_uppercase() {
                    Class::AL
                } else {
                    class(c)
                }
            })
            .collect();
        let e = if rtl {
            base | 1
        } else if base == 1 {
            2
        } else {
            0
        };
        let (levels, cluster) = resolve(&cls, &vec![e; cls.len()], base);
        visual_order(&levels, &cluster)
            .into_iter()
            .map(|i| {
                if levels[i] % 2 == 1 && rtl {
                    mirrored(chars[i])
                } else {
                    chars[i]
                }
            })
            .collect()
    }

    #[test]
    fn plain_paragraph_reverses_arabic_and_keeps_trailing_neutrals_right() {
        // Probe p1: "A1 <Arabic> ." left-aligned with the stop on the right.
        assert_eq!(visual("ab XY ZW .", false, 0), "ab WZ YX .");
        // Probe p1, w:bidi without w:rtl: the same order.
        assert_eq!(visual("ab XY ZW .", false, 1), "ab WZ YX .");
        // An rtl run: the stop turns right to left (left end).
        assert_eq!(visual("XY ZW .", true, 0), ". WZ YX");
    }

    #[test]
    fn plain_run_clusters_stay_left_to_right() {
        // Probe p6 (W1 = A, W2 = B): visual B <cluster> A.
        assert_eq!(visual("A , B", false, 0), "B,  A");
        assert_eq!(visual("A, B", false, 0), "B, A");
        assert_eq!(visual("A., B", false, 0), "B., A");
        assert_eq!(visual("A ) ( B", false, 0), "B) (  A");
        // A leading space stays with its letter: "sp (" draws "( sp".
        assert_eq!(visual("A (B", false, 0), "B( A");
        // Hyphen-minus reverses like plain UAX #9.
        assert_eq!(visual("A - B", false, 0), "B - A");
        // A number keeps its own unit: " (5) " draws ") 5 (".
        assert_eq!(visual("A (5) B", false, 0), "B) 5( A");
    }

    #[test]
    fn numbers_and_latin_inside_arabic() {
        // Probe p2 line 1: "و 4.5% و" → و % 4.5 و.
        assert_eq!(visual("A 4.5% B", false, 0), "B% 4.5 A");
        assert_eq!(visual("A 1,234.56 B", false, 0), "B 1,234.56 A");
        // Probe p6, bidi + rtl: Latin words keep their order.
        assert_eq!(
            visual("A hello world B 12 C", true, 1),
            "C 12 B hello world A"
        );
        // rtl brackets are mirrored.
        assert_eq!(visual("A (B) C", true, 1), "C (B) A");
        // Probe p2: a plain run in a w:bidi paragraph orders as at level 0.
        assert_eq!(visual("A 4.5% B", false, 1), "B% 4.5 A");
        assert_eq!(visual("A (5) B", false, 1), "B) 5( A");
    }

    #[test]
    fn hebrew_and_presentation_forms_are_right_to_left() {
        assert_eq!(class('ש'), Class::R);
        assert_eq!(class('\u{FEF7}'), Class::AL);
        assert_eq!(class('\u{0661}'), Class::AN);
        assert_eq!(class('\u{064E}'), Class::NSM);
    }
}
