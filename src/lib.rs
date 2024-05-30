use std::ffi::{c_char, CString};

pub use self::{context::*, plot::*, plot_elements::*};
pub use implot_sys as sys;
pub use sys::{ImPlotPoint, ImPlotRange, ImPlotRect, ImVec2, ImVec4};

mod context;
mod plot;
mod plot_elements;

const NUMBER_OF_AXES: usize = sys::ImAxis_::COUNT as usize;

pub struct PlotUi<'ui> {
    context: &'ui Context,
}

/// Choice of axis.
pub type AxisChoice = sys::ImAxis_;

/// Markers.
pub type Marker = sys::ImPlotMarker_;

/// Colorable plot elements. These are called "ImPlotCol" in ImPlot itself, but I found that
/// name somewhat confusing because we are not referring to colors, but _which_ thing can
/// be colored - hence I added the "Element".
pub type PlotColorElement = sys::ImPlotCol_;

/// Colormap choice.
pub type Colormap = sys::ImPlotColormap_;

/// Style variable choice, as in "which thing will be affected by a style setting".
pub type StyleVar = sys::ImPlotStyleVar_;

/// Used to position items on a plot (e.g. legends, labels, etc.)
pub type PlotLocation = sys::ImPlotLocation_;

/// Used to hide/show legends, shoe them horizontally, etc.
pub type PlotLegendFlags = sys::ImPlotLegendFlags_;

/// Switch to one of the built-in preset colormaps.
#[rustversion::attr(since(1.48), doc(alias = "PushColormap"))]
pub fn push_colormap_from_preset(preset: Colormap) {
    unsafe {
        sys::ImPlot_PushColormap_PlotColormap(preset as sys::ImPlotColormap);
    }
}

/// Switch to a colormap by name.
#[rustversion::attr(since(1.48), doc(alias = "PushColormap"))]
pub fn push_colormap_from_name(name: &str) {
    let name = CString::new(name).unwrap();
    unsafe {
        sys::ImPlot_PushColormap_Str(name.as_ptr());
    }
}

/// Set a custom colormap in the form of a vector of colors.
#[rustversion::attr(since(1.48), doc(alias = "SetColormap"))]
pub fn add_colormap_from_vec(name: &str, colors: Vec<ImVec4>, discrete: bool) {
    let name = CString::new(name).unwrap();
    unsafe {
        sys::ImPlot_AddColormap_Vec4Ptr(
            name.as_ptr(),
            colors.as_ptr(),
            colors.len() as i32,
            discrete,
        );
    }
}

// --- Push/pop utils -------------------------------------------------------------------------
// Currently not in a struct yet. imgui-rs has some smarts about dealing with stacks, in particular
// leak detection, which I'd like to replicate here at some point.
/// Push a style color to the stack, giving an element and the four components of the color.
/// The components should be between 0.0 (no intensity) and 1.0 (full intensity).
/// The return value is a token that gets used for removing the style color from the stack again:
/// ```no_run
/// # use implot::{push_style_color, PlotColorElement};
/// let pushed_var = push_style_color(&PlotColorElement::Line, 1.0, 1.0, 1.0, 0.2);
/// // Plot some things
/// pushed_var.pop();
/// ```
#[rustversion::attr(since(1.48), doc(alias = "PushStyleColor"))]
pub fn push_style_color(
    element: &PlotColorElement,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
) -> StyleColorToken {
    unsafe {
        sys::ImPlot_PushStyleColor_Vec4(
            *element as sys::ImPlotCol,
            sys::ImVec4 {
                x: red,
                y: green,
                z: blue,
                w: alpha,
            },
        );
    }
    StyleColorToken { was_popped: false }
}

/// Tracks a change pushed to the style color stack
pub struct StyleColorToken {
    /// Whether this token has been popped or not.
    was_popped: bool,
}

impl StyleColorToken {
    #[rustversion::attr(since(1.48), doc(alias = "PopStyleColor"))]
    pub fn pop(mut self) {
        if self.was_popped {
            panic!("Attempted to pop a style color token twice.")
        }
        self.was_popped = true;
        unsafe {
            sys::ImPlot_PopStyleColor(1);
        }
    }
}

