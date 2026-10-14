//! Office Math layout: an OMML tree (`model::MathZone`) to a box of placed
//! glyphs and rules, driven by the math font's OpenType MATH table the way
//! Word does it (Word's math engine is where that table comes from).
//!
//! Rules measured from Word's exports (online and local; see
//! `current_focus/office-math.md`):
//! - script sizes are the base size × ScriptPercentScaleDown (73%) and
//!   × ScriptScriptPercentScaleDown (60%), floored to the half point
//!   (16pt → 11.5 / 9.5, 12pt → 8.5 / 7.0);
//! - script-level glyphs take the font's `ssty` alternates (1st and 2nd);
//! - letters map to the Mathematical Alphanumeric Symbols (italic by default);
//! - atoms are spaced with TeX's table (thin 3/18, medium 4/18, thick 5/18 em);
//! - an italic glyph followed by an upright one keeps its italic correction.

mod alphanumeric;
mod font;

use crate::model::{FracKind, MathNode, MathRow, MathRun, MathSettings};
pub(crate) use font::MathFont;
use ttf_parser::math::{Constants, MathValue};

/// A laid-out zone. Coordinates are points from the box's left end on its
/// baseline, y up.
#[derive(Debug)]
pub struct MathBox {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub items: Vec<MathItem>,
    /// The font's MathLeading at the zone's size: ink higher than the text's
    /// ascent less this gap raises the line (OpenType MATH, `MathLeading`).
    pub leading: f32,
    /// The math font's own line box at the zone's size (`MathFont::line_metrics`):
    /// what a line of nothing but plain math is sized by.
    pub line_ascent: f32,
    pub line_descent: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MathItem {
    /// A glyph of the math font by its original id; `ch` is what it reads as.
    Glyph {
        gid: u16,
        ch: char,
        x: f32,
        y: f32,
        size: f32,
        color: Option<[u8; 3]>,
    },
    /// A filled rectangle (fraction bar, radical overbar) in the text colour,
    /// `y` its bottom.
    Rule { x: f32, y: f32, w: f32, h: f32 },
}

/// Lay out one zone at `size` points.
pub(crate) fn layout(
    root: &MathRow,
    display: bool,
    size: f32,
    font: &MathFont,
    settings: &MathSettings,
) -> MathBox {
    let cx = Ctx {
        font,
        settings,
        base: size,
    };
    let style = if display { Style::D } else { Style::T };
    let b = cx.row(root, St::new(style));
    let (line_ascent, line_descent) = font.line_metrics(size);
    MathBox {
        width: b.w,
        ascent: b.asc,
        descent: b.desc,
        items: b.items,
        leading: font.constant(Constants::math_leading, size),
        line_ascent,
        line_descent,
    }
}

/// TeX's styles: display, text, script, scriptscript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    D,
    T,
    S,
    SS,
}

/// A style and whether it is cramped (superscripts sit lower).
#[derive(Debug, Clone, Copy)]
struct St {
    style: Style,
    cramped: bool,
}

impl St {
    fn new(style: Style) -> Self {
        Self {
            style,
            cramped: false,
        }
    }
    fn script_level(self) -> usize {
        match self.style {
            Style::D | Style::T => 0,
            Style::S => 1,
            Style::SS => 2,
        }
    }
    fn display(self) -> bool {
        self.style == Style::D
    }
    /// Superscript style.
    fn sup(self) -> Self {
        Self {
            style: match self.style {
                Style::D | Style::T => Style::S,
                _ => Style::SS,
            },
            cramped: self.cramped,
        }
    }
    /// Subscript style.
    fn sub(self) -> Self {
        self.sup().cramp()
    }
    /// Numerator style; the denominator is its cramped form.
    fn num(self) -> Self {
        Self {
            style: match self.style {
                Style::D => Style::T,
                Style::T => Style::S,
                _ => Style::SS,
            },
            cramped: self.cramped,
        }
    }
    fn cramp(self) -> Self {
        Self {
            cramped: true,
            ..self
        }
    }
}

/// TeX's atom classes, for the space between neighbours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
}

/// A box being built: like `MathBox`, plus what spacing needs to know of
/// its ends.
struct B {
    w: f32,
    asc: f32,
    desc: f32,
    items: Vec<MathItem>,
    first: Class,
    last: Class,
    /// Italic correction of the last glyph when it is italic.
    ic: f32,
    first_italic: bool,
    /// One glyph only (scripts attach to its italic correction).
    single_glyph: Option<u16>,
}

