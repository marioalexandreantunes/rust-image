use image::{ImageBuffer, Rgb, Rgba};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{fs, io};

pub const TOLERANCE: u8 = 30; // Adjust tolerance level as needed
pub const PERCENTAGE: usize = 25; // Adjust image PERCENTAGE to be ok as needed
pub const EMULATOR_SIZES: (u32, u32) = (860, 644); // tuple of two u32 values : emulator sizes

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
                    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = image::open(path.clone())
                        .expect("Failed to load image")
                        .to_rgba8();
                    // TODO : in push i need the image name and path
                    let file_name = path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()
                        .replace(".png", "");
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
/// Vec<Vec<(u32, u32, String)>> - a vector with vectors with x,y + template name used
///
pub fn get_template_matches(
    larger_image: &str,
    template_path: &str,
    debug: bool,
    search_zone: Rect,
) -> Vec<Vec<(u32, u32, String)>> {
    //check if the paths exist
    let larger_image_path: &Path = Path::new(larger_image);
    let template_path: &Path = Path::new(template_path);
    if !larger_image_path.exists() || !template_path.exists() {
        panic!("One of the paths does not exist");
    }

    let templates: Result<Vec<(String, ImageBuffer<Rgba<u8>, Vec<u8>>)>, io::Error> =
        get_png_files_from_path(template_path);

    // a rectangle to rduce serach zone
    let search_zone = search_zone;

    // Init the results thread-safe var
    let results: Arc<Mutex<Vec<Vec<(u32, u32, String)>>>> = Arc::new(Mutex::new(Vec::new()));

    let start_time: Instant = Instant::now();

    // Use rayon::scope to process files in parallel
    rayon::scope(|s: &rayon::Scope| {
        for template in templates.unwrap() {
            let start_time_loop: Instant = Instant::now();
            let results: Arc<Mutex<Vec<Vec<(u32, u32, String)>>>> = Arc::clone(&results);
            // TODO : get from name the tolerance and percentage
            let mut tolerance: u8 = TOLERANCE;
            let mut percentage: usize = PERCENTAGE;
            let mut template_name: String = "".to_string();

            let name: String = template.0;
            let parts: Vec<&str> = name.split('_').collect();
            if parts.len() >= 3 {
                template_name = parts[0].to_string();
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
                let result: Vec<(u32, u32)> = template_match(
                    &large_image,
                    &temp,
                    tolerance.clone(),
                    percentage.clone(),
                    search_zone.clone(),
                );
                let mut results: std::sync::MutexGuard<Vec<Vec<(u32, u32, String)>>> =
                    results.lock().unwrap();
                let elapsed_loop: std::time::Duration = start_time_loop.elapsed();
                if debug {
                    println!("{} took: {:.2?}", template_name, elapsed_loop);
                }
                // Convert each point to a tuple with a String
                let points_with_string: Vec<(u32, u32, String)> = result
                    .into_iter()
                    .map(|(x, y)| (x, y, template_name.to_string()))
                    .collect();
                results.push(points_with_string);
            });
        }
    });

    // get the results
    let debug_results: Vec<Vec<(u32, u32, String)>> = results.lock().unwrap().clone();
    let return_results: Vec<Vec<(u32, u32, String)>> = results.lock().unwrap().clone();

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
    search_zone: Rect,
) -> Vec<(u32, u32)> {
    let (large_width, large_height): (u32, u32) = larger_image.dimensions();
    let (sub_width, sub_height) = subimage.dimensions();
    let pixel_count: usize = (sub_width * sub_height) as usize;
    let tolerance_threshold: usize = pixel_count * percentage / 100;

    if large_width < search_zone.width() {
        panic!(
            "Search zone width {} is larger than the larger image width {}",
            search_zone.width(),
            large_width
        );
    }
    if large_height < search_zone.height() {
        panic!(
            "Search zone height {} is larger than the larger image height {}",
            search_zone.height(),
            large_height
        );
    }

    // Create a vector of possible top-left corner positions to be checked
    let positions: Vec<(u32, u32)> = (search_zone.left() as u32
        ..=search_zone.height() as u32 - sub_height)
        .flat_map(|y| {
            (search_zone.top() as u32..=search_zone.width() - sub_width).map(move |x| (x, y))
        })
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
pub fn debug_image(results: Vec<Vec<(u32, u32, String)>>, large_image_save: &str) {
    if results.is_empty() {
        println!("Subimage not found in larger image.");
    } else {
        println!("debug_image drawing {} templates:", results.len());
        let mut large_image_save = image::open(large_image_save)
            .expect("Failed to load image")
            .to_rgb8();

        for (_outer_index, inner_vec) in results.iter().enumerate() {
            //println!("Subimage #{} found at positions:", outer_index + 1);
            for (x, y, _name) in inner_vec {
                //println!("{} at ({}, {})", name, x, y);
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