/// Push a f32 style variable to the stack. The returned token is used for removing
/// the variable from the stack again:
/// ```no_run
/// # use implot::{push_style_var_f32, StyleVar};
/// let pushed_var = push_style_var_f32(&StyleVar::LineWeight, 11.0);
/// // Plot some things
/// pushed_var.pop();
/// ```
#[rustversion::attr(since(1.48), doc(alias = "PushStyleVar"))]
pub fn push_style_var_f32(element: &StyleVar, value: f32) -> StyleVarToken {
    unsafe {
        sys::ImPlot_PushStyleVar_Float(*element as sys::ImPlotStyleVar, value);
    }
    StyleVarToken { was_popped: false }
}

/// Push an u32 style variable to the stack. The only i32 style variable is Marker
/// at the moment, for that, use something like
/// ```no_run
/// # use implot::{push_style_var_i32, StyleVar, Marker};
/// let markerchoice = push_style_var_i32(&StyleVar::Marker, Marker::Cross as i32);
/// // plot things
/// markerchoice.pop()
/// ```
#[rustversion::attr(since(1.48), doc(alias = "PushStyleVar"))]
pub fn push_style_var_i32(element: &StyleVar, value: i32) -> StyleVarToken {
    unsafe {
        sys::ImPlot_PushStyleVar_Int(*element as sys::ImPlotStyleVar, value);
    }
    StyleVarToken { was_popped: false }
}

/// Push an ImVec2 style variable to the stack. The returned token is used for removing
/// the variable from the stack again.
pub fn push_style_var_imvec2(element: &StyleVar, value: ImVec2) -> StyleVarToken {
    unsafe {
        sys::ImPlot_PushStyleVar_Vec2(*element as sys::ImPlotStyleVar, value);
    }
    StyleVarToken { was_popped: false }
}

/// Tracks a change pushed to the style variable stack
pub struct StyleVarToken {
    /// Whether this token has been popped or not.
    was_popped: bool,
}

impl StyleVarToken {
    /// Pop this token from the stack.
    #[rustversion::attr(since(1.48), doc(alias = "PopStyleVar"))]
    pub fn pop(mut self) {
        if self.was_popped {
            panic!("Attempted to pop a style var token twice.")
        }
        self.was_popped = true;
        unsafe {
            sys::ImPlot_PopStyleVar(1);
        }
    }
}

// --- Miscellaneous -----------------------------------------------------------------------------
/// Returns true if the plot area in the current or most recent plot is hovered.
#[rustversion::attr(since(1.48), doc(alias = "IsPlotHovered"))]
pub fn is_plot_hovered() -> bool {
    unsafe { sys::ImPlot_IsPlotHovered() }
}

pub type PlotDragToolFlags = sys::ImPlotDragToolFlags_;

/// Returns true if the user changed the coordinates.
#[rustversion::attr(since(1.48), doc(alias = "DragRect"))]
#[allow(clippy::too_many_arguments)]
pub fn drag_rect(
    id: i32,
    x1: &mut f64,
    y1: &mut f64,
    x2: &mut f64,
    y2: &mut f64,
    color: ImVec4,
    flags: PlotDragToolFlags,
    clicked: &mut bool,
    hovered: &mut bool,
    held: &mut bool,
) -> bool {
    unsafe {
        sys::ImPlot_DragRect(
            id,
            x1,
            y1,
            x2,
            y2,
            color,
            flags.0 as sys::ImPlotDragToolFlags,
            clicked,
            hovered,
            held,
        )
    }
}

/// Returns the mouse position in x,y coordinates of the current or most recent plot,
/// for the specified choice of axes.
#[rustversion::attr(since(1.48), doc(alias = "GetPlotMousePos"))]
pub fn get_plot_mouse_position(x_axis: AxisChoice, y_axis: AxisChoice) -> ImPlotPoint {
    let mut point = ImPlotPoint { x: 0.0, y: 0.0 }; // doesn't seem to have default()
    unsafe {
        sys::ImPlot_GetPlotMousePos(
            &mut point as *mut ImPlotPoint,
            x_axis as sys::ImAxis,
            y_axis as sys::ImAxis,
        );
    }
    point
}

/// Convert pixels, given as an `ImVec2`, to a position in the current plot's coordinate system.
#[rustversion::attr(since(1.48), doc(alias = "PixelsToPlot"))]
pub fn pixels_to_plot_vec2(
    pixel_position: &ImVec2,
    x_axis: AxisChoice,
    y_axis: AxisChoice,
) -> ImPlotPoint {
    let mut point = ImPlotPoint { x: 0.0, y: 0.0 }; // doesn't seem to have default()
    unsafe {
        sys::ImPlot_PixelsToPlot_Vec2(
            &mut point as *mut ImPlotPoint,
            *pixel_position,
            x_axis as sys::ImAxis,
            y_axis as sys::ImAxis,
        );
    }
    point
}