impl B {
    fn empty() -> Self {
        Self {
            w: 0.0,
            asc: 0.0,
            desc: 0.0,
            items: Vec::new(),
            first: Class::Ord,
            last: Class::Ord,
            ic: 0.0,
            first_italic: false,
            single_glyph: None,
        }
    }

    /// An empty box with `base`'s ends, for spacing it as `base`.
    fn ends_of(base: &B) -> Self {
        Self {
            first: base.first,
            last: base.last,
            ic: base.ic,
            first_italic: base.first_italic,
            ..Self::empty()
        }
    }

    fn ord(mut self) -> Self {
        self.first = Class::Ord;
        self.last = Class::Ord;
        self.single_glyph = None;
        self
    }

    /// Place `other` with its origin at (dx, dy) in this box.
    fn put(&mut self, other: B, dx: f32, dy: f32) {
        self.w = self.w.max(dx + other.w);
        self.asc = self.asc.max(other.asc + dy);
        self.desc = self.desc.max(other.desc - dy);
        self.items
            .extend(other.items.into_iter().map(|it| shift(it, dx, dy)));
    }

    fn rule(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.w = self.w.max(x + w);
        self.asc = self.asc.max(y + h);
        self.desc = self.desc.max(-y);
        self.items.push(MathItem::Rule { x, y, w, h });
    }
}

fn shift(mut it: MathItem, dx: f32, dy: f32) -> MathItem {
    match &mut it {
        MathItem::Glyph { x, y, .. } | MathItem::Rule { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
    }
    it
}

struct Ctx<'a> {
    font: &'a MathFont<'a>,
    settings: &'a MathSettings,
    base: f32,
}

/// Floor to the half point, as Word sizes its script levels.
fn half_point_floor(v: f32) -> f32 {
    ((v * 2.0) + 1e-3).floor() / 2.0
}

type Constant<'a> = fn(&Constants<'a>) -> MathValue<'a>;

impl<'a> Ctx<'a> {
    fn size(&self, st: St) -> f32 {
        let pct = match st.script_level() {
            0 => return self.base,
            1 => self.font.script_percent(),
            _ => self.font.script_script_percent(),
        };
        half_point_floor(self.base * pct / 100.0)
    }

    /// A MATH constant in points at `st`'s size.
    fn k(&self, st: St, f: Constant<'a>) -> f32 {
        self.font.constant(f, self.size(st))
    }

    /// The display-style constant in display style, else the other.
    fn kd(&self, st: St, display: Constant<'a>, text: Constant<'a>) -> f32 {
        self.k(st, if st.display() { display } else { text })
    }

    fn glyph(&self, ch: char, st: St, color: Option<[u8; 3]>) -> B {
        let gid = self.font.glyph(ch, st.script_level());
        self.glyph_id(gid, ch, self.size(st), color)
    }

    fn glyph_id(&self, gid: u16, ch: char, size: f32, color: Option<[u8; 3]>) -> B {
        let (adv, asc, desc) = self.font.metrics(gid, size);
        B {
            w: adv,
            asc,
            desc,
            items: vec![MathItem::Glyph {
                gid,
                ch,
                x: 0.0,
                y: 0.0,
                size,
                color,
            }],
            single_glyph: Some(gid),
            ..B::empty()
        }
    }

    /// Lay out a row: atoms side by side with TeX's spacing. A run's
    /// characters are atoms of the row, so a sign at a run's start can be
    /// binary.
    fn row(&self, row: &MathRow, st: St) -> B {
        let mut atoms: Vec<B> = Vec::new();
        for node in row {
            match node {
                MathNode::Run(run) => atoms.extend(self.run(run, st)),
                _ => atoms.push(self.node(node, st)),
            }
        }
        self.hbox(atoms, st)
    }

    fn hbox(&self, mut atoms: Vec<B>, st: St) -> B {
        // TeX rule: a binary operator with nothing to operate on is ordinary.
        for i in 0..atoms.len() {
            if atoms[i].first == Class::Bin {
                let prev = i.checked_sub(1).map(|p| atoms[p].last);
                let next = atoms.get(i + 1).map(|n| n.first);
                let unary = matches!(
                    prev,
                    None | Some(Class::Bin | Class::Op | Class::Rel | Class::Open | Class::Punct)
                ) || matches!(
                    next,
                    None | Some(Class::Rel | Class::Close | Class::Punct)
                );
                if unary {
                    atoms[i].first = Class::Ord;
                    atoms[i].last = Class::Ord;
                }
            }
        }
        if atoms.len() == 1 {
            return atoms.pop().unwrap();
        }
        let em = self.size(st);
        let mut out = atoms.first().map_or_else(B::empty, B::ends_of);
        let mut prev: Option<(Class, f32)> = None;
        for a in atoms {
            if let Some((last, ic)) = prev {
                out.w += space(last, a.first, st) * em / 18.0;
                // An italic glyph followed by an upright one keeps its
                // italic correction.
                if !a.first_italic {
                    out.w += ic;
                }
            }
            prev = Some((a.last, a.ic));
            out.put(a, out.w, 0.0);
        }
        (out.last, out.ic) = prev.unwrap_or((Class::Ord, 0.0));
        out
    }

