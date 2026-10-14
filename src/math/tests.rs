use super::*;
use crate::model::{MathNode, MathRun, MathScript};

fn cambria_math() -> Vec<u8> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/Cambria.ttc")).unwrap()
}

fn run(text: &str) -> MathNode {
    MathNode::Run(MathRun {
        text: text.to_string(),
        style: None,
        script: MathScript::Roman,
        normal: false,
        color: None,
    })
}

fn glyphs(b: &MathBox) -> Vec<(char, f32, f32, f32)> {
    b.items
        .iter()
        .filter_map(|it| match it {
            MathItem::Glyph { ch, x, y, size, .. } => Some((*ch, *x, *y, *size)),
            _ => None,
        })
        .collect()
}

fn layout_at(root: Vec<MathNode>, display: bool, size: f32) -> MathBox {
    let data = cambria_math();
    let font = MathFont::new(&data, 1).expect("Cambria Math face");
    layout(&root, display, size, &font, &MathSettings::default())
}

#[test]
fn script_sizes_floor_to_half_points() {
    // Word: 9→6.5/5, 12→8.5/7, 16→11.5/9.5, 20→14.5/12 (probe mp_sizes).
    for (base, script, scriptscript) in [
        (9.0, 6.5, 5.0),
        (12.0, 8.5, 7.0),
        (16.0, 11.5, 9.5),
        (20.0, 14.5, 12.0),
    ] {
        let root = vec![MathNode::Scripts {
            base: vec![run("x")],
            sub: None,
            sup: Some(vec![MathNode::Scripts {
                base: vec![run("y")],
                sub: None,
                sup: Some(vec![run("2")]),
                pre: false,
            }]),
            pre: false,
        }];
        let sizes: Vec<f32> = glyphs(&layout_at(root, false, base))
            .iter()
            .map(|g| g.3)
            .collect();
        assert_eq!(sizes, vec![base, script, scriptscript], "base {base}");
    }
}

#[test]
fn pendulum_radical_fraction() {
    // T=2π ²√(l/g) at 16pt, inline: Word draws l and g at 11.5pt, 9.5pt above
    // and 8.02pt below the baseline, the degree at 9.5pt.
    let root = vec![
        run("T=2π"),
        MathNode::Rad {
            degree: Some(vec![run("2")]),
            body: vec![MathNode::Frac {
                kind: crate::model::FracKind::Bar,
                num: vec![run("l")],
                den: vec![run("g")],
            }],
        },
    ];
    let b = layout_at(root, false, 16.0);
    let g = glyphs(&b);
    let find = |c: char| {
        *g.iter()
            .find(|g| g.0 == c)
            .unwrap_or_else(|| panic!("{c} in {g:?}"))
    };
    let l = find('\u{1D459}');
    let gg = find('\u{1D454}');
    assert_eq!(l.3, 11.5);
    assert!((l.2 - 9.5).abs() < 0.3, "numerator shift {}", l.2);
    assert!((gg.2 + 8.02).abs() < 0.3, "denominator shift {}", gg.2);
    assert_eq!(find('2').3, 16.0);
    let degree = g
        .iter()
        .filter(|g| g.0 == '2')
        .map(|g| g.3)
        .fold(99.0f32, f32::min);
    assert_eq!(degree, 9.5);
    // T's italic correction and a thick space before "=": Word 4.75pt.
    let t = find('\u{1D447}');
    let eq = find('=');
    let t_adv = 9.5234375;
    assert!(
        (eq.1 - t.1 - t_adv - 4.75).abs() < 0.2,
        "gap {}",
        eq.1 - t.1 - t_adv
    );
}

#[test]
fn display_fraction_shifts() {
    // a/b on its own line at 12pt: numerator 9.05 up, denominator 7.95 down.
    let root = vec![MathNode::Frac {
        kind: crate::model::FracKind::Bar,
        num: vec![run("a")],
        den: vec![run("b")],
    }];
    let b = layout_at(root, true, 12.0);
    let g = glyphs(&b);
    assert_eq!(g[0].3, 12.0);
    assert!((g[0].2 - 9.05).abs() < 0.2, "{g:?}");
    assert!((g[1].2 + 7.95).abs() < 0.2, "{g:?}");
}

#[test]
fn binary_and_relation_spacing() {
    // a+b=c at 12pt: medium (2.67) around +, thick (3.33) around =, plus a's
    // and b's italic corrections before the upright operators.
    let b = layout_at(vec![run("a+b=c")], false, 12.0);
    let g = glyphs(&b);
    let gap = |i: usize, adv: f32| g[i + 1].1 - g[i].1 - adv;
    let data = cambria_math();
    let font = MathFont::new(&data, 1).unwrap();
    let adv = |c: char| font.metrics(font.glyph(c, 0), 12.0).0;
    let plus = adv('+');
    assert!(
        (gap(1, plus) - 2.67).abs() < 0.05,
        "after + {}",
        gap(1, plus)
    );
    let eq = adv('=');
    assert!((gap(3, eq) - 3.33).abs() < 0.05, "after = {}", gap(3, eq));
    // A leading minus is unary: no space after it.
    let b = layout_at(vec![run("-a")], false, 12.0);
    let g = glyphs(&b);
    assert_eq!(g[0].0, '\u{2212}');
    assert!((g[1].1 - adv('\u{2212}')).abs() < 0.01);
}
