use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use png;
pub struct PngHelper;

impl PngHelper {
    pub fn test() {
        let path = Path::new(r"./bla-bla.png");
        let file = File::create(path).unwrap();
        let w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, 2, 1); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
        let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000)
        );
        encoder.set_source_chromaticities(source_chromaticities);
        let mut array: [u8; 256 * 3] = [255; 256 * 3];
        // first color
        array[0] = 255;
        array[1] = 0;
        array[2] = 0;
        // second color
        array[3] = 0;
        array[4] = 255;
        array[5] = 127;
        encoder.set_palette(&array[..]);
        let mut writer = encoder.write_header().unwrap();
        let data = [0, 1]; // sequence of indexes in palette
        writer.write_image_data(&data).unwrap(); // Save
    }
}