    /// A math run: one atom per character.
    fn run(&self, run: &MathRun, st: St) -> Vec<B> {
        let size = self.size(st);
        run.text
            .chars()
            .map(|ch| {
                let (drawn, italic) = alphanumeric::styled(ch, run.style, run.script);
                let drawn = match drawn {
                    // Word draws a hyphen-minus in math as the minus sign.
                    '-' if !run.normal => '\u{2212}',
                    '\'' if !run.normal => '\u{2032}',
                    c => c,
                };
                let gid = self.font.glyph(drawn, st.script_level());
                let mut b = self.glyph_id(gid, drawn, size, run.color);
                if italic {
                    b.first_italic = true;
                    b.ic = self.font.italic_correction(gid, size);
                }
                let class = if run.normal {
                    Class::Ord
                } else {
                    class_of(drawn)
                };
                b.first = class;
                b.last = class;
                b
            })
            .collect()
    }

    fn node(&self, node: &MathNode, st: St) -> B {
        match node {
            MathNode::Run(run) => self.hbox(self.run(run, st), st),
            MathNode::Frac { kind, num, den } => self.frac(*kind, num, den, st),
            MathNode::Scripts {
                base,
                sub,
                sup,
                pre,
            } => self.scripts(self.row(base, st), sub.as_ref(), sup.as_ref(), *pre, st),
            MathNode::Rad { degree, body } => self.radical(degree.as_ref(), body, st),
            MathNode::Nary {
                chr,
                under_over,
                sub,
                sup,
                grow,
                body,
            } => self.nary(
                *chr,
                *under_over,
                sub.as_ref(),
                sup.as_ref(),
                *grow,
                body,
                st,
            ),
            MathNode::Delim {
                open,
                close,
                sep,
                grow,
                items,
            } => self.delim(*open, *close, *sep, *grow, items, st),
            MathNode::Func { name, body } => {
                let mut name_b = self.row(name, st);
                name_b.first = Class::Op;
                name_b.last = Class::Op;
                self.hbox(vec![name_b, self.row(body, st)], st)
            }
            MathNode::Accent { chr, body } => self.accent(*chr, body, st),
            MathNode::Bar { top, body } => self.bar(*top, body, st),
            MathNode::GroupChr { chr, top, body } => self.group_chr(*chr, *top, body, st),
            MathNode::Limit { upper, body, limit } => {
                let b = self.row(body, st);
                let lim = Some(self.row(limit, st.sup()));
                if *upper {
                    self.limits(b, None, lim, st)
                } else {
                    self.limits(b, lim, None, st)
                }
            }
            MathNode::Matrix { rows } => {
                let rows = rows
                    .iter()
                    .map(|r| r.iter().map(|c| self.row(c, st)).collect())
                    .collect();
                self.grid(rows, st, true)
            }
            MathNode::EqArr { rows } => {
                let rows = rows.iter().map(|r| vec![self.row(r, st)]).collect();
                self.grid(rows, st, false)
            }
            MathNode::Boxed(body) => self.row(body, st).ord(),
            MathNode::BorderBox { hide, body } => self.border_box(*hide, body, st),
            MathNode::Phantom {
                show,
                zero_width,
                zero_ascent,
                zero_descent,
                body,
            } => {
                let mut b = self.row(body, st).ord();
                if !show {
                    b.items.clear();
                }
                if *zero_width {
                    b.w = 0.0;
                }
                if *zero_ascent {
                    b.asc = 0.0;
                }
                if *zero_descent {
                    b.desc = 0.0;
                }
                b
            }
        }
    }

