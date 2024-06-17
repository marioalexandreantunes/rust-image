extern crate image;
extern crate imageproc;

use imageproc::rect::Rect;

mod template_match;
use template_match::get_template_matches;

fn main() {
    let debug: bool = true;
    let img: &str = "tests/large_image.png";
    let temp: &str = "tests/templates";
    // rectangle imageproc::rect::Rect fron x 100 y 100 abd width less 100
    //let width = 860 - 100;
    //let height = 644 - 100;
    //let search_zone: imageproc::rect::Rect = Rect::at(100, 100).of_size(width, height);

    // All image 
    let search_zone: imageproc::rect::Rect = Rect::at(0, 0).of_size(860, 644);
    
    println!(
        "windows to search is from x:{}, y:{} to x1:{} y1:{}",
        search_zone.left(), search_zone.top(), search_zone.width(), search_zone.height()
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
