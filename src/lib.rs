use std::collections::HashMap;
use std::rc::Rc;

use image::{Rgb, RgbImage};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Blue,
    Green,
    Red,
    White,
    Yellow,
    Black,
    Rgb(Rgb<u8>),
}

impl Color {
    pub fn to_rgb(&self) -> Rgb<u8> {
        match self {
            Color::Blue => [0, 0, 255].into(),
            Color::Green => [0, 255, 0].into(),
            Color::Red => [255, 0, 0].into(),
            Color::White => [255, 255, 255].into(),
            Color::Yellow => [255, 255, 0].into(),
            Color::Black => [0, 0, 0].into(),
            Color::Rgb(rgb) => *rgb,
        }
    }
}

fn to_hex_color(input: &str) -> Option<Rgb<u8>> {
    if input.len() != 7 {
        return None;
    }
    if !input.starts_with("#") {
        return None;
    }

    let hex = input[1..].chars()
        .filter_map(|x| x.to_digit(16))
        .map(|x| x as u8)
        .collect::<Vec<_>>();
    if hex.len() != 6 {
        return None;
    }

    Some([(hex[0] << 4) | hex[1], (hex[2] << 4) | hex[3], (hex[4] << 4) | hex[5]].into())
}

impl std::str::FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "b" => Ok(Color::Blue),
            "g" => Ok(Color::Green),
            "r" => Ok(Color::Red),
            "w" => Ok(Color::White),
            "y" => Ok(Color::Yellow),
            "s" => Ok(Color::Black),
            _ if to_hex_color(s).is_some() => to_hex_color(s).ok_or(()).map(Color::Rgb),
            _ => Err(()),
        }
    }
}

pub trait MsPaint {
    fn rectangle(&mut self, left: u32, top: u32, width: u32, height: u32, color: &Color);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

impl MsPaint for RgbImage {
    fn rectangle(&mut self, left: u32, top: u32, width: u32, height: u32, color: &Color) {
        for y in top..(top + height) {
            for x in left..(left + width) {
                self[(x, y)] = color.to_rgb();
            }
        }
    }

    fn width(&self) -> u32 {
        RgbImage::width(self)
    }

    fn height(&self) -> u32 {
        RgbImage::height(self)
    }
}

#[derive(Debug)]
pub struct UnresolvedFlagElement(UnresolvedFlagGeometry, u32);

impl UnresolvedFlagElement {
    pub fn resolve(
        &self,
        tags: &HashMap<String, Rc<UnresolvedFlagGeometry>>,
    ) -> Option<FlagElement> {
        Some(FlagElement(self.0.resolve(tags)?, self.1))
    }
}

#[derive(Debug)]
pub enum UnresolvedFlagGeometry {
    Solid(Color),
    Horizontal(Vec<UnresolvedFlagElement>),
    Vertical(Vec<UnresolvedFlagElement>),
    Tag(String, Rc<UnresolvedFlagGeometry>),
    Reference(String),
}

impl UnresolvedFlagGeometry {
    pub fn tags(&self) -> HashMap<String, Rc<UnresolvedFlagGeometry>> {
        let mut map = HashMap::new();
        match self {
            UnresolvedFlagGeometry::Tag(tag, geo) => {
                map.extend(geo.tags());
                map.insert(tag.clone(), geo.clone());
            }
            UnresolvedFlagGeometry::Horizontal(elements)
            | UnresolvedFlagGeometry::Vertical(elements) => {
                map.extend(elements.iter().flat_map(|el| el.0.tags().into_iter()));
            }
            _ => {}
        }

        map
    }