    fn frac(&self, kind: FracKind, num: &MathRow, den: &MathRow, st: St) -> B {
        // m:smallFrac sets display fractions at text size.
        let st = if self.settings.small_frac && st.display() {
            St::new(Style::T)
        } else {
            st
        };
        if kind == FracKind::Linear {
            let slash = self.glyph('/', st, None).ord();
            let atoms = vec![self.row(num, st), slash, self.row(den, st)];
            return self.hbox(atoms, st).ord();
        }
        let n = self.row(num, st.num());
        let d = self.row(den, st.num().cramp());
        if kind == FracKind::Skewed {
            return self.skewed(n, d, st);
        }
        let w = n.w.max(d.w);
        let (nx, dx) = ((w - n.w) / 2.0, (w - d.w) / 2.0);
        let mut b = B::empty();
        if kind == FracKind::NoBar {
            let mut up = self.kd(
                st,
                Constants::stack_top_display_style_shift_up,
                Constants::stack_top_shift_up,
            );
            let mut down = self.kd(
                st,
                Constants::stack_bottom_display_style_shift_down,
                Constants::stack_bottom_shift_down,
            );
            let gap = self.kd(
                st,
                Constants::stack_display_style_gap_min,
                Constants::stack_gap_min,
            );
            let actual = (up - n.desc) - (d.asc - down);
            if actual < gap {
                up += (gap - actual) / 2.0;
                down += (gap - actual) / 2.0;
            }
            b.put(n, nx, up);
            b.put(d, dx, -down);
            return b;
        }
        let axis = self.k(st, Constants::axis_height);
        let t = self.k(st, Constants::fraction_rule_thickness);
        let up = self
            .kd(
                st,
                Constants::fraction_numerator_display_style_shift_up,
                Constants::fraction_numerator_shift_up,
            )
            .max(
                axis + t / 2.0
                    + n.desc
                    + self.kd(
                        st,
                        Constants::fraction_num_display_style_gap_min,
                        Constants::fraction_numerator_gap_min,
                    ),
            );
        let down = self
            .kd(
                st,
                Constants::fraction_denominator_display_style_shift_down,
                Constants::fraction_denominator_shift_down,
            )
            .max(
                d.asc - (axis - t / 2.0)
                    + self.kd(
                        st,
                        Constants::fraction_denom_display_style_gap_min,
                        Constants::fraction_denominator_gap_min,
                    ),
            );
        b.put(n, nx, up);
        b.put(d, dx, -down);
        b.rule(0.0, axis - t / 2.0, w, t);
        b
    }

    /// `m:type="skw"`: numerator raised, denominator lowered, a slash between.
    fn skewed(&self, n: B, d: B, st: St) -> B {
        let gap = self.k(st, Constants::skewed_fraction_horizontal_gap);
        let slash = self.glyph('/', st, None);
        let up = self.k(st, Constants::superscript_shift_up);
        let down = self.k(st, Constants::subscript_shift_down);
        let (nw, sw) = (n.w, slash.w);
        let mut b = B::empty();
        b.put(n, 0.0, up);
        b.put(slash, nw + (gap - sw).max(0.0) / 2.0, 0.0);
        b.put(d, nw + gap.max(sw), -down);
        b
    }

    fn scripts(
        &self,
        base: B,
        sub: Option<&MathRow>,
        sup: Option<&MathRow>,
        pre: bool,
        st: St,
    ) -> B {
        let sub_b = sub.map(|r| self.row(r, st.sub()));
        let sup_b = sup.map(|r| self.row(r, st.sup()));
        if !pre {
            let (bw, ic) = (base.w, base.ic);
            // The superscript clears the base's italic correction.
            return self.attach(base, sub_b, sup_b, bw, bw + ic, st);
        }
        let (sub_shift, sup_shift) = self.script_shifts(&base, sub_b.as_ref(), sup_b.as_ref(), st);
        let sw = sub_b.as_ref().map_or(0.0, |s| s.w);
        let pw = sup_b.as_ref().map_or(0.0, |s| s.w);
        let scripts_w = sw.max(pw);
        let mut b = B::ends_of(&base);
        if let Some(s) = sub_b {
            b.put(s, scripts_w - sw, -sub_shift);
        }
        if let Some(s) = sup_b {
            b.put(s, scripts_w - pw, sup_shift);
        }
        b.put(base, scripts_w, 0.0);
        b
    }

    /// `base` with a subscript at `sub_x` and a superscript at `sup_x`, then
    /// SpaceAfterScript.
    fn attach(&self, base: B, sub: Option<B>, sup: Option<B>, sub_x: f32, sup_x: f32, st: St) -> B {
        let (sub_shift, sup_shift) = self.script_shifts(&base, sub.as_ref(), sup.as_ref(), st);
        let mut b = B::ends_of(&base);
        b.put(base, 0.0, 0.0);
        if let Some(s) = sub {
            b.put(s, sub_x, -sub_shift);
        }
        if let Some(s) = sup {
            b.put(s, sup_x, sup_shift);
        }
        b.w += self.k(st, Constants::space_after_script);
        b
    }

