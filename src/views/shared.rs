use image::DynamicImage;
use image::GenericImageView;
use midpoint_engine::floem::peniko::Color;
use midpoint_engine::floem::reactive::create_effect;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::views::dyn_container;
use midpoint_engine::floem::views::empty;
use midpoint_engine::floem::views::img;
use midpoint_engine::floem::views::img_dynamic; // Note: using img_dynamic instead of img
use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{
    reactive::{create_rw_signal, RwSignal},
    IntoView,
};
use std::fs;
use std::rc::Rc;

use crate::helpers::utilities::get_common_os_dir;

pub fn dynamic_img(image_path: String, filename: String, width: f32, height: f32) -> impl IntoView {
    let image_signal: RwSignal<Option<Rc<DynamicImage>>> = create_rw_signal(None); // fix?

    create_effect(move |_| {
        let sync_dir = get_common_os_dir().expect("Couldn't get CommonOS directory");
        let target_dir = sync_dir.join(&image_path);
        let target_file = target_dir.join(&filename);

        match image::open(&target_file) {
            Ok(img) => {
                println!("Loaded image dimensions: {:?}", img.dimensions());
                image_signal.set(Some(Rc::new(img)));
            }
            Err(e) => {
                println!("Error loading image: {}", e);
                image_signal.set(None);
            }
        }
    });

    // img_dynamic(move || image_signal.get()).style(
    //     |s| {
    //         s.width(120.0)
    //             .height(120.0)
    //             .background(Color::rgb(200.0, 200.0, 200.0))
    //     }, // To see where it should be
    // )

    dyn_container(
        move || image_signal.get(),
        move |image_signal_real| {
            if image_signal_real.is_some() {
                img_dynamic(move || image_signal.get())
                    .style(
                        move |s| {
                            s.width(width)
                                .height(height)
                                .background(Color::rgb(200.0, 200.0, 200.0))
                                .border_radius(5.0)
                        }, // To see where it should be
                    )
                    .into_any()
            } else {
                empty().into_any()
            }
        },
    )
}

// pub fn test_image() -> impl IntoView {
//     // Create a simple test image
//     let mut img = image::RgbaImage::new(100, 100);
//     // Fill with a red color
//     for pixel in img.pixels_mut() {
//         *pixel = image::Rgba([255, 0, 0, 255]);
//     }
//     let dynamic_img = DynamicImage::ImageRgba8(img);

//     println!("Created test image");

//     img_dynamic(move || Some(Rc::new(dynamic_img.clone()))).style(|s| {
//         s.width(100)
//             .height(100)
//             .border(1)
//             .border_color(Color::rgb(255.0, 0.0, 0.0))
//             .background(Color::rgb(200.0, 200.0, 200.0))
//     })
// }

// // Let's also add a test with a known working image
// pub fn test_static_image() -> impl IntoView {
//     // Create a simple 2x2 red square image
//     let raw_bytes = vec![
//         255, 0, 0, 255, // Red pixel
//         255, 0, 0, 255, // Red pixel
//         255, 0, 0, 255, // Red pixel
//         255, 0, 0, 255, // Red pixel
//     ];

//     img(move || raw_bytes.clone()).style(|s| {
//         s.width(100)
//             .height(100)
//             .border(1)
//             .border_color(Color::rgb(255.0, 0.0, 0.0))
//             .background(Color::rgb(200.0, 200.0, 200.0))
//     })
// }

// pub fn test_static_image() -> impl IntoView {
//     // Create a 100x100 red image
//     let mut img = image::RgbaImage::new(100, 100);
//     for pixel in img.pixels_mut() {
//         *pixel = image::Rgba([255, 0, 0, 255]);
//     }
//     let dynamic_img = image::DynamicImage::ImageRgba8(img);
//     println!("Created test image: {:?}", dynamic_img.dimensions());

//     img_dynamic(move || Some(Rc::new(dynamic_img.clone()))).style(|s| {
//         s.width(100)
//             .height(100)
//             .border(1)
//             .border_color(Color::rgb(255.0, 0.0, 0.0))
//             .background(Color::rgb(200.0, 200.0, 200.0))
//     })
// }
