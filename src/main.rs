extern crate image;
extern crate imageproc;

mod template_match;
use template_match::get_template_matches;

fn main() {
    let debug: bool = true;
    let img: &str = "tests/large_image.png";
    let temp: &str = "tests/templates";
    let results: Vec<Vec<(u32, u32)>> = get_template_matches(img, temp, debug);
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