    /// Script shifts (sub down, sup up), the OpenType MATH rules.
    fn script_shifts(&self, base: &B, sub: Option<&B>, sup: Option<&B>, st: St) -> (f32, f32) {
        // A lone glyph's scripts don't drop with its ink (TeX's rule 18a).
        let (b_asc, b_desc) = if base.single_glyph.is_some() {
            (0.0, 0.0)
        } else {
            (base.asc, base.desc)
        };
        let mut sub_shift = sub.map_or(0.0, |s| {
            self.k(st, Constants::subscript_shift_down)
                .max(b_desc + self.k(st, Constants::subscript_baseline_drop_min))
                .max(s.asc - self.k(st, Constants::subscript_top_max))
        });
        let mut sup_shift = sup.map_or(0.0, |s| {
            let up = if st.cramped {
                self.k(st, Constants::superscript_shift_up_cramped)
            } else {
                self.k(st, Constants::superscript_shift_up)
            };
            up.max(b_asc - self.k(st, Constants::superscript_baseline_drop_max))
                .max(s.desc + self.k(st, Constants::superscript_bottom_min))
        });
        if let (Some(sb), Some(sp)) = (sub, sup) {
            let gap_min = self.k(st, Constants::sub_superscript_gap_min);
            let gap = (sup_shift - sp.desc) - (sb.asc - sub_shift);
            if gap < gap_min {
                sub_shift += gap_min - gap;
                let max_bottom = self.k(st, Constants::superscript_bottom_max_with_subscript);
                let delta = max_bottom - (sup_shift - sp.desc);
                if delta > 0.0 {
                    sup_shift += delta;
                    sub_shift -= delta;
                }
            }
        }
        (sub_shift, sup_shift)
    }

    fn radical(&self, degree: Option<&MathRow>, body: &MathRow, st: St) -> B {
        let body_b = self.row(body, st.cramp());
        let t = self.k(st, Constants::radical_rule_thickness);
        let mut gap = self.kd(
            st,
            Constants::radical_display_style_vertical_gap,
            Constants::radical_vertical_gap,
        );
        let target = body_b.asc + body_b.desc + gap + t;
        let surd = self.stretch('√', target, self.size(st), true);
        let surd_h = surd.asc + surd.desc;
        if surd_h > target {
            gap += (surd_h - target) / 2.0;
        }
        let rule_top = body_b.asc + gap + t;
        // The surd's top is the overbar's top.
        let surd_dy = rule_top - surd.asc;
        let surd_bottom = surd.desc - surd_dy;
        let mut b = B::empty();
        let mut x = 0.0;
        if let Some(deg) = degree {
            let d = self.row(deg, St::new(Style::SS));
            let before = self.k(st, Constants::radical_kern_before_degree);
            let after = self.k(st, Constants::radical_kern_after_degree);
            let raise = self.font.radical_degree_bottom_raise_percent() / 100.0;
            let dy = raise * (rule_top + surd_bottom) - surd_bottom + d.desc;
            x = (before + d.w + after).max(0.0);
            b.put(d, before, dy);
        }
        let surd_w = surd.w;
        b.put(surd, x, surd_dy);
        x += surd_w;
        let bw = body_b.w;
        b.put(body_b, x, 0.0);
        b.rule(x, rule_top - t, bw, t);
        b.asc = b
            .asc
            .max(rule_top + self.k(st, Constants::radical_extra_ascender));
        b
    }

