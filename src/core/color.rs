use skia_safe as skia;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.2, 0.55, 0.95, 1.0);
}

#[cfg(test)]
impl Color {
    pub const RED: Self = Self::new(0.95, 0.25, 0.25, 1.0);
    pub const EMERALD: Self = Self::new(0.18, 0.8, 0.44, 1.0);
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn to_skia(self) -> skia::Color4f {
        skia::Color4f::new(self.r, self.g, self.b, self.a)
    }

    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub fn to_hex(self) -> String {
        let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    pub fn to_hex_rgba(self) -> String {
        let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (self.a.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    }

    pub fn from_hsv(h: f32, s: f32, v: f32, a: f32) -> Self {
        let h = ((h % 360.0) + 360.0) % 360.0;
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: r1 + m,
            g: g1 + m,
            b: b1 + m,
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn to_hsv(self) -> (f32, f32, f32) {
        let r = self.r.clamp(0.0, 1.0);
        let g = self.g.clamp(0.0, 1.0);
        let b = self.b.clamp(0.0, 1.0);

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let v = max;
        let s = if max > 1e-5 { delta / max } else { 0.0 };

        let h = if delta < 1e-5 {
            0.0
        } else if (max - r).abs() < 1e-5 {
            60.0 * (((g - b) / delta) % 6.0)
        } else if (max - g).abs() < 1e-5 {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };
        (h, s, v)
    }

    #[allow(dead_code)]
    pub fn to_gdk(self) -> gtk4::gdk::RGBA {
        gtk4::gdk::RGBA::builder()
            .red(self.r)
            .green(self.g)
            .blue(self.b)
            .alpha(self.a)
            .build()
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('#');
        if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Self::from_rgb_u8(r, g, b))
        } else if s.len() == 8 {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some(Self::from_rgba_u8(r, g, b, a))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsv_rgb_roundtrip() {
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let (h, s, v) = red.to_hsv();
        assert!((h - 0.0).abs() < 1e-3);
        assert!((s - 1.0).abs() < 1e-3);
        assert!((v - 1.0).abs() < 1e-3);

        let red_back = Color::from_hsv(h, s, v, 1.0);
        assert!((red_back.r - 1.0).abs() < 1e-3);
        assert!((red_back.g - 0.0).abs() < 1e-3);
        assert!((red_back.b - 0.0).abs() < 1e-3);

        let green = Color::new(0.0, 1.0, 0.0, 1.0);
        let (hg, sg, vg) = green.to_hsv();
        assert!((hg - 120.0).abs() < 1e-3);
        assert!((sg - 1.0).abs() < 1e-3);
        assert!((vg - 1.0).abs() < 1e-3);

        let blue = Color::new(0.0, 0.0, 1.0, 1.0);
        let (hb, sb, vb) = blue.to_hsv();
        assert!((hb - 240.0).abs() < 1e-3);
        assert!((sb - 1.0).abs() < 1e-3);
        assert!((vb - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_hex_rgba_conversions() {
        let col = Color::new(1.0, 0.5, 0.25, 0.8);
        let hex = col.to_hex_rgba();
        assert_eq!(hex.len(), 9);
        assert!(hex.starts_with('#'));

        let parsed = Color::from_hex(&hex).unwrap();
        assert!((parsed.r - 1.0).abs() < 0.01);
        assert!((parsed.g - 0.5).abs() < 0.01);
        assert!((parsed.b - 0.25).abs() < 0.01);
        assert!((parsed.a - 0.8).abs() < 0.02);
    }
}
