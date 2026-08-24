use serde::{Deserialize, Serialize};

/// Standard graphic design units of measurement (at 96 DPI screen/CSS standard)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Unit {
    #[default]
    Px,
    Mm,
    Cm,
    M,
    In,
    Pt,
    Pc,
}

impl Unit {
    pub const ALL: [Unit; 7] = [
        Unit::Px,
        Unit::Mm,
        Unit::Cm,
        Unit::M,
        Unit::In,
        Unit::Pt,
        Unit::Pc,
    ];

    /// Conversion factor: 1 Unit in Pixels (96 DPI)
    pub fn to_px_factor(self) -> f64 {
        match self {
            Unit::Px => 1.0,
            Unit::Mm => 96.0 / 25.4,          // ≈ 3.779527559
            Unit::Cm => 96.0 / 2.54,          // ≈ 37.79527559
            Unit::M => (96.0 / 2.54) * 100.0, // ≈ 3779.527559
            Unit::In => 96.0,                 // 96.0
            Unit::Pt => 96.0 / 72.0,          // 4/3 ≈ 1.333333333
            Unit::Pc => 16.0,                 // 12 pt = 16.0 px
        }
    }

    /// Convert a value in this unit to pixels
    pub fn to_px(self, val: f64) -> f64 {
        val * self.to_px_factor()
    }

    /// Convert a value in pixels to this unit
    pub fn from_px(self, px: f64) -> f64 {
        px / self.to_px_factor()
    }