    #[allow(clippy::too_many_arguments)]
    fn nary(
        &self,
        chr: char,
        under_over: Option<bool>,
        sub: Option<&MathRow>,
        sup: Option<&MathRow>,
        grow: bool,
        body: &MathRow,
        st: St,
    ) -> B {
        let size = self.size(st);
        let body_b = self.row(body, st);
        // m:grow: the operator spans its operand.
        let grown = if grow { body_b.asc + body_b.desc } else { 0.0 };
        let op = if st.display() || grow {
            // Word sets every display operator one variant taller than the
            // font's DisplayOperatorMinHeight (1.22em) asks: probes of 20
            // operators all take the variant of 1.91-2.15em, never the ∫
            // family's 1.35em one, so the bar lies in (1.35, 1.91] em.
            let min_h = if st.display() {
                1.5 * self.font.display_operator_min_height(size)
            } else {
                0.0
            };
            self.stretch(chr, min_h.max(grown), size, true)
        } else {
            self.glyph(chr, st, None)
        };
        let op_ic = op
            .single_glyph
            .map_or(0.0, |g| self.font.italic_correction(g, size));
        // Large operators are centred on the math axis.
        let mut centred = B {
            single_glyph: op.single_glyph,
            ..B::empty()
        };
        let dy = self.k(st, Constants::axis_height) - (op.asc - op.desc) / 2.0;
        centred.put(op, 0.0, dy);
        let is_integral = matches!(
            chr,
            '∫' | '∬' | '∭' | '∮' | '∯' | '∰' | '∱' | '∲' | '∳' | '⨌'
        );
        let default_under = if is_integral {
            self.settings.int_under_over
        } else {
            self.settings.nary_under_over
        };
        let lo = sub.map(|r| self.row(r, st.sub()));
        let hi = sup.map(|r| self.row(r, st.sup()));
        // Inline (text style) limits always go beside the operator.
        let mut op_b = if under_over.unwrap_or(default_under) && st.display() {
            self.limits(centred, lo, hi, st)
        } else if lo.is_some() || hi.is_some() {
            // The italic correction moves the lower limit left of the upper
            // one (OpenType MATH, large operators).
            let w = centred.w;
            self.attach(centred, lo, hi, w - op_ic, w, st)
        } else {
            centred
        };
        op_b.first = Class::Op;
        op_b.last = Class::Op;
        self.hbox(vec![op_b, body_b], st)
    }

    /// Limits centred under and over `base` (`m:limLow`, `m:limUpp`, n-ary
    /// operators with undOvr limits).
    fn limits(&self, base: B, lo: Option<B>, hi: Option<B>, st: St) -> B {
        let w = base
            .w
            .max(lo.as_ref().map_or(0.0, |l| l.w))
            .max(hi.as_ref().map_or(0.0, |h| h.w));
        let (base_asc, base_desc) = (base.asc, base.desc);
        let mut b = B::empty();
        let mut centre = |part: B, dy: f32| {
            let x = (w - part.w) / 2.0;
            b.put(part, x, dy);
        };
        centre(base, 0.0);
        if let Some(h) = hi {
            let dy = (base_asc + self.k(st, Constants::upper_limit_gap_min) + h.desc)
                .max(base_asc + self.k(st, Constants::upper_limit_baseline_rise_min));
            centre(h, dy);
        }
        if let Some(l) = lo {
            let dy = (base_desc + self.k(st, Constants::lower_limit_gap_min) + l.asc)
                .max(base_desc + self.k(st, Constants::lower_limit_baseline_drop_min));
            centre(l, -dy);
        }
        b
    }

    #[allow(clippy::too_many_arguments)]
    fn delim(
        &self,
        open: Option<char>,
        close: Option<char>,
        sep: char,
        grow: bool,
        items: &[MathRow],
        st: St,
    ) -> B {
        let size = self.size(st);
        let axis = self.k(st, Constants::axis_height);
        let bodies: Vec<B> = items.iter().map(|r| self.row(r, st)).collect();
        let (asc, desc) = bodies
            .iter()
            .fold((0.0f32, 0.0f32), |(a, d), b| (a.max(b.asc), d.max(b.desc)));
        // Delimiters cover the content symmetrically about the axis, unless
        // m:grow is off; even at base size they sit centred on it.
        let target = if grow {
            2.0 * (asc - axis).max(desc + axis)
        } else {
            0.0
        };
        let fence = |ch: char, class: Class| {
            let g = self.stretch(ch, target, size, true);
            let dy = axis - (g.asc - g.desc) / 2.0;
            let mut b = B {
                first: class,
                last: class,
                ..B::empty()
            };
            b.put(g, 0.0, dy);
            b
        };
        let mut parts: Vec<B> = Vec::new();
        parts.extend(open.map(|c| fence(c, Class::Open)));
        let n = bodies.len();
        for (i, body) in bodies.into_iter().enumerate() {
            parts.push(body.ord());
            if i + 1 < n {
                parts.push(fence(sep, Class::Punct));
            }
        }
        parts.extend(close.map(|c| fence(c, Class::Close)));
        let mut b = self.hbox(parts, st);
        b.first_italic = false;
        b.ic = 0.0;
        b
    }

