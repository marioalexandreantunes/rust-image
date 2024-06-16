
use image::{ImageBuffer, Rgb, Rgba, DynamicImage, GenericImageView};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use rayon::prelude::*;
use std::{fs, io};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;


pub const TOLERANCE: u8 = 30; // Adjust tolerance level as needed
pub const PERCENTAGE: usize = 25; // Adjust image PERCENTAGE to be ok as needed

/// Get all png files from a path
///
/// # Arguments
///
/// * 'path' - as string.
///
/// # Returns
///
/// io::Result<Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>> to use in tamplate_match
///
pub fn get_png_files_from_path<P: AsRef<Path>>(
    path: P,
) -> io::Result<Vec<(String, ImageBuffer<Rgba<u8>, Vec<u8>>)>> {
    // this final vector needs to have the 
    let mut image_buffers: Vec<(String, ImageBuffer<Rgba<u8>, Vec<u8>>)> = Vec::new();
    let mut png_files: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>> = Vec::new();
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let path: PathBuf = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "png" {
                    // need to return ImageBuffer<Rgba<u8> and path and a new &str
                    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
                        image::open(path.clone()).expect("Failed to load image").to_rgba8();
                    // TODO : in push i need the image name and path
                    let file_name = path.file_name().unwrap().to_string_lossy().into_owned().replace(".png", "");
                    png_files.push(img.clone());
                    image_buffers.push((file_name, img));
                }
            }
        }
    }
    // DISPLAY THE image_buffers
    for (file_name, img) in image_buffers.clone() {
        let (width, height) = img.dimensions();
        println!("{} - {}x{}", file_name, width, height);
    }
    Ok(image_buffers)
}


/// Proceeds with a template match, using parallelism.
///
/// # Arguments
///
/// * 'larger_image' - big image as a source.
/// * 'template' - path with all templates to search.
/// * 'debug' - if is to display and save infos.
///
/// # Returns
///
/// Vec<Vec<(u32, u32)>> - a vector with vectors with all positions matched.
///
pub fn get_template_matches(larger_image: &str, template_path: &str, debug: bool) -> Vec<Vec<(u32, u32)>>{
    //check if the paths exist
    let larger_image_path: &Path = Path::new(larger_image);
    let template_path: &Path = Path::new(template_path);
    if !larger_image_path.exists() || !template_path.exists() {
        panic!("One of the paths does not exist");
    }

    let start_time: Instant = Instant::now();

    let templates: Result<Vec<(String, ImageBuffer<Rgba<u8>, Vec<u8>>)>, io::Error> =
        get_png_files_from_path(template_path);

    // Init the results thread-safe var
    let results: Arc<Mutex<Vec<Vec<(u32, u32)>>>> = Arc::new(Mutex::new(Vec::new()));

    // Use rayon::scope to process files in parallel
    rayon::scope(|s: &rayon::Scope| {
        for template in templates.unwrap() {
            let results: Arc<Mutex<Vec<Vec<(u32, u32)>>>> = Arc::clone(&results);
            // TODO : get from name the tolerance and percentage
            let mut tolerance: u8 = TOLERANCE;
            let mut percentage: usize = PERCENTAGE;
            let name: String = template.0;
            let parts: Vec<&str> = name.split('_').collect();
            if parts.len() >= 3 {
                tolerance = parts[1].parse().unwrap_or(TOLERANCE);
                percentage = parts[2].parse().unwrap_or(PERCENTAGE);
            }
            
            // image buffer
            let temp: ImageBuffer<Rgba<u8>, Vec<u8>> = template.1;
            s.spawn(move |_| {
                let large_image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                    image::open(larger_image)
                        .expect("Failed to load image")
                        .to_rgba8();
                let result: Vec<(u32, u32)> =
                    template_match(&large_image, &temp, tolerance.clone(), percentage).clone();
                let mut results: std::sync::MutexGuard<Vec<Vec<(u32, u32)>>> =
                    results.lock().unwrap();
                results.push(result);
            });
        }
    });

    // get the results
    let debug_results: Vec<Vec<(u32, u32)>> = results.lock().unwrap().clone();
    let return_results: Vec<Vec<(u32, u32)>> = results.lock().unwrap().clone();

    let elapsed: std::time::Duration = start_time.elapsed();

    // print results  id debug
    if debug == true {
        println!(
            "Time taken to match templates: {:.6} seconds",
            elapsed.as_secs_f64()
        );
        for (idx, result) in debug_results.iter().enumerate() {
            println!("Template {} found at coordinates: {:?}", idx + 1, result);
        }
        debug_image(debug_results, larger_image);
    }
    return_results
}

