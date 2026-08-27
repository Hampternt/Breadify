//! The Matvare Expressen mark, as vector paths.
//!
//! The supplied SVG is eighteen filled paths in two colours and nothing else —
//! no text, no transforms, no gradients — so it is read here directly rather
//! than through an SVG engine. Keeping it vector matters: the mark sits at
//! 26 mm on the masthead of every sheet, and a rasterised logo is the first
//! thing that looks cheap on paper.

use std::sync::LazyLock;

use crate::page::Colour;

/// A point in the artwork's own coordinate space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    /// Whether this is a bezier control point rather than a point on the
    /// curve. Cubic segments appear as on-curve, control, control, on-curve.
    pub control: bool,
}

/// One closed outline. A shape with a hole in it — the counter of an `a` —
/// has more than one.
pub type Ring = Vec<Vertex>;

/// One filled path of the mark.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub fill: Colour,
    pub rings: Vec<Ring>,
}

/// A piece of artwork: its own coordinate space, and what to fill.
#[derive(Debug, Clone, PartialEq)]
pub struct Artwork {
    pub width: f64,
    pub height: f64,
    pub shapes: Vec<Shape>,
}

const WORDMARK_SVG: &str = include_str!("../assets/matvare-expressen.svg");

static WORDMARK: LazyLock<Artwork> = LazyLock::new(|| parse(WORDMARK_SVG));

/// The mark, parsed once.
pub fn wordmark() -> &'static Artwork {
    &WORDMARK
}

/// Reads the paths out of an SVG of the shape this one has.
///
/// Understands `M`, `L`, `H`, `V`, `C` and `Z` in absolute form, which is
/// everything the mark uses. Anything else is skipped rather than guessed at.
fn parse(svg: &str) -> Artwork {
    let (width, height) = viewbox(svg).unwrap_or((166.0, 40.0));
    let shapes = svg
        .split("<path")
        .skip(1)
        .filter_map(|element| {
            let rings = rings(&attribute(element, "d")?);
            (!rings.is_empty()).then(|| Shape {
                fill: fill(&attribute(element, "fill").unwrap_or_default()),
                rings,
            })
        })
        .collect();

    Artwork {
        width,
        height,
        shapes,
    }
}

fn viewbox(svg: &str) -> Option<(f64, f64)> {
    let box_values = attribute(svg, "viewBox")?;
    let mut parts = box_values.split_whitespace().skip(2);
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

/// The value of an attribute, from the first element that carries it.
fn attribute(element: &str, name: &str) -> Option<String> {
    let start = element.find(&format!("{name}=\""))? + name.len() + 2;
    let rest = &element[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

/// `#FF4F46` or a colour name. Only the two the mark uses are recognised.
fn fill(value: &str) -> Colour {
    if let Some(hex) = value.strip_prefix('#')
        && hex.len() == 6
        && let (Ok(red), Ok(green), Ok(blue)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        )
    {
        return Colour::rgb(red, green, blue);
    }
    match value {
        "black" | "none" => Colour::grey(0),
        _ => Colour::grey(0xFF),
    }
}

/// Turns one `d` attribute into closed outlines.
fn rings(data: &str) -> Vec<Ring> {
    let mut rings: Vec<Ring> = Vec::new();
    let mut ring: Ring = Vec::new();
    let mut at = (0.0, 0.0);
    let mut tokens = tokenise(data).into_iter().peekable();

    while let Some(token) = tokens.next() {
        let Token::Command(command) = token else {
            continue;
        };

        loop {
            match command {
                'M' | 'L' => {
                    let Some((x, y)) = pair(&mut tokens) else {
                        break;
                    };
                    if command == 'M' && !ring.is_empty() {
                        rings.push(std::mem::take(&mut ring));
                    }
                    at = (x, y);
                    ring.push(on_curve(at));
                }
                'H' => {
                    let Some(x) = number(&mut tokens) else {
                        break;
                    };
                    at = (x, at.1);
                    ring.push(on_curve(at));
                }
                'V' => {
                    let Some(y) = number(&mut tokens) else {
                        break;
                    };
                    at = (at.0, y);
                    ring.push(on_curve(at));
                }
                'C' => {
                    let (Some(first), Some(second), Some(end)) =
                        (pair(&mut tokens), pair(&mut tokens), pair(&mut tokens))
                    else {
                        break;
                    };
                    ring.push(control(first));
                    ring.push(control(second));
                    at = end;
                    ring.push(on_curve(at));
                }
                'Z' => {
                    if !ring.is_empty() {
                        rings.push(std::mem::take(&mut ring));
                    }
                    break;
                }
                _ => break,
            }

            // A command repeats while numbers keep following it.
            if !matches!(tokens.peek(), Some(Token::Number(_))) {
                break;
            }
        }
    }

    if !ring.is_empty() {
        rings.push(ring);
    }
    rings
}

fn on_curve((x, y): (f64, f64)) -> Vertex {
    Vertex {
        x,
        y,
        control: false,
    }
}

fn control((x, y): (f64, f64)) -> Vertex {
    Vertex {
        x,
        y,
        control: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Command(char),
    Number(f64),
}

fn number(tokens: &mut std::iter::Peekable<std::vec::IntoIter<Token>>) -> Option<f64> {
    match tokens.peek() {
        Some(Token::Number(value)) => {
            let value = *value;
            tokens.next();
            Some(value)
        }
        _ => None,
    }
}

fn pair(tokens: &mut std::iter::Peekable<std::vec::IntoIter<Token>>) -> Option<(f64, f64)> {
    Some((number(tokens)?, number(tokens)?))
}

/// Splits path data into commands and numbers. SVG allows numbers to run
/// together (`1.5.25` is two numbers) and to be separated by anything.
fn tokenise(data: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut number = String::new();

    let flush = |number: &mut String, tokens: &mut Vec<Token>| {
        if let Ok(value) = number.parse::<f64>() {
            tokens.push(Token::Number(value));
        }
        number.clear();
    };

    for character in data.chars() {
        match character {
            '0'..='9' => number.push(character),
            '-' | '+' => {
                if !number.is_empty() && !number.ends_with(['e', 'E']) {
                    flush(&mut number, &mut tokens);
                }
                number.push(character);
            }
            '.' => {
                if number.contains('.') {
                    flush(&mut number, &mut tokens);
                }
                number.push(character);
            }
            'e' | 'E' if !number.is_empty() => number.push(character),
            letter if letter.is_ascii_alphabetic() => {
                flush(&mut number, &mut tokens);
                tokens.push(Token::Command(letter.to_ascii_uppercase()));
            }
            _ => flush(&mut number, &mut tokens),
        }
    }
    flush(&mut number, &mut tokens);
    tokens
}