    /// Short suffix for the unit (e.g. "px", "mm", "cm")
    pub fn suffix(self) -> &'static str {
        match self {
            Unit::Px => "px",
            Unit::Mm => "mm",
            Unit::Cm => "cm",
            Unit::M => "m",
            Unit::In => "in",
            Unit::Pt => "pt",
            Unit::Pc => "pc",
        }
    }

    /// Descriptive name with suffix (e.g. "Pixels (px)")
    pub fn display_name(self) -> &'static str {
        match self {
            Unit::Px => "Pixels (px)",
            Unit::Mm => "Milímetros (mm)",
            Unit::Cm => "Centímetros (cm)",
            Unit::M => "Metros (m)",
            Unit::In => "Polegadas (in)",
            Unit::Pt => "Pontos (pt)",
            Unit::Pc => "Picas (pc)",
        }
    }

    /// Recommended decimal places for formatting
    pub fn decimals(self) -> usize {
        match self {
            Unit::Px => 1,
            Unit::Mm => 2,
            Unit::Cm => 2,
            Unit::M => 4,
            Unit::In => 2,
            Unit::Pt => 1,
            Unit::Pc => 2,
        }
    }

    /// Format a pixel value into this unit without the suffix (e.g. "120.5")
    pub fn format(self, px: f32) -> String {
        let val = self.from_px(px as f64);
        let dec = self.decimals();
        let s = format!("{:.1$}", val, dec);
        // Trim trailing zeros after decimal point for cleaner display
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_string()
        } else {
            s
        }
    }

    /// Format a pixel value into this unit with its suffix (e.g. "120.5 mm")
    #[allow(dead_code)]
    pub fn format_with_suffix(self, px: f32) -> String {
        format!("{} {}", self.format(px), self.suffix())
    }

    /// Parse unit from a suffix string
    pub fn from_suffix(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "px" | "pixel" | "pixels" => Some(Unit::Px),
            "mm" | "millimeter" | "millimeters" => Some(Unit::Mm),
            "cm" | "centimeter" | "centimeters" => Some(Unit::Cm),
            "m" | "mt" | "meter" | "meters" => Some(Unit::M),
            "in" | "inch" | "inches" | "\"" => Some(Unit::In),
            "pt" | "point" | "points" => Some(Unit::Pt),
            "pc" | "pica" | "picas" => Some(Unit::Pc),
            _ => None,
        }
    }

    /// Convert from index (0..7) for UI dropdowns
    pub fn from_index(idx: u32) -> Self {
        match idx {
            1 => Unit::Mm,
            2 => Unit::Cm,
            3 => Unit::M,
            4 => Unit::In,
            5 => Unit::Pt,
            6 => Unit::Pc,
            _ => Unit::Px,
        }
    }

    /// Convert to index (0..7) for UI dropdowns
    pub fn to_index(self) -> u32 {
        match self {
            Unit::Px => 0,
            Unit::Mm => 1,
            Unit::Cm => 2,
            Unit::M => 3,
            Unit::In => 4,
            Unit::Pt => 5,
            Unit::Pc => 6,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Math Expression Evaluator
// ─────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    default_unit: Unit,
    base_val: Option<f64>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str, default_unit: Unit, base_val: Option<f64>) -> Self {
        Self {
            chars: input.chars().peekable(),
            default_unit,
            base_val,
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
                continue;
            }
            match c {
                '+' => {
                    self.chars.next();
                    return Ok(Some(Token::Plus));
                }
                '-' => {
                    self.chars.next();
                    return Ok(Some(Token::Minus));
                }
                '*' => {
                    self.chars.next();
                    return Ok(Some(Token::Star));
                }
                '/' => {
                    self.chars.next();
                    return Ok(Some(Token::Slash));
                }
                '^' => {
                    self.chars.next();
                    return Ok(Some(Token::Caret));
                }
                '(' => {
                    self.chars.next();
                    return Ok(Some(Token::LParen));
                }
                ')' => {
                    self.chars.next();
                    return Ok(Some(Token::RParen));
                }
                '0'..='9' | '.' | ',' => {
                    return self.read_number().map(Some);
                }
                '%' => {
                    self.chars.next();
                    let base = self.base_val.unwrap_or(100.0);
                    return Ok(Some(Token::Number(base * 0.01)));
                }
                _ => {
                    return Err(format!("Caractere inesperado: '{}'", c));
                }
            }
        }
        Ok(None)
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut num_str = String::new();
        let mut has_dot = false;

        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.chars.next();
            } else if (c == '.' || c == ',') && !has_dot {
                has_dot = true;
                num_str.push('.');
                self.chars.next();
            } else {
                break;
            }
        }

        let raw_val: f64 = num_str
            .parse()
            .map_err(|_| format!("Número inválido: {}", num_str))?;

        // Read optional suffix immediately following number or after space
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }

        let mut unit_str = String::new();
        if let Some(&c) = self.chars.peek() {
            if c == '%' {
                self.chars.next();
                let base = self.base_val.unwrap_or(100.0);
                return Ok(Token::Number(base * (raw_val / 100.0)));
            }
            while let Some(&c) = self.chars.peek() {
                if c.is_alphabetic() || c == '"' {
                    unit_str.push(c);
                    self.chars.next();
                } else {
                    break;
                }
            }
        }

        let px_value = if !unit_str.is_empty() {
            if let Some(explicit_unit) = Unit::from_suffix(&unit_str) {
                explicit_unit.to_px(raw_val)
            } else {
                return Err(format!("Unidade desconhecida: '{}'", unit_str));
            }
        } else {
            // Use default unit
            self.default_unit.to_px(raw_val)
        };

        Ok(Token::Number(px_value))
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn parse_expression(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;

        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.next();
                    let right = self.parse_term()?;
                    left += right;
                }
                Token::Minus => {
                    self.next();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_power()?;

        while let Some(tok) = self.peek() {
            match tok {
                Token::Star => {
                    self.next();
                    let right = self.parse_power()?;
                    left *= right;
                }
                Token::Slash => {
                    self.next();
                    let right = self.parse_power()?;
                    if right.abs() < 1e-12 {
                        return Err("Divisão por zero".to_string());
                    }
                    left /= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<f64, String> {
        let left = self.parse_unary()?;

        if let Some(Token::Caret) = self.peek() {
            self.next();
            let right = self.parse_power()?; // Right-associative
            Ok(left.powf(right))
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<f64, String> {
        if let Some(Token::Minus) = self.peek() {
            self.next();
            let val = self.parse_unary()?;
            Ok(-val)
        } else if let Some(Token::Plus) = self.peek() {
            self.next();
            self.parse_unary()
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::LParen) => {
                let val = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(val),
                    _ => Err("Parêntese ')' não fechado".to_string()),
                }
            }
            Some(tok) => Err(format!("Token inesperado: {:?}", tok)),
            None => Err("Fim de expressão inesperado".to_string()),
        }
    }
}

/// Evaluates a user input string (math expression + optional units) into world pixels.
///
/// # Examples
/// ```
/// use gnome_paths::core::{eval_math_expression, Unit};
/// assert_eq!(eval_math_expression("100 + 50", Unit::Px, None).unwrap(), 150.0);
/// assert_eq!(eval_math_expression("10 * 5", Unit::Px, None).unwrap(), 50.0);
/// ```
pub fn eval_math_expression(
    input: &str,
    default_unit: Unit,
    base_val: Option<f32>,
) -> Result<f32, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Entrada vazia".to_string());
    }

    let base_f64 = base_val.map(|v| v as f64);
    let mut lexer = Lexer::new(trimmed, default_unit, base_f64);
    let mut tokens = Vec::new();

    while let Some(tok) = lexer.next_token()? {
        tokens.push(tok);
    }

    if tokens.is_empty() {
        return Err("Expressão vazia".to_string());
    }

    let mut parser = Parser::new(tokens);
    let result = parser.parse_expression()?;

    if parser.pos < parser.tokens.len() {
        return Err("Tokens adicionais após expressão válida".to_string());
    }

    if result.is_nan() || result.is_infinite() {
        return Err("Resultado numérico inválido (NaN/Inf)".to_string());
    }

    Ok(result as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversions() {
        let mm_in_px = Unit::Mm.to_px(25.4);
        assert!((mm_in_px - 96.0).abs() < 1e-4);

        let in_in_px = Unit::In.to_px(1.0);
        assert!((in_in_px - 96.0).abs() < 1e-4);

        let cm_in_px = Unit::Cm.to_px(2.54);
        assert!((cm_in_px - 96.0).abs() < 1e-4);

        let pt_in_px = Unit::Pt.to_px(72.0);
        assert!((pt_in_px - 96.0).abs() < 1e-4);

        let pc_in_px = Unit::Pc.to_px(6.0);
        assert!((pc_in_px - 96.0).abs() < 1e-4);

        let m_in_px = Unit::M.to_px(1.0);
        assert!((m_in_px - 3779.5276).abs() < 0.1);
    }

    #[test]
    fn test_eval_simple_math() {
        assert_eq!(
            eval_math_expression("100 + 50", Unit::Px, None).unwrap(),
            150.0
        );
        assert_eq!(
            eval_math_expression("200 - 35", Unit::Px, None).unwrap(),
            165.0
        );
        assert_eq!(
            eval_math_expression("50 * 3", Unit::Px, None).unwrap(),
            150.0
        );
        assert_eq!(
            eval_math_expression("1920 / 2", Unit::Px, None).unwrap(),
            960.0
        );
        assert_eq!(eval_math_expression("2 ^ 3", Unit::Px, None).unwrap(), 8.0);
    }

    #[test]
    fn test_eval_operator_precedence_and_parentheses() {
        assert_eq!(
            eval_math_expression("10 + 20 * 2", Unit::Px, None).unwrap(),
            50.0
        );
        assert_eq!(
            eval_math_expression("(10 + 20) * 2", Unit::Px, None).unwrap(),
            60.0
        );
        assert_eq!(
            eval_math_expression("100 / (2 + 3)", Unit::Px, None).unwrap(),
            20.0
        );
        assert_eq!(
            eval_math_expression("-50 + 20", Unit::Px, None).unwrap(),
            -30.0
        );
    }

    #[test]
    fn test_eval_units_and_mixed_units() {
        // 1 in + 0 in = 96 px
        let res = eval_math_expression("1in", Unit::Px, None).unwrap();
        assert!((res - 96.0).abs() < 1e-4);

        // 25.4mm + 10px = 96 + 10 = 106 px
        let res = eval_math_expression("25.4mm + 10px", Unit::Px, None).unwrap();
        assert!((res - 106.0).abs() < 1e-3);

        // In mm mode: 10 + 10 = 20mm in px
        let res = eval_math_expression("10 + 10", Unit::Mm, None).unwrap();
        let expected = Unit::Mm.to_px(20.0) as f32;
        assert!((res - expected).abs() < 1e-4);
    }

    #[test]
    fn test_eval_percentage() {
        // 50% with base 200 = 100
        let res = eval_math_expression("50%", Unit::Px, Some(200.0)).unwrap();
        assert_eq!(res, 100.0);

        // 200 + 20% of 200 = 240
        let res = eval_math_expression("200 + 20%", Unit::Px, Some(200.0)).unwrap();
        assert_eq!(res, 240.0);
    }

    #[test]
    fn test_unit_formatting() {
        assert_eq!(Unit::Px.format(100.0), "100");
        assert_eq!(Unit::Px.format(100.5), "100.5");
        assert_eq!(Unit::Mm.format(96.0), "25.4");
        assert_eq!(Unit::Cm.format(96.0), "2.54");
        assert_eq!(Unit::In.format(96.0), "1");
    }
}