/// Convert pixels, given as floats `x` and `y`, to a position in the current plot's coordinate
/// system.
#[rustversion::attr(since(1.48), doc(alias = "PixelsToPlot"))]
pub fn pixels_to_plot_f32(
    pixel_position_x: f32,
    pixel_position_y: f32,
    x_axis: AxisChoice,
    y_axis: AxisChoice,
) -> ImPlotPoint {
    let mut point = ImPlotPoint { x: 0.0, y: 0.0 }; // doesn't seem to have default()
    unsafe {
        sys::ImPlot_PixelsToPlot_Float(
            &mut point as *mut ImPlotPoint,
            pixel_position_x,
            pixel_position_y,
            x_axis as sys::ImAxis,
            y_axis as sys::ImAxis,
        );
    }
    point
}

/// Convert a position in the current plot's coordinate system to pixels.
#[rustversion::attr(since(1.48), doc(alias = "PlotToPixels"))]
pub fn plot_to_pixels_vec2(
    plot_position: &ImPlotPoint,
    x_axis: AxisChoice,
    y_axis: AxisChoice,
) -> ImVec2 {
    let mut pixel_position = ImVec2 { x: 0.0, y: 0.0 }; // doesn't seem to have default()
    unsafe {
        sys::ImPlot_PlotToPixels_PlotPoInt(
            &mut pixel_position as *mut ImVec2,
            *plot_position,
            x_axis as sys::ImAxis,
            y_axis as sys::ImAxis,
        );
    }
    pixel_position
}

/// Convert a position in the current plot's coordinate system to pixels.
#[rustversion::attr(since(1.48), doc(alias = "PlotToPixels"))]
pub fn plot_to_pixels_f32(
    plot_position_x: f64,
    plot_position_y: f64,
    x_axis: AxisChoice,
    y_axis: AxisChoice,
) -> ImVec2 {
    let mut pixel_position = ImVec2 { x: 0.0, y: 0.0 }; // doesn't seem to have default()
    unsafe {
        sys::ImPlot_PlotToPixels_double(
            &mut pixel_position as *mut ImVec2,
            plot_position_x,
            plot_position_y,
            x_axis as sys::ImAxis,
            y_axis as sys::ImAxis,
        );
    }
    pixel_position
}

/// Returns the current or most recent plot axis range for the specified choice of Y axis.
#[rustversion::attr(since(1.48), doc(alias = "GetPlotLimits"))]
pub fn get_plot_limits(x_axis: AxisChoice, y_axis: AxisChoice) -> ImPlotRect {
    // ImPlotLimits doesn't seem to have default()
    let mut limits = ImPlotRect {
        X: ImPlotRange { Min: 0.0, Max: 0.0 },
        Y: ImPlotRange { Min: 0.0, Max: 0.0 },
    };
    unsafe {
        sys::ImPlot_GetPlotLimits(
            &mut limits as *mut ImPlotRect,
            x_axis as sys::ImAxis,
            y_axis as sys::ImAxis,
        );
    }
    limits
}

/// Set the Y axis to be used for any upcoming plot elements
#[rustversion::attr(since(1.48), doc(alias = "SetAxis"))]
pub fn set_axis(axis_choice: AxisChoice) {
    unsafe {
        sys::ImPlot_SetAxis(axis_choice as sys::ImAxis);
    }
}

/// Returns true if the axis plot area in the current plot is hovered.
#[rustversion::attr(since(1.48), doc(alias = "IsAxisHovered"))]
pub fn is_axis_hovered(axis: AxisChoice) -> bool {
    unsafe { sys::ImPlot_IsAxisHovered(axis as sys::ImAxis) }
}

/// Returns true if the given item in the legend of the current plot is hovered.
pub fn is_legend_entry_hovered(legend_entry: &str) -> bool {
    unsafe { sys::ImPlot_IsLegendEntryHovered(legend_entry.as_ptr() as *const c_char) }
}

// --- Demo window -------------------------------------------------------------------------------
/// Show the demo window for poking around what functionality implot has to
/// offer. Note that not all of this is necessarily implemented in implot-rs
/// already - if you find something missing you'd really like, raise an issue.
// This requires implot_demo.cpp to be in the list of sources in implot-sys.
#[rustversion::attr(since(1.48), doc(alias = "ShowDemoWindow"))]
pub fn show_demo_window(show: &mut bool) {
    unsafe {
        implot_sys::ImPlot_ShowDemoWindow(show);
    }
}
