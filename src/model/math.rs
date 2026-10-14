//! Office Math (OMML, ISO/IEC 29500-1 §22.1) as a tree, laid out by `crate::math`.

/// One `m:oMath` zone: inline in a paragraph, or one equation of an
/// `m:oMathPara` (display math). Its run's size is the zone's.
#[derive(Debug)]
pub struct MathZone {
    pub root: MathRow,
    pub display: bool,
}

pub type MathRow = Vec<MathNode>;

#[derive(Debug, Clone)]
pub enum MathNode {
    Run(MathRun),
    Frac {
        kind: FracKind,
        num: MathRow,
        den: MathRow,
    },
    /// `m:sSub`, `m:sSup`, `m:sSubSup` and (with `pre`) `m:sPre`.
    Scripts {
        base: MathRow,
        sub: Option<MathRow>,
        sup: Option<MathRow>,
        pre: bool,
    },
    Rad {
        degree: Option<MathRow>,
        body: MathRow,
    },
    Nary {
        chr: char,
        /// `m:limLoc`; None takes the document default (`m:intLim`/`m:naryLim`).
        under_over: Option<bool>,
        sub: Option<MathRow>,
        sup: Option<MathRow>,
        grow: bool,
        body: MathRow,
    },
    Delim {
        open: Option<char>,
        close: Option<char>,
        sep: char,
        grow: bool,
        items: Vec<MathRow>,
    },
    Func {
        name: MathRow,
        body: MathRow,
    },
    Accent {
        chr: char,
        body: MathRow,
    },
    Bar {
        top: bool,
        body: MathRow,
    },
    GroupChr {
        chr: char,
        top: bool,
        body: MathRow,
    },
    /// `m:limLow` (`upper` false) and `m:limUpp`.
    Limit {
        upper: bool,
        body: MathRow,
        limit: MathRow,
    },
    Matrix {
        rows: Vec<Vec<MathRow>>,
    },
    EqArr {
        rows: Vec<MathRow>,
    },
    Boxed(MathRow),
    BorderBox {
        /// Hidden sides: top, bottom, left, right.
        hide: [bool; 4],
        body: MathRow,
    },
    Phantom {
        show: bool,
        zero_width: bool,
        zero_ascent: bool,
        zero_descent: bool,
        body: MathRow,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FracKind {
    Bar,
    Skewed,
    Linear,
    NoBar,
}

#[derive(Debug, Clone)]
pub struct MathRun {
    pub text: String,
    /// `m:sty`; None: Word's default, italic letters and upright everything else.
    pub style: Option<MathStyle>,
    pub script: MathScript,
    /// `m:nor`: ordinary text, no math italic and no operator spacing.
    pub normal: bool,
    pub color: Option<[u8; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathStyle {
    Plain,
    Bold,
    Italic,
    BoldItalic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MathScript {
    #[default]
    Roman,
    Script,
    Fraktur,
    DoubleStruck,
    SansSerif,
    Monospace,
}

/// Document-level math properties (`m:mathPr` in settings.xml, §22.1.2.62).
#[derive(Debug, Clone)]
pub struct MathSettings {
    pub font: String,
    /// `m:smallFrac`: fractions at script size even in display math.
    pub small_frac: bool,
    /// Limits of an integral go under/over it (`m:intLim` undOvr); Word's
    /// default puts them beside it.
    pub int_under_over: bool,
    /// Limits of other n-ary operators go under/over (`m:naryLim`, default undOvr).
    pub nary_under_over: bool,
}

impl Default for MathSettings {
    fn default() -> Self {
        Self {
            font: "Cambria Math".to_string(),
            small_frac: false,
            int_under_over: false,
            nary_under_over: true,
        }
    }
}
