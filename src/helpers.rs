use glob::glob;
use itertools::Itertools;

pub struct Helpers;

impl Helpers {
    pub fn glob_paths(paths: &Vec<String>) -> Vec<String> {
        let mut input_images_globbed: Vec<String> = Vec::new();
        paths.iter().for_each(|input_image| {
            if input_image.contains(&"*".to_string()) {
                let paths = glob(input_image).unwrap_or_else(|_| panic!("Incorrect glob pattern: {}", input_image));
                paths.for_each(|path| {
                    if let Ok(path) = path {
                        // TODO: WTF?! Maybe it is possible to do it better?
                        input_images_globbed.push(path.to_str().unwrap().clone().parse().unwrap());
                    }
                });
            } else {
                input_images_globbed.push(input_image.to_string());
            }
        });
        input_images_globbed = input_images_globbed.into_iter().unique().collect();
        input_images_globbed.sort();
        input_images_globbed
    }
}