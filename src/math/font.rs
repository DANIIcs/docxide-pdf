//! The math font: its MATH table, `ssty` alternates and glyph metrics.

use ttf_parser::gsub::{AlternateSubstitution, SubstitutionSubtable};
use ttf_parser::math::{Constants, MathValue};
use ttf_parser::{Face, GlyphId, Tag};

pub(crate) struct MathFont<'a> {
    face: Face<'a>,
    constants: Constants<'a>,
    /// The `ssty` feature's alternate subtables, found once.
    ssty: Vec<AlternateSubstitution<'a>>,
    upem: f32,
}

pub(super) enum Stretched {
    Glyph(u16),
    /// Parts and their offsets (points) along the stretch direction.
    Assembly(Vec<(u16, f32)>),
}

impl<'a> MathFont<'a> {
    /// The face, if it has a MATH table with constants.
    pub(crate) fn new(data: &'a [u8], index: u32) -> Option<Self> {
        let face = Face::parse(data, index).ok()?;
        let constants = face.tables().math?.constants?;
        let ssty = face
            .tables()
            .gsub
            .and_then(|gsub| {
                let feature = gsub.features.find(Tag::from_bytes(b"ssty"))?;
                let lookups = feature.lookup_indices.into_iter();
                Some(
                    lookups
                        .filter_map(|li| gsub.lookups.get(li))
                        .flat_map(|l| l.subtables.into_iter::<SubstitutionSubtable>())
                        .filter_map(|st| match st {
                            SubstitutionSubtable::Alternate(alt) => Some(alt),
                            _ => None,
                        })
                        .collect(),
                )
            })
            .unwrap_or_default();
        Some(Self {
            upem: face.units_per_em() as f32,
            face,
            constants,
            ssty,
        })
    }

    fn pt(&self, units: f32, size: f32) -> f32 {
        units * size / self.upem
    }