    pub fn resolve(
        &self,
        tags: &HashMap<String, Rc<UnresolvedFlagGeometry>>,
    ) -> Option<FlagGeometry> {
        match self {
            UnresolvedFlagGeometry::Solid(color) => Some(FlagGeometry::Solid(*color)),
            UnresolvedFlagGeometry::Horizontal(elements) => Some(FlagGeometry::Horizontal(
                elements
                    .iter()
                    .filter_map(|x| x.resolve(tags))
                    .collect::<Vec<_>>(),
            )),
            UnresolvedFlagGeometry::Vertical(elements) => Some(FlagGeometry::Vertical(
                elements
                    .iter()
                    .filter_map(|x| x.resolve(tags))
                    .collect::<Vec<_>>(),
            )),
            UnresolvedFlagGeometry::Tag(_, geo) => geo.resolve(tags),
            UnresolvedFlagGeometry::Reference(tag) => tags.get(tag).and_then(|x| x.resolve(tags)),
        }
    }
}

#[derive(Debug)]
pub struct FlagElement(FlagGeometry, u32);

#[derive(Debug)]
pub enum FlagGeometry {
    Solid(Color),
    Horizontal(Vec<FlagElement>),
    Vertical(Vec<FlagElement>),
}

impl FlagGeometry {
    fn draw_area<P: MsPaint>(&self, buffer: &mut P, left: u32, top: u32, width: u32, height: u32) {
        match self {
            FlagGeometry::Solid(color) => {
                buffer.rectangle(left, top, width, height, color);
            }
            FlagGeometry::Horizontal(elements) => {
                let total: u32 = elements.iter().map(|x| x.1).sum();
                let mut offset = left;
                for FlagElement(geo, pivot) in elements {
                    let element_width = (pivot * width) / total;
                    geo.draw_area(buffer, offset, top, element_width, height);
                    offset += element_width;
                }
            }
            FlagGeometry::Vertical(elements) => {
                let total: u32 = elements.iter().map(|x| x.1).sum();
                let mut offset = top;
                for FlagElement(geo, pivot) in elements {
                    let element_height = (pivot * height) / total;
                    geo.draw_area(buffer, left, offset, width, element_height);
                    offset += element_height;
                }
            }
        }
    }

    pub fn draw<P: MsPaint>(&self, buffer: &mut P) {
        self.draw_area(buffer, 0, 0, buffer.width(), buffer.height());
    }
}

#[derive(Debug)]
pub enum SExpr {
    List(Vec<SExpr>),
    Literal(String),
}

impl SExpr {
    pub fn parse<I>(input: &mut std::iter::Peekable<I>) -> Option<SExpr>
    where
        I: Iterator<Item = char>,
    {
        let mut sexpr = None;
        while let Some(c) = input.peek() {
            match sexpr {
                None if c.is_whitespace() => {}
                None if *c == '(' => {
                    sexpr = Some(SExpr::List(Vec::new()));
                }
                None => {
                    sexpr = Some(SExpr::Literal(c.to_string()));
                }
                Some(SExpr::List(_)) if *c == ')' => {
                    input.next();
                    break;
                }
                Some(SExpr::List(ref mut list)) => {
                    list.push(SExpr::parse(input)?);
                    continue;
                }
                Some(SExpr::Literal(_)) if c.is_whitespace() || *c == ')' => {
                    break;
                }
                Some(SExpr::Literal(ref mut literal)) => literal.push(*c),
            }

            input.next();
        }

        while let Some(c) = input.peek() {
            if c.is_whitespace() {
                input.next();
            } else {
                break;
            }
        }

        sexpr
    }

    pub fn list(&self) -> Option<&[SExpr]> {
        match self {
            SExpr::List(list) => Some(list.as_slice()),
            SExpr::Literal(_) => None,
        }
    }

    pub fn literal(&self) -> Option<&str> {
        match self {
            SExpr::Literal(literal) => Some(literal.as_str()),
            SExpr::List(_) => None,
        }
    }

    pub fn to_flag_geometry(&self) -> Option<UnresolvedFlagGeometry> {
        let list = self.list()?;

        match list {
            [op, c] if op.literal()? == "s" => {
                let color = c.literal().and_then(|lit| lit.parse().ok())?;
                Some(UnresolvedFlagGeometry::Solid(color))
            }
            [op, ..] if op.literal()? == "h" || op.literal()? == "v" => {
                let weights = list[1..]
                    .iter()
                    .step_by(2)
                    .filter_map(|x| x.literal())
                    .filter_map(|x| x.parse().ok());
                let geos = list[2..].iter().step_by(2);

                let mut elements = Vec::new();
                for (weight, geo) in weights.zip(geos) {
                    let geo = geo.to_flag_geometry()?;
                    elements.push(UnresolvedFlagElement(geo, weight));
                }

                match op.literal()? {
                    "h" => Some(UnresolvedFlagGeometry::Horizontal(elements)),
                    "v" => Some(UnresolvedFlagGeometry::Vertical(elements)),
                    _ => None,
                }
            }
            [op, ..] if op.literal()? == "v" => unimplemented!(),
            [op, tag, geo] if op.literal()? == "t" => {
                let tag = tag.literal()?.to_string();
                let geo = Rc::new(geo.to_flag_geometry()?);
                Some(UnresolvedFlagGeometry::Tag(tag, geo))
            }
            [op, tag] if op.literal()? == "r" => {
                let tag = tag.literal()?.to_string();
                Some(UnresolvedFlagGeometry::Reference(tag))
            }
            _ => {
                eprintln!("{:?}", list);
                None
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_sexpr_parse() {
        println!(
            "{:#?}",
            SExpr::parse(&mut "(h 33 (s b) (h 50 (s w) (s r)))".chars().peekable()).unwrap()
        );
    }
}