    fn accent(&self, chr: char, body: &MathRow, st: St) -> B {
        let base = self.row(body, st.cramp());
        let size = self.size(st);
        let acc = self.stretch(chr, base.w, size, false);
        // Accents are drawn for x-height letters; a taller base lifts them.
        let dy = (base.asc - self.k(st, Constants::accent_base_height)).max(0.0);
        let attach = |b: &B| {
            b.single_glyph
                .and_then(|g| self.font.top_accent_attachment(g, size))
                .unwrap_or(b.w / 2.0)
        };
        let dx = attach(&base) - attach(&acc);
        let mut b = B::ends_of(&base);
        let w = base.w;
        b.put(base, 0.0, 0.0);
        b.put(acc, dx, dy);
        // An accent wider than its base doesn't widen it.
        b.w = w;
        b
    }

    fn bar(&self, top: bool, body: &MathRow, st: St) -> B {
        let base = if top {
            self.row(body, st.cramp())
        } else {
            self.row(body, st)
        };
        let (w, asc, desc) = (base.w, base.asc, base.desc);
        let mut b = B::empty();
        b.put(base, 0.0, 0.0);
        if top {
            let gap = self.k(st, Constants::overbar_vertical_gap);
            let t = self.k(st, Constants::overbar_rule_thickness);
            b.rule(0.0, asc + gap, w, t);
            b.asc = b
                .asc
                .max(asc + gap + t + self.k(st, Constants::overbar_extra_ascender));
        } else {
            let gap = self.k(st, Constants::underbar_vertical_gap);
            let t = self.k(st, Constants::underbar_rule_thickness);
            b.rule(0.0, -(desc + gap + t), w, t);
            b.desc = b
                .desc
                .max(desc + gap + t + self.k(st, Constants::underbar_extra_descender));
        }
        b
    }

    fn group_chr(&self, chr: char, top: bool, body: &MathRow, st: St) -> B {
        let base = self.row(body, st);
        let brace = self.stretch(chr, base.w, self.size(st), false);
        let w = base.w.max(brace.w);
        let dy = if top {
            base.asc + self.k(st, Constants::stretch_stack_gap_below_min) + brace.desc
        } else {
            -(base.desc + self.k(st, Constants::stretch_stack_gap_above_min) + brace.asc)
        };
        let mut b = B::empty();
        let (bx, cx) = ((w - base.w) / 2.0, (w - brace.w) / 2.0);
        b.put(base, bx, 0.0);
        b.put(brace, cx, dy);
        b
    }

    /// Matrix cells (centred, columns an em apart) or equation-array rows,
    /// stacked a math line apart and centred on the math axis.
    fn grid(&self, rows: Vec<Vec<B>>, st: St, matrix: bool) -> B {
        let size = self.size(st);
        let (line_asc, line_desc) = self.font.line_metrics(size);
        let ncols = rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut col_w = vec![0.0f32; ncols];
        for r in &rows {
            for (i, c) in r.iter().enumerate() {
                col_w[i] = col_w[i].max(c.w);
            }
        }
        let col_gap = if matrix { size } else { 0.0 };
        let total_w = col_w.iter().sum::<f32>() + col_gap * ncols.saturating_sub(1) as f32;
        // Baselines from the first row's (y = 0) down; `top` is that row's ascent.
        let (mut y, mut top) = (0.0f32, None);
        let mut placed: Vec<(B, f32, f32)> = Vec::new();
        for r in rows {
            let asc = r.iter().map(|c| c.asc).fold(line_asc, f32::max);
            let desc = r.iter().map(|c| c.desc).fold(line_desc, f32::max);
            if top.is_some() {
                y -= asc;
            }
            top.get_or_insert(asc);
            let mut x = 0.0;
            for (i, c) in r.into_iter().enumerate() {
                let cx = if matrix {
                    x + (col_w[i] - c.w) / 2.0
                } else {
                    (total_w - c.w) / 2.0
                };
                placed.push((c, cx, y));
                x += col_w[i] + col_gap;
            }
            y -= desc;
        }
        let top = top.unwrap_or(0.0);
        let dy = self.k(st, Constants::axis_height) - (top + y) / 2.0;
        let mut b = B::empty();
        for (c, x, cy) in placed {
            b.put(c, x, cy + dy);
        }
        b.asc = b.asc.max(top + dy);
        b.desc = b.desc.max(-(y + dy));
        b.w = total_w;
        b
    }

