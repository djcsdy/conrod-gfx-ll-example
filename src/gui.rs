use conrod;

widget_ids! {
    /// Unique IDs for each widget.
    pub struct Ids {
        // The scrollable canvas.
        canvas,

        // The title and introduction widgets.
        title,
        introduction,

        // Shapes.
        shapes_canvas,
        rounded_rectangle,
        shapes_left_col,
        shapes_right_col,
        shapes_title,
        line,
        point_path,
        rectangle_fill,
        rectangle_outline,
        trapezoid,
        oval_fill,
        oval_outline,
        circle,

        // Images.
        image_title,
        rust_logo,

        // Button, XyPad, Toggle.
        button_title,
        button,
        xy_pad,
        toggle,
        ball,

        // NumberDialer, PlotPath
        dialer_title,
        number_dialer,
        plot_path,

        // Scrollbar
        canvas_scrollbar,
    }
}

/// A demonstration of some application state we want to control with a conrod GUI.
/// Borrowed from the standard conrod examples.
pub struct State {
    ball_xy: conrod::Point,
    ball_color: conrod::Color,
    sine_frequency: f32,
    rust_logo: conrod::image::Id,
}

impl State {
    /// Simple constructor for the `DemoApp`.
    pub fn new(rust_logo: conrod::image::Id) -> Self {
        State {
            ball_xy: [0.0, 0.0],
            ball_color: conrod::color::WHITE,
            sine_frequency: 1.0,
            rust_logo: rust_logo,
        }
    }
}
