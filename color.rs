use std::collections::HashMap;
use std::sync::RwLock;

use unicode_segmentation::UnicodeSegmentation;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        let hex = match hex.len() {
            3 => hex.chars().flat_map(|c| std::iter::repeat(c).take(2)).collect::<String>(),
            6 => hex.to_string(),
            _ => return Color { r: 0, g: 0, b: 0 },
        };

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        Color { r, g, b }
    }
}

lazy_static! {
    pub static ref COLORS: RwLock<HashMap<String, HashMap<String, Color>>> = {
        let mut map = HashMap::new();
        map.insert("default".to_string(), HashMap::from([
            ("red".to_string(), Color { r: 255, g: 0, b: 0 }),
            ("green".to_string(), Color { r: 0, g: 255, b: 0 }),
            ("blue".to_string(), Color { r: 0, g: 0, b: 255 }),
            ("yellow".to_string(), Color { r: 255, g: 255, b: 0 }),
            ("cyan".to_string(), Color { r: 0, g: 255, b: 255 }),
            ("magenta".to_string(), Color { r: 255, g: 0, b: 255 }),
            ("black".to_string(), Color { r: 0, g: 0, b: 0 }),
            ("white".to_string(), Color { r: 255, g: 255, b: 255 }),
            ("gray".to_string(), Color { r: 128, g: 128, b: 128 }),
            ("light_red".to_string(), Color { r: 255, g: 102, b: 102 }),
            ("light_green".to_string(), Color { r: 102, g: 255, b: 102 }),
            ("light_blue".to_string(), Color { r: 102, g: 102, b: 255 }),
            ("light_yellow".to_string(), Color { r: 255, g: 255, b: 102 }),
            ("light_cyan".to_string(), Color { r: 102, g: 255, b: 255 }),
            ("light_magenta".to_string(), Color { r: 255, g: 102, b: 255 }),
            ("light_gray".to_string(), Color { r: 211, g: 211, b: 211 }),
            ("dark_red".to_string(), Color { r: 139, g: 0, b: 0 }),
            ("dark_green".to_string(), Color { r: 0, g: 100, b: 0 }),
            ("dark_blue".to_string(), Color { r: 0, g: 0, b: 139 }),
            ("dark_yellow".to_string(), Color { r: 139, g: 139, b: 0 }),
            ("dark_cyan".to_string(), Color { r: 0, g: 139, b: 139 }),
            ("dark_magenta".to_string(), Color { r: 139, g: 0, b: 139 }),
            ("dark_gray".to_string(), Color { r: 64, g: 64, b: 64 }),
        ]));
        RwLock::new(map)
    };
}

#[derive(Clone, Debug)]
pub enum ColorRef<'a> {
    Direct(Color),
    Named(&'a str, &'a str),
}

fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| matches!(c, 'a'..='z' | '_'))
}

pub fn resolve_color_ref(c: &ColorRef) -> Option<Color> {
    match c {
        ColorRef::Direct(color) => Some(*color),
        ColorRef::Named(ns, name) => {
            if !is_valid_identifier(ns) || !is_valid_identifier(name) {
                return None;
            }
            let map = COLORS.read().ok()?;
            map.get(*ns)?.get(*name).copied()
        }
    }
}

pub fn add_color(namespace: &str, name: &str, c: Color) -> Result<(), String> {
    if namespace == "default" {
        return Err("cannot modify default or pastel namespace".into());
    }
    if !is_valid_identifier(namespace) {
        return Err("namespace must be lowercase and contain only [a-z_]".into());
    }
    if !is_valid_identifier(name) {
        return Err("name must be lowercase and contain only [a-z_]".into());
    }

    let mut colors = COLORS.write().unwrap();
    let ns_entry = colors.entry(namespace.to_string()).or_default();

    if ns_entry.contains_key(name) {
        panic!("color '{}::{}' already exists - use change_color() instead", namespace, name);
    }

    ns_entry.insert(name.to_string(), c);
    Ok(())
}

pub fn remove_color(namespace: &str, name: &str) -> Result<(), String> {
    if namespace == "default" {
        return Err("cannot modify default or pastel namespace".into());
    }
    if !is_valid_identifier(namespace) {
        return Err("namespace must be lowercase and contain only [a-z_]".into());
    }
    if !is_valid_identifier(name) {
        return Err("name must be lowercase and contain only [a-z_]".into());
    }
    let mut colors = COLORS.write().unwrap();
    if let Some(ns) = colors.get_mut(namespace) {
        if ns.remove(name).is_none() {
            return Err("color does not exist in namespace".into());
        }
        if ns.is_empty() {
            colors.remove(namespace);
        }
        Ok(())
    } else {
        Err("namespace does not exist".into())
    }
}

pub fn change_color(namespace: &str, name: &str, c: Color) -> Result<(), String> {
    if namespace == "default" {
        return Err("cannot modify default or pastel namespace".into());
    }
    if !is_valid_identifier(namespace) {
        return Err("namespace must be lowercase and contain only [a-z_]".into());
    }
    if !is_valid_identifier(name) {
        return Err("name must be lowercase and contain only [a-z_]".into());
    }
    let mut colors = COLORS.write().unwrap();
    if let Some(ns) = colors.get_mut(namespace) {
        if ns.contains_key(name) {
            ns.insert(name.to_string(), c);
            Ok(())
        } else {
            Err("color does not exist in namespace".into())
        }
    } else {
        Err("namespace does not exist".into())
    }
}

