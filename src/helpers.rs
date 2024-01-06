use glob::glob;
use itertools::Itertools;

pub struct Helpers;

impl Helpers {
    pub fn glob_paths(paths: &Vec<String>) -> Vec<String> {
        let mut paths_globbed: Vec<String> = Vec::new();
        paths.iter().for_each(|input_image| {
            if input_image.contains(&"*".to_string()) {
                let paths = glob(input_image)
                    .unwrap_or_else(|_| panic!("Incorrect glob pattern: {}", input_image));
                paths.for_each(|path| {
                    if let Ok(path) = path {
                        // TODO: WTF?! Maybe it is possible to do it better?
                        paths_globbed.push(path.to_str().unwrap().parse().unwrap());
                    }
                });
            } else {
                paths_globbed.push(input_image.to_string());
            }
        });
        paths_globbed = paths_globbed.into_iter().unique().collect();
        paths_globbed.sort();
        paths_globbed
    }
}
