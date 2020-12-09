use std::collections::HashMap;
use std::env::args;
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
        }
    }
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
pub enum UnresolvedFlagGeometry {
    Solid(Color),
    Horizontal(Rc<UnresolvedFlagGeometry>, Rc<UnresolvedFlagGeometry>, u32),
    Vertical(Rc<UnresolvedFlagGeometry>, Rc<UnresolvedFlagGeometry>, u32),
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
            UnresolvedFlagGeometry::Horizontal(car, cdr, ..)
            | UnresolvedFlagGeometry::Vertical(car, cdr, ..) => {
                map.extend(car.tags());
                map.extend(cdr.tags());
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
            UnresolvedFlagGeometry::Horizontal(car, cdr, pivot) => Some(FlagGeometry::Horizontal(
                Rc::new(car.resolve(tags)?),
                Rc::new(cdr.resolve(tags)?),
                *pivot,
            )),
            UnresolvedFlagGeometry::Vertical(car, cdr, pivot) => Some(FlagGeometry::Vertical(
                Rc::new(car.resolve(tags)?),
                Rc::new(cdr.resolve(tags)?),
                *pivot,
            )),
            UnresolvedFlagGeometry::Tag(_, geo) => geo.resolve(tags),
            UnresolvedFlagGeometry::Reference(tag) => tags.get(tag).and_then(|x| x.resolve(tags)),
        }
    }
}

#[derive(Debug)]
pub enum FlagGeometry {
    Solid(Color),
    Horizontal(Rc<FlagGeometry>, Rc<FlagGeometry>, u32),
    Vertical(Rc<FlagGeometry>, Rc<FlagGeometry>, u32),
}

impl FlagGeometry {
    fn draw_area<P: MsPaint>(&self, buffer: &mut P, left: u32, top: u32, width: u32, height: u32) {
        match self {
            FlagGeometry::Solid(color) => {
                buffer.rectangle(left, top, width, height, color);
            }
            FlagGeometry::Horizontal(car, cdr, pivot) => {
                car.draw_area(buffer, left, top, (pivot * width) / 100, height);
                cdr.draw_area(
                    buffer,
                    left + (pivot * width) / 100,
                    top,
                    ((100 - pivot) * width) / 100,
                    height,
                );
            }
            FlagGeometry::Vertical(car, cdr, pivot) => {
                car.draw_area(buffer, left, top, width, (pivot * height) / 100);
                cdr.draw_area(
                    buffer,
                    left,
                    top + (pivot * height) / 100,
                    width,
                    ((100 - pivot) * height) / 100,
                );
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
            [op, pivot, car, cdr] => {
                let pivot = pivot.literal().and_then(|lit| lit.parse().ok())?;
                let car = Rc::new(car.to_flag_geometry()?);
                let cdr = Rc::new(cdr.to_flag_geometry()?);
                match op.literal()? {
                    "h" => Some(UnresolvedFlagGeometry::Horizontal(car, cdr, pivot)),
                    "v" => Some(UnresolvedFlagGeometry::Vertical(car, cdr, pivot)),
                    _ => None,
                }
            }
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

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let flag = args()
        .nth(1)
        .and_then(|fdl| SExpr::parse(&mut fdl.chars().peekable()))
        .and_then(|sexpr| sexpr.to_flag_geometry())
        .and_then(|ufg| {
            let tags = ufg.tags();
            ufg.resolve(&tags)
        })
        .unwrap();

    eprintln!("{:#?}", flag);

    let mut img = RgbImage::new(400, 300);
    flag.draw(&mut img);
    img.save("out.png")?;

    Ok(())
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
