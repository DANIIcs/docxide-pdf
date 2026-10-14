//! Letters in math runs drawn as Mathematical Alphanumeric Symbols
//! (U+1D400–1D7FF), the way Word writes them to its PDFs: 𝑇 for an
//! unstyled T, 𝐝 for `m:sty="b"`, 𝒆 for "bi", 𝔸 for `m:scr="double-struck"`.

use crate::model::{MathScript, MathStyle};

/// The character to draw for `ch` and whether it is italic. Word's default
/// (no `m:sty`) italicises Latin letters and lowercase Greek only.
pub(super) fn styled(ch: char, style: Option<MathStyle>, script: MathScript) -> (char, bool) {
    let default_italic = ch.is_ascii_alphabetic()
        || ('α'..='ω').contains(&ch)
        || matches!(ch, 'ϑ' | 'ϕ' | 'ϖ' | 'ϱ' | 'ϵ' | 'ı' | 'ȷ');
    let (bold, italic) = match style {
        None => (false, default_italic),
        Some(MathStyle::Plain) => (false, false),
        Some(MathStyle::Bold) => (true, false),
        Some(MathStyle::Italic) => (false, true),
        Some(MathStyle::BoldItalic) => (true, true),
    };
    let mapped = match script {
        MathScript::Roman => roman(ch, bold, italic),
        _ => other_script(ch, bold, italic, script),
    };
    match mapped {
        Some(c) => (c, italic && c != ch),
        None => (ch, false),
    }
}

fn offset(base: u32, i: u32) -> Option<char> {
    char::from_u32(base + i)
}

/// Latin index 0..52 (A–Z, a–z).
fn latin(ch: char) -> Option<u32> {
    match ch {
        'A'..='Z' => Some(ch as u32 - 'A' as u32),
        'a'..='z' => Some(26 + ch as u32 - 'a' as u32),
        _ => None,
    }
}

/// Greek index in the math block's order: Α–Ω (ϴ in final sigma's slot), ∇,
/// α–ω, then ∂ ϵ ϑ ϰ ϕ ϱ ϖ.
fn greek(ch: char) -> Option<u32> {
    Some(match ch {
        'ϴ' => 17,
        '\u{391}'..='\u{3A9}' if ch != '\u{3A2}' => ch as u32 - 0x391,
        '∇' => 25,
        'α'..='ω' => 26 + ch as u32 - 'α' as u32,
        '∂' => 51,
        'ϵ' => 52,
        'ϑ' => 53,
        'ϰ' => 54,
        'ϕ' => 55,
        'ϱ' => 56,
        'ϖ' => 57,
        _ => return None,
    })
}

fn roman(ch: char, bold: bool, italic: bool) -> Option<char> {
    if !bold && !italic {
        return None;
    }
    if let Some(i) = latin(ch) {
        if italic && !bold && ch == 'h' {
            return Some('\u{210E}');
        }
        let base = match (bold, italic) {
            (true, false) => 0x1D400,
            (false, true) => 0x1D434,
            _ => 0x1D468,
        };
        return offset(base, i);
    }
    if let Some(i) = greek(ch) {
        let base = match (bold, italic) {
            (true, false) => 0x1D6A8,
            (false, true) => 0x1D6E2,
            _ => 0x1D71C,
        };
        return offset(base, i);
    }
    if bold && ch.is_ascii_digit() {
        return offset(0x1D7CE, ch as u32 - '0' as u32);
    }
    None
}

