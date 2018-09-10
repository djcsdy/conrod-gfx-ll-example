use conrod;

/// A demonstration of some application state we want to control with a conrod GUI.
/// Borrowed from the standard conrod examples.
pub struct DemoApp {
    ball_xy: conrod::Point,
    ball_color: conrod::Color,
    sine_frequency: f32,
    rust_logo: conrod::image::Id,
}

impl DemoApp {
    /// Simple constructor for the `DemoApp`.
    pub fn new(rust_logo: conrod::image::Id) -> Self {
        DemoApp {
            ball_xy: [0.0, 0.0],
            ball_color: conrod::color::WHITE,
            sine_frequency: 1.0,
            rust_logo: rust_logo,
        }
    }
}