fn interpolate_multi_color(colors: &[Color], factor: f64) -> Color {
    if factor <= 0.0 {
        return colors[0];
    }
    if factor >= 1.0 {
        return *colors.last().unwrap();
    }
    let total = colors.len() - 1;
    let scaled = factor * total as f64;
    let index = scaled.floor() as usize;
    let inner_fac = scaled - index as f64;

    let start = colors[index];
    let end = colors[index + 1];

    Color {
        r: (start.r as f64 + (end.r as f64 - start.r as f64) * inner_fac) as u8,
        g: (start.g as f64 + (end.g as f64 - start.g as f64) * inner_fac) as u8,
        b: (start.b as f64 + (end.b as f64 - start.b as f64) * inner_fac) as u8,
    }
}

fn apply_gradient(lines: &[&str], colors: &[Color]) -> Vec<String> {
    let total = (lines.len() - 1).max(1) as f32;

    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let pos = i as f32 / total;
            let color = interpolate_multi_color(colors, pos as f64);
            format!(
                "\x1b[38;2;{};{};{}m{}",
                color.r, color.g, color.b, line
            )
        })
        .collect()
}

pub enum GradientDirection {
    Horizontal,
    Vertical,
}

fn apply_gradient_fixed_len(
    graphemes: &[&str],
    colors: &[Color],
    target_len: usize,
) -> String {
    let mut result = String::with_capacity(graphemes.len() * 10);
    let range = (target_len - 1).max(1) as f32;

    for (i, grapheme) in graphemes.iter().enumerate() {
        let pos = i as f32 / range;
        let color = interpolate_multi_color(colors, pos as f64);
        result.push_str(&format!(
            "\x1b[38;2;{};{};{}m{}",
            color.r, color.g, color.b, grapheme
        ));
    }

    result.push_str("\x1b[0m");
    result
}

pub fn gradient_text(
    text: &str,
    color_refs: &[ColorRef],
    direction: GradientDirection,
    align_gradient: Option<bool>,
) -> Result<String, String> {
    if color_refs.len() < 2 {
        return Err("at least two colors are required".into());
    }

    let rgb_colors: Vec<_> = color_refs
        .iter()
        .map(|c| resolve_color_ref(c).ok_or("could not resolve all colors"))
        .collect::<Result<_, _>>()?;

    let lines: Vec<&str> = text.lines().collect();

    match direction {
        GradientDirection::Vertical => {
            if align_gradient.is_some() {
                return Err("align_gradient must be None for vertical gradients".into());
            }

            let colored_lines = apply_gradient(&lines, &rgb_colors)
                .into_iter()
                .map(|l| l + "\x1b[0m")
                .collect::<Vec<_>>();

            Ok(colored_lines.join("\n"))
        }
        GradientDirection::Horizontal => {
            let align = align_gradient.unwrap_or(false);

            let max_len = if align {
                lines
                    .iter()
                    .map(|l| strip_ansi_codes(l).graphemes(true).count())
                    .max()
                    .unwrap_or(0)
            } else {
                0
            };

            let result = lines
                .iter()
                .map(|line| {
                    let graphemes: Vec<&str> = line.graphemes(true).collect();
                    let gradient_basis = if align {
                        max_len
                    } else {
                        graphemes.len()
                    };
                    apply_gradient_fixed_len(&graphemes, &rgb_colors, gradient_basis)
                })
                .collect::<Vec<_>>();

            Ok(result.join("\n"))
        }
    }
}

pub fn rainbow_text(
    text: &str,
    direction: GradientDirection,
    align_gradient: Option<bool>,
) -> Result<String, String> {
    let rainbow: Vec<ColorRef> = vec![
        ColorRef::Direct(Color::from_hex("#ff0000")),
        ColorRef::Direct(Color::from_hex("#ff7f00")),
        ColorRef::Direct(Color::from_hex("#ffff00")),
        ColorRef::Direct(Color::from_hex("#00ff00")),
        ColorRef::Direct(Color::from_hex("#0000ff")),
        ColorRef::Direct(Color::from_hex("#4b0082")),
        ColorRef::Direct(Color::from_hex("#9400d3")),
    ];
    gradient_text(text, &rainbow, direction, align_gradient)
}

pub fn colored_text(
    text: &str,
    color_ref: &ColorRef,
) -> Result<String, String> {
    let color = resolve_color_ref(color_ref)
        .ok_or("could not resolve color reference")?;
    Ok(format!(
        "\x1b[38;2;{};{};{}m{}\x1b[0m",
        color.r, color.g, color.b, text
    ))
}

static ANSI_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\x1B\[[0-9;]*m").unwrap());

pub fn strip_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

pub fn visible_length(s: &str) -> usize {
    strip_ansi_codes(s).graphemes(true).count()
}