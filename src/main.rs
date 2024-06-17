extern crate image;
extern crate imageproc;

use imageproc::rect::Rect;

mod template_match;
use template_match::{get_template_matches, EMULATOR_SIZES};

fn main() {
    let debug: bool = true;
    let img: &str = "tests/large_image.png";
    let temp: &str = "tests/templates";

    // Emulator sizes and if is to reduce search zone?
    let reduced_zone: bool = true;
    let (width, height) = EMULATOR_SIZES;
    let mut search_zone: Rect = Rect::at(0, 0).of_size(width, height);

    if reduced_zone {
        // rectangle Rect from x > 100 y>100 and width - 100, height - 100
        let _width = width - 100;
        let _height = height - 100;
        search_zone = Rect::at(100, 100).of_size(_width, _height);
    }

    println!(
        "windows to search is from x:{}, y:{} to x1:{} y1:{}",
        search_zone.left(),
        search_zone.top(),
        search_zone.width(),
        search_zone.height()
    );

    let results: Vec<Vec<(u32, u32)>> = get_template_matches(img, temp, debug, search_zone);
    // print each result
    if !debug {
        for (i, result) in results.iter().enumerate() {
            println!("Template {} matches:", i);
            for (x, y) in result {
                println!("({}, {})", x, y);
            }
        }
    }
}