fn other_script(ch: char, bold: bool, italic: bool, script: MathScript) -> Option<char> {
    if let Some(d) = ch.to_digit(10) {
        let base = match (script, bold) {
            (MathScript::DoubleStruck, _) => 0x1D7D8,
            (MathScript::SansSerif, false) => 0x1D7E2,
            (MathScript::SansSerif, true) => 0x1D7EC,
            (MathScript::Monospace, _) => 0x1D7F6,
            _ => return None,
        };
        return offset(base, d);
    }
    let i = latin(ch)?;
    // Letters encoded before the math block (Letterlike Symbols).
    let hole = match (script, bold, ch) {
        (MathScript::Script, false, 'B') => Some('ℬ'),
        (MathScript::Script, false, 'E') => Some('ℰ'),
        (MathScript::Script, false, 'F') => Some('ℱ'),
        (MathScript::Script, false, 'H') => Some('ℋ'),
        (MathScript::Script, false, 'I') => Some('ℐ'),
        (MathScript::Script, false, 'L') => Some('ℒ'),
        (MathScript::Script, false, 'M') => Some('ℳ'),
        (MathScript::Script, false, 'R') => Some('ℛ'),
        (MathScript::Script, false, 'e') => Some('ℯ'),
        (MathScript::Script, false, 'g') => Some('ℊ'),
        (MathScript::Script, false, 'o') => Some('ℴ'),
        (MathScript::Fraktur, false, 'C') => Some('ℭ'),
        (MathScript::Fraktur, false, 'H') => Some('ℌ'),
        (MathScript::Fraktur, false, 'I') => Some('ℑ'),
        (MathScript::Fraktur, false, 'R') => Some('ℜ'),
        (MathScript::Fraktur, false, 'Z') => Some('ℨ'),
        (MathScript::DoubleStruck, _, 'C') => Some('ℂ'),
        (MathScript::DoubleStruck, _, 'H') => Some('ℍ'),
        (MathScript::DoubleStruck, _, 'N') => Some('ℕ'),
        (MathScript::DoubleStruck, _, 'P') => Some('ℙ'),
        (MathScript::DoubleStruck, _, 'Q') => Some('ℚ'),
        (MathScript::DoubleStruck, _, 'R') => Some('ℝ'),
        (MathScript::DoubleStruck, _, 'Z') => Some('ℤ'),
        _ => None,
    };
    if hole.is_some() {
        return hole;
    }
    let base = match (script, bold, italic) {
        (MathScript::Script, false, _) => 0x1D49C,
        (MathScript::Script, true, _) => 0x1D4D0,
        (MathScript::Fraktur, false, _) => 0x1D504,
        (MathScript::Fraktur, true, _) => 0x1D56C,
        (MathScript::DoubleStruck, _, _) => 0x1D538,
        (MathScript::SansSerif, false, false) => 0x1D5A0,
        (MathScript::SansSerif, true, false) => 0x1D5D4,
        (MathScript::SansSerif, false, true) => 0x1D608,
        (MathScript::SansSerif, true, true) => 0x1D63C,
        (MathScript::Monospace, _, _) => 0x1D670,
        (MathScript::Roman, ..) => return None,
    };
    offset(base, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_default_italic() {
        // Characters Word wrote for the pendulum reference's T, l, π, ϑ.
        assert_eq!(styled('T', None, MathScript::Roman), ('\u{1D447}', true));
        assert_eq!(styled('l', None, MathScript::Roman), ('\u{1D459}', true));
        assert_eq!(styled('π', None, MathScript::Roman), ('\u{1D70B}', true));
        assert_eq!(styled('ϑ', None, MathScript::Roman), ('\u{1D717}', true));
        assert_eq!(styled('h', None, MathScript::Roman), ('ℎ', true));
        assert_eq!(styled('2', None, MathScript::Roman), ('2', false));
        assert_eq!(styled('Δ', None, MathScript::Roman), ('Δ', false));
    }

    #[test]
    fn explicit_styles() {
        assert_eq!(styled('d', Some(MathStyle::Bold), MathScript::Roman).0, '𝐝');
        assert_eq!(
            styled('e', Some(MathStyle::BoldItalic), MathScript::Roman).0,
            '𝒆'
        );
        assert_eq!(
            styled('c', Some(MathStyle::Plain), MathScript::Roman).0,
            'c'
        );
        assert_eq!(styled('Δ', Some(MathStyle::Bold), MathScript::Roman).0, '𝚫');
        assert_eq!(styled('R', None, MathScript::DoubleStruck).0, 'ℝ');
        assert_eq!(styled('A', None, MathScript::DoubleStruck).0, '𝔸');
        assert_eq!(styled('L', None, MathScript::Script).0, 'ℒ');
    }
}
