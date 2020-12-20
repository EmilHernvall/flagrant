use std::env::args;

use flagrant::SExpr;

use image::RgbImage;

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