    fn border_box(&self, hide: [bool; 4], body: &MathRow, st: St) -> B {
        let base = self.row(body, st);
        let t = self.k(st, Constants::fraction_rule_thickness);
        // ponytail: the padding is one rule thickness plus the radical gap;
        // measure Word's borderBox if documents rely on it.
        let pad = self.k(st, Constants::radical_vertical_gap) + t;
        let (w, asc, desc) = (base.w + 2.0 * pad, base.asc + pad, base.desc + pad);
        let mut b = B::empty();
        b.put(base, pad, 0.0);
        let [top, bot, left, right] = hide;
        if !top {
            b.rule(0.0, asc - t, w, t);
        }
        if !bot {
            b.rule(0.0, -desc, w, t);
        }
        if !left {
            b.rule(0.0, -desc, t, asc + desc);
        }
        if !right {
            b.rule(w - t, -desc, t, asc + desc);
        }
        b
    }

    /// The glyph for `ch` at least `target` tall (or wide): the base glyph,
    /// a larger variant, or an assembly of parts.
    fn stretch(&self, ch: char, target: f32, size: f32, vertical: bool) -> B {
        let gid = self.font.glyph(ch, 0);
        match self.font.stretch(gid, target, size, vertical) {
            font::Stretched::Glyph(g) => self.glyph_id(g, ch, size, None),
            font::Stretched::Assembly(parts) => {
                let mut b = B::empty();
                for (g, offset) in parts {
                    let (dx, dy) = if vertical {
                        (0.0, offset)
                    } else {
                        (offset, 0.0)
                    };
                    b.put(self.glyph_id(g, ch, size, None), dx, dy);
                }
                b
            }
        }
    }
}

/// TeX's inter-atom space in eighteenths of an em; the medium and thick
/// spaces (and most thin ones) vanish in script styles.
fn space(left: Class, right: Class, st: St) -> f32 {
    use Class::*;
    let script = st.script_level() > 0;
    let (n, script_too) = match (left, right) {
        (Ord, Op) | (Op, Ord) | (Op, Op) | (Close, Op) => (3.0, true),
        (Ord, Bin) | (Bin, Ord) | (Bin, Op) | (Bin, Open) | (Op, Bin) | (Close, Bin) => {
            (4.0, false)
        }
        (Ord | Op | Close, Rel) | (Rel, Ord | Op | Open) => (5.0, false),
        (Punct, Ord | Op | Rel | Open | Close | Punct) => (3.0, false),
        _ => (0.0, true),
    };
    if script && !script_too { 0.0 } else { n }
}

/// The class of a character in a math run.
fn class_of(ch: char) -> Class {
    match ch {
        '+' | '\u{2212}' | '±' | '∓' | '×' | '÷' | '⋅' | '∙' | '·' | '∗' | '∘' | '∩' | '∪'
        | '⊕' | '⊖' | '⊗' | '⊘' | '⊙' | '∧' | '∨' | '∖' | '⋆' | '⋄' | '⊎' | '⊓' | '⊔' | '≀' => {
            Class::Bin
        }
        '='
        | '<'
        | '>'
        | '≤'
        | '≥'
        | '≠'
        | '≈'
        | '≡'
        | '≅'
        | '∼'
        | '≃'
        | '≪'
        | '≫'
        | '∝'
        | '∈'
        | '∉'
        | '∋'
        | '⊂'
        | '⊃'
        | '⊆'
        | '⊇'
        | '⊊'
        | '⊋'
        | '∣'
        | '∥'
        | '⊥'
        | '≺'
        | '≻'
        | '≼'
        | '≽'
        | '≔'
        | ':'
        | '≐'
        | '≜'
        | '≦'
        | '≧'
        | '≲'
        | '≳'
        | '⊢'
        | '⊨'
        | '∶'
        | '≍'
        | '≑'
        | '\u{2192}'..='\u{21FF}'
        | '\u{2190}'
        | '\u{2191}'
        | '\u{27F0}'..='\u{27FF}' => Class::Rel,
        '(' | '[' | '{' | '⟨' | '⌈' | '⌊' | '⟦' | '〈' => Class::Open,
        ')' | ']' | '}' | '⟩' | '⌉' | '⌋' | '⟧' | '〉' | '!' => Class::Close,
        ',' | ';' => Class::Punct,
        '∑' | '∏' | '∐' | '∫' | '∬' | '∭' | '∮' | '∯' | '∰' | '⋂' | '⋃' | '⋀' | '⋁' | '⨀' | '⨁'
        | '⨂' | '⨄' | '⨆' => Class::Op,
        _ => Class::Ord,
    }
}

#[cfg(test)]
mod tests;