    pub(super) fn constant(&self, f: fn(&Constants<'a>) -> MathValue<'a>, size: f32) -> f32 {
        self.pt(f(&self.constants).value as f32, size)
    }

    pub(super) fn script_percent(&self) -> f32 {
        self.constants.script_percent_scale_down() as f32
    }

    pub(super) fn script_script_percent(&self) -> f32 {
        self.constants.script_script_percent_scale_down() as f32
    }

    pub(super) fn display_operator_min_height(&self, size: f32) -> f32 {
        self.pt(self.constants.display_operator_min_height() as f32, size)
    }

    pub(super) fn radical_degree_bottom_raise_percent(&self) -> f32 {
        self.constants.radical_degree_bottom_raise_percent() as f32
    }

    /// The font's line box per its hhea table at `size`: the ascent and
    /// descent Word gives a display-math line (Cambria Math's OS/2 win
    /// metrics are several ems tall and never used).
    pub(crate) fn line_metrics(&self, size: f32) -> (f32, f32) {
        let asc = self.face.ascender() as f32 + self.face.line_gap() as f32;
        (
            self.pt(asc, size),
            self.pt(-(self.face.descender() as f32), size),
        )
    }

    /// The glyph for `ch` at a script level (0 none, 1 script, 2 scriptscript):
    /// script levels take the `ssty` alternates.
    pub(super) fn glyph(&self, ch: char, level: usize) -> u16 {
        let gid = self.face.glyph_index(ch).map_or(0, |g| g.0);
        if level == 0 {
            return gid;
        }
        self.ssty(gid, level).unwrap_or(gid)
    }

    fn ssty(&self, gid: u16, level: usize) -> Option<u16> {
        self.ssty.iter().find_map(|alt| {
            let set = alt.alternate_sets.get(alt.coverage.get(GlyphId(gid))?)?;
            let n = set.alternates.len() as usize;
            set.alternates.get((level.min(n) - 1) as u16).map(|g| g.0)
        })
    }

    /// Advance, ink ascent and ink descent at `size`.
    pub(super) fn metrics(&self, gid: u16, size: f32) -> (f32, f32, f32) {
        let g = GlyphId(gid);
        let adv = self.face.glyph_hor_advance(g).unwrap_or(0) as f32;
        let (asc, desc) = self
            .face
            .glyph_bounding_box(g)
            .map_or((0.0, 0.0), |b| (b.y_max as f32, -(b.y_min as f32)));
        (self.pt(adv, size), self.pt(asc, size), self.pt(desc, size))
    }

    pub(super) fn italic_correction(&self, gid: u16, size: f32) -> f32 {
        self.face
            .tables()
            .math
            .and_then(|m| m.glyph_info)
            .and_then(|i| i.italic_corrections)
            .and_then(|c| c.get(GlyphId(gid)))
            .map_or(0.0, |v| self.pt(v.value as f32, size))
    }

    pub(super) fn top_accent_attachment(&self, gid: u16, size: f32) -> Option<f32> {
        let v = self
            .face
            .tables()
            .math?
            .glyph_info?
            .top_accent_attachments?
            .get(GlyphId(gid))?;
        Some(self.pt(v.value as f32, size))
    }

    /// The smallest variant of `gid` at least `target` points tall (or wide),
    /// else its assembly built to that length, else the largest variant.
    pub(super) fn stretch(&self, gid: u16, target: f32, size: f32, vertical: bool) -> Stretched {
        let Some(variants) = self.face.tables().math.and_then(|m| m.variants) else {
            return Stretched::Glyph(gid);
        };
        let constructions = if vertical {
            variants.vertical_constructions
        } else {
            variants.horizontal_constructions
        };
        let Some(c) = constructions.get(GlyphId(gid)) else {
            return Stretched::Glyph(gid);
        };
        let mut largest = gid;
        for v in c.variants {
            largest = v.variant_glyph.0;
            if self.pt(v.advance_measurement as f32, size) >= target {
                return Stretched::Glyph(largest);
            }
        }
        let Some(assembly) = c.assembly else {
            return Stretched::Glyph(largest);
        };
        let parts: Vec<_> = assembly.parts.into_iter().collect();
        let overlap = variants.min_connector_overlap as f32;
        let target_units = target * self.upem / size;
        // The fewest extender repeats whose parts, at the minimum overlap,
        // reach the target.
        let sequence = |repeats: usize| -> Vec<_> {
            parts
                .iter()
                .flat_map(|p| {
                    std::iter::repeat_n(*p, if p.part_flags.extender() { repeats } else { 1 })
                })
                .collect()
        };
        let length = |seq: &[ttf_parser::math::GlyphPart]| -> f32 {
            seq.iter().map(|p| p.full_advance as f32).sum::<f32>()
                - overlap * seq.len().saturating_sub(1) as f32
        };
        let mut repeats = 1;
        while length(&sequence(repeats)) < target_units && repeats < 100 {
            repeats += 1;
        }
        let sequence = sequence(repeats);
        // Spread any surplus over the joins, never below the minimum overlap.
        let joins = sequence.len().saturating_sub(1) as f32;
        let surplus = (length(&sequence) - target_units).max(0.0);
        let mut out = Vec::with_capacity(sequence.len());
        let mut pos = 0.0f32;
        for (i, p) in sequence.iter().enumerate() {
            if i > 0 {
                let prev = sequence[i - 1];
                let max_overlap =
                    (prev.end_connector_length.min(p.start_connector_length) as f32).max(overlap);
                let extra = if joins > 0.0 { surplus / joins } else { 0.0 };
                pos -= (overlap + extra).min(max_overlap);
            }
            out.push((p.glyph_id.0, self.pt(pos, size)));
            pos += p.full_advance as f32;
        }
        // Vertical parts stack bottom-up from 0: each sits on its position.
        if vertical {
            for o in &mut out {
                o.1 += self.metrics(o.0, size).2;
            }
        }
        Stretched::Assembly(out)
    }
}