/// Proceeds with a template match, using parallelism.
///
/// # Arguments
///
/// * 'larger_image' - big image as a source.
/// * 'template' - small image to search in source image.
/// * 'tolerance' - diff between each pixel channels (RGBA), default is 30.
/// * 'percentage' - what template percentage can be wrong, default is 25%.
///
/// # Returns
///
/// Vec<(u32, u32)> - a vector with all positions matched.
///
fn template_match(
    larger_image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    subimage: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    tolerance: u8,
    percentage: usize,
) -> Vec<(u32, u32)> {
    let (large_width, large_height): (u32, u32) = larger_image.dimensions();
    let (sub_width, sub_height) = subimage.dimensions();
    let pixel_count: usize = (sub_width * sub_height) as usize;
    let tolerance_threshold: usize = pixel_count * percentage / 100;

    // Create a vector of possible top-left corner positions to be checked
    let positions: Vec<(u32, u32)> = (0..=large_height - sub_height)
        .flat_map(|y| (0..=large_width - sub_width).map(move |x| (x, y)))
        .collect();

    // Use parallel processing to speed up the search
    positions
        .into_par_iter()
        .filter_map(|(x, y)| {
            let mut fail_count = 0;
            for sub_y in 0..sub_height {
                for sub_x in 0..sub_width {
                    let large_pixel = larger_image.get_pixel(x + sub_x, y + sub_y);
                    let sub_pixel = subimage.get_pixel(sub_x, sub_y);

                    // Compare RGBA values with tolerance
                    if !pixels_match_with_tolerance(large_pixel, sub_pixel, tolerance) {
                        fail_count += 1;
                        if fail_count > tolerance_threshold {
                            return None; // Exit early if fail count exceeds threshold
                        }
                    }
                }
            }
            Some((x, y))
        })
        .collect()
}

fn pixels_match_with_tolerance(pixel1: &Rgba<u8>, pixel2: &Rgba<u8>, tolerance: u8) -> bool {
    for i in 0..4 {
        // RGBA channels
        if (pixel1[i] as i16 - pixel2[i] as i16).abs() > tolerance as i16 {
            return false;
        }
    }
    true
}

/// Save a Image with all results
///
/// # Arguments
///
/// * 'results' - Vec<Vec<(u32, u32)>> from parallel template_match.
/// * 'large_image_save' - path from source image
///
pub fn debug_image(results: Vec<Vec<(u32, u32)>>, large_image_save: &str) {
    if results.is_empty() {
        println!("Subimage not found in larger image.");
    } else {
        println!("debug_image drawing {} templates:", results.len());
        let mut large_image_save = image::open(large_image_save)
            .expect("Failed to load image")
            .to_rgb8();

        for (_outer_index, inner_vec) in results.iter().enumerate() {
            //println!("Subimage #{} found at positions:", outer_index + 1);
            for (x, y) in inner_vec {
                //println!("({}, {})", x, y);
                let black = Rgb([0u8, 0u8, 0u8]);
                let x1: i32 = *x as i32;
                let y1: i32 = *y as i32;
                let rect = Rect::at(x1, y1).of_size(20, 20);
                draw_hollow_rect_mut(&mut large_image_save, rect, black);
            }
        }
        // Save the image with markers
        let output_path = "tests/result_image.png";
        large_image_save
            .save(output_path)
            .expect("Failed to save image");
        println!("Result image with markers saved to: {}", output_path);
    }
}


// TODO , make tests this is based in AFORGE ExhaustiveTemplateMatching.cs
pub fn exaust_temlate_image(image: &DynamicImage, template: &DynamicImage, threshold: i32, max_diff: i32) -> Vec<Vec<i32>> {
    let (image_width, image_height) = image.dimensions();
    let (template_width, template_height) = template.dimensions();

    let mut map = vec![vec![0; (image_width - template_width + 1) as usize]; (image_height - template_height + 1) as usize];

    let image_pixels = image.as_bytes();
    let template_pixels = template.as_bytes();

    let pixel_size = image.color().channel_count() as usize;

    for y in 0..image_height - template_height + 1 {
        for x in 0..image_width - template_width + 1 {
            let mut dif = 0;

            for i in 0..template_height {
                for j in 0..template_width {
                    let image_idx = ((y + i) * image_width + (x + j)) as usize * pixel_size;
                    let template_idx = (i * template_width + j) as usize * pixel_size;

                    for k in 0..pixel_size {
                        let d = (image_pixels[image_idx + k] as i32) - (template_pixels[template_idx + k] as i32);
                        if d > 0 {
                            dif += d;
                        } else {
                            dif -= d;
                        }
                    }
                }
            }

            let sim = max_diff - dif;
            if sim >= threshold {
                map[y as usize][x as usize] = sim;
            }
        }
    }

    map
}
