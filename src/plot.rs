//! # Plot module
//!
//! This module defines the `Plot` struct, which is used to create a 2D plot that will
//! contain all other objects that can be created using this library.

#![allow(clippy::bad_bit_mask)]

use crate::{AxisChoice, Context, PlotLegendFlags, PlotLocation, PlotUi, NUMBER_OF_AXES};
pub use imgui::Condition;
use implot_sys::{self as sys, ImAxis, ImPlotFlags, ImPlotLocation, ImPlotPoint, ImVec4};
use std::ffi::{c_int, c_void, CString};
use std::os::raw::c_char;
use std::{cell::RefCell, rc::Rc};
pub use sys::{ImPlotRange, ImVec2};

const DEFAULT_PLOT_SIZE_X: f32 = 400.0;
const DEFAULT_PLOT_SIZE_Y: f32 = 400.0;
pub(crate) const IMPLOT_AUTO: i32 = -1;
pub(crate) const IMVEC2_ZERO: ImVec2 = ImVec2 { x: 0.0, y: 0.0 };
pub(crate) const IMPLOT_AUTO_COL: ImVec4 = ImVec4 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: -1.0,
};

pub type PlotFlags = sys::ImPlotFlags_;
pub type AxisFlags = sys::ImPlotAxisFlags_;
pub type AxisScale = sys::ImPlotScale_;
pub type PlotCond = sys::ImPlotCond_;

/// Internally-used struct for storing axis limits
#[derive(Clone)]
enum AxisLimitSpecification {
    /// Direct limits, specified as values
    Single(ImPlotRange, PlotCond),
    /// Limits that are linked to limits of other plots (via clones of the same Rc)
    Linked(Rc<RefCell<ImPlotRange>>),
}

type FormatCallback<'p> = dyn FnMut(f64) -> String + 'p;

pub enum AxisFormat<'p> {
    FormatString(CString),
    Callback(Box<Box<FormatCallback<'p>>>),
}

impl<'p, F: FnMut(f64) -> String + 'p> From<F> for AxisFormat<'p> {
    fn from(cb: F) -> Self {
        Self::Callback(Box::new(Box::new(cb)))
    }
}

impl From<CString> for AxisFormat<'_> {
    fn from(fmt: CString) -> Self {
        Self::FormatString(fmt)
    }
}

/// Struct to represent an ImPlot. This is the main construct used to contain all kinds of plots in ImPlot.
///
/// `Plot` is to be used (within an imgui window) with the following pattern:
/// ```no_run
/// # use implot;
/// let plotting_context = implot::Context::create();
/// let plot_ui = plotting_context.get_plot_ui();
/// implot::Plot::new("my title")
///     .size([300.0, 200.0]) // other things such as .x_label("some_label") can be added too
///     .build(&plot_ui, |_| {
///         // Do things such as plotting lines
///     });
///
/// ```
/// (If you are coming from the C++ implementation or the C bindings: build() calls both
/// begin() and end() internally)
pub struct Plot<'p> {
    /// Title of the plot, shown on top. Stored as CString because that's what we'll use
    /// afterwards, and this ensures the CString itself will stay alive long enough for the plot.
    title: CString,
    /// Size of the plot in [x, y] direction, in the same units imgui uses.
    size: [f32; 2],
    /// Label of an axis. Stored as CString because that's what we'll use
    /// afterwards, and this ensures the CString itself will stay alive long enough for the plot.
    labels: [Option<CString>; NUMBER_OF_AXES],
    /// Enable the axis
    axis_enabled: [bool; NUMBER_OF_AXES],
    /// Axis limits, if present
    axis_limits: [Option<AxisLimitSpecification>; NUMBER_OF_AXES],
    /// Axis limits constraints, if present
    axis_limits_constraints: [Option<(f64, f64)>; NUMBER_OF_AXES],
    /// Axis zoom constraints, if present
    axis_zoom_constraints: [Option<(f64, f64)>; NUMBER_OF_AXES],
    /// Positions for custom axis ticks, if any
    axis_tick_positions: [Option<Vec<f64>>; NUMBER_OF_AXES],
    /// Labels for custom axis ticks, if any. I'd prefer to store these together
    /// with the positions in one vector of an algebraic data type, but this would mean extra
    /// copies when it comes time to draw the plot because the C++ library expects separate lists.
    /// The data is stored as CStrings because those are null-terminated, and since we have to
    /// convert to null-terminated data anyway, we may as well do that directly instead of cloning
    /// Strings and converting them afterwards.
    axis_tick_labels: [Option<Vec<CString>>; NUMBER_OF_AXES],
    /// Axis scale (e.g.: linear, log10, ...)
    axis_scales: [sys::ImPlotScale; NUMBER_OF_AXES],
    /// Custom axis ticks format
    axis_format: [Option<AxisFormat<'p>>; NUMBER_OF_AXES],
    /// Whether to also show the default ticks when showing custom ticks or not
    show_axis_default_ticks: [bool; NUMBER_OF_AXES],
    /// Configuration for the legend, if specified. The tuple contains location, orientation
    /// and a boolean (true means legend is outside of plot, false means within). If nothing
    /// is set, implot's defaults are used. Note also  that if these are set, then implot's
    /// interactive legend configuration does not work because it is overridden by the settings
    /// here.
    legend_configuration: Option<(PlotLocation, PlotLegendFlags)>,
    /// Flags relating to the plot TODO(4bb4) make those into bitflags
    plot_flags: sys::ImPlotFlags,
    /// Flags relating to the each of the Y axes of the plot TODO(4bb4) make those into bitflags
    axis_flags: [sys::ImPlotAxisFlags; NUMBER_OF_AXES],
}

impl<'p> Plot<'p> {
    /// Create a new plot with some defaults set. Does not draw anything yet.
    /// Note that this uses antialiasing by default, unlike the C++ API. If you are seeing
    /// artifacts or weird rendering, try disabling it.
    ///
    /// # Panics
    /// Will panic if the title string contains internal null bytes.
    pub fn new(title: &str) -> Self {
        // Needed for initialization, see https://github.com/rust-lang/rust/issues/49147
        const LABELS_NONE: Option<CString> = None;
        const LIMITS_NONE: Option<AxisLimitSpecification> = None;
        const LIMITS_CONSTRAINTS_NONE: Option<(f64, f64)> = None;
        const LIMITS_ZOOM_NONE: Option<(f64, f64)> = None;
        const POS_NONE: Option<Vec<f64>> = None;
        const TICK_NONE: Option<Vec<CString>> = None;
        const AXIS_FORMAT_NONE: Option<AxisFormat> = None;

        let mut axis_enabled = [false; NUMBER_OF_AXES];
        axis_enabled[AxisChoice::X1 as usize] = true;
        axis_enabled[AxisChoice::Y1 as usize] = true;

        // TODO(4bb4) question these defaults, maybe remove some of them
        Self {
            title: CString::new(title)
                .unwrap_or_else(|_| panic!("String contains internal null bytes: {}", title)),
            size: [DEFAULT_PLOT_SIZE_X, DEFAULT_PLOT_SIZE_Y],
            labels: [LABELS_NONE; NUMBER_OF_AXES],
            axis_enabled,
            axis_limits: [LIMITS_NONE; NUMBER_OF_AXES],
            axis_limits_constraints: [LIMITS_CONSTRAINTS_NONE; NUMBER_OF_AXES],
            axis_zoom_constraints: [LIMITS_ZOOM_NONE; NUMBER_OF_AXES],
            axis_tick_positions: [POS_NONE; NUMBER_OF_AXES],
            axis_tick_labels: [TICK_NONE; NUMBER_OF_AXES],
            axis_scales: [AxisScale::Linear as sys::ImPlotScale; NUMBER_OF_AXES],
            show_axis_default_ticks: [false; NUMBER_OF_AXES],
            legend_configuration: None,
            plot_flags: PlotFlags::NONE.0 as sys::ImPlotFlags,
            axis_flags: [AxisFlags::NONE.0 as sys::ImPlotAxisFlags; NUMBER_OF_AXES],
            axis_format: [AXIS_FORMAT_NONE; NUMBER_OF_AXES],
        }
    }

    #[inline]
    pub fn with_axis(mut self, choice: AxisChoice) -> Self {
        self.axis_enabled[choice as usize] = true;
        self
    }

    /// Sets the plot size, given as [size_x, size_y]. Units are the same as
    /// what imgui uses. TODO(4bb4) ... which is? I'm not sure it's pixels
    #[inline]
    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.size = size;
        self
    }

    /// Set the x label of the plot
    ///
    /// # Panics
    /// Will panic if the label string contains internal null bytes.
    #[inline]
    pub fn x_label(self, label: &str) -> Self {
        self.axis_label(label, AxisChoice::X1)
    }

    /// Set the y label of the plot
    ///
    /// # Panics
    /// Will panic if the label string contains internal null bytes.
    #[inline]
    pub fn y_label(self, label: &str) -> Self {
        self.axis_label(label, AxisChoice::Y1)
    }

    pub fn axis_label(mut self, label: &str, axis_choice: AxisChoice) -> Self {
        self.axis_enabled[axis_choice as usize] = true;
        self.labels[axis_choice as usize] = if !label.is_empty() {
            Some(
                CString::new(label)
                    .unwrap_or_else(|_| panic!("String contains internal null bytes: {}", label)),
            )
        } else {
            None
        };
        self
    }

    /// Set the Y limits of the plot for the given Y axis. Call multiple times with different
    /// `axis_choice` values to set for multiple axes, or use the convenience methods such as
    /// [`Plot::y1_limits`].
    ///
    /// Note: This conflicts with `linked_axis_limits`, whichever is called last on plot construction
    /// takes effect for a given axis.
    #[inline]
    pub fn axis_limits<L: Into<ImPlotRange>>(
        mut self,
        limits: L,
        axis_choice: AxisChoice,
        condition: PlotCond,
    ) -> Self {
        let axis_index = axis_choice as usize;
        self.axis_enabled[axis_index] = true;
        self.axis_limits[axis_index] =
            Some(AxisLimitSpecification::Single(limits.into(), condition));
        self
    }

    /// Convenience function to directly set the X limits for the first X axis. To programmatically
    /// (or on demand) decide which axis to set limits for, use [`Plot::axis_limits`]
    #[inline]
    pub fn x_limits<L: Into<ImPlotRange>>(self, limits: L, condition: PlotCond) -> Self {
        self.x1_limits(limits, condition)
    }

    /// Convenience function to directly set the Y limits for the first Y axis. To programmatically
    /// (or on demand) decide which axis to set limits for, use [`Plot::axis_limits`]
    #[inline]
    pub fn y_limits<L: Into<ImPlotRange>>(self, limits: L, condition: PlotCond) -> Self {
        self.y1_limits(limits, condition)
    }

    /// Convenience function to directly set the X limits for the first X axis. To programmatically
    /// (or on demand) decide which axis to set limits for, use [`Plot::axis_limits`]
    #[inline]
    pub fn x1_limits<L: Into<ImPlotRange>>(self, limits: L, condition: PlotCond) -> Self {
        self.axis_limits(limits, AxisChoice::X1, condition)
    }

    /// Convenience function to directly set the Y limits for the first Y axis. To programmatically
    /// (or on demand) decide which axis to set limits for, use [`Plot::y_limits`]
    #[inline]
    pub fn y1_limits<L: Into<ImPlotRange>>(self, limits: L, condition: PlotCond) -> Self {
        self.axis_limits(limits, AxisChoice::Y1, condition)
    }

    /// Convenience function to directly set the Y limits for the second Y axis. To
    /// programmatically (or on demand) decide which axis to set limits for, use [`Plot::y_limits`]
    #[inline]
    pub fn y2_limits<L: Into<ImPlotRange>>(self, limits: L, condition: PlotCond) -> Self {
        self.axis_limits(limits, AxisChoice::Y2, condition)
    }

    /// Convenience function to directly set the Y limits for the third Y axis. To programmatically
    /// (or on demand) decide which axis to set limits for, use [`Plot::y_limits`]
    #[inline]
    pub fn y3_limits<L: Into<ImPlotRange>>(self, limits: L, condition: PlotCond) -> Self {
        self.axis_limits(limits, AxisChoice::Y3, condition)
    }

    #[inline]
    pub fn axis_limits_constraints(mut self, axis: AxisChoice, v_min: f64, v_max: f64) -> Self {
        let axis_index = axis as usize;
        self.axis_limits_constraints[axis_index] = Some((v_min, v_max));
        self
    }

    #[inline]
    pub fn axis_zoom_constraints(mut self, axis: AxisChoice, z_min: f64, z_max: f64) -> Self {
        let axis_index = axis as usize;
        self.axis_zoom_constraints[axis_index] = Some((z_min, z_max));
        self
    }

    #[inline]
    pub fn axis_format(mut self, axis: AxisChoice, formatter: impl Into<AxisFormat<'p>>) -> Self {
        let axis_index = axis as usize;
        self.axis_format[axis_index] = Some(formatter.into());
        self
    }

    /// Set linked Y limits of the plot for the given Y axis. Pass clones of the same `Rc` into
    /// other plots to link their limits with the same values. Call multiple times with different
    /// `y_axis_choice` values to set for multiple axes, or use the convenience methods such as
    /// [`Plot::y1_limits`].
    ///
    /// Note: This conflicts with `y_limits`, whichever is called last on plot construction takes
    /// effect for a given axis.
    #[inline]
    pub fn linked_axis_limits(
        mut self,
        limits: Rc<RefCell<ImPlotRange>>,
        axis_choice: AxisChoice,
    ) -> Self {
        let axis_index = axis_choice as usize;
        self.axis_enabled[axis_index] = true;
        self.axis_limits[axis_index] = Some(AxisLimitSpecification::Linked(limits));
        self
    }

    /// Convenience function to directly set linked X limits for the first X axis. To
    /// programmatically (or on demand) decide which axis to set limits for, use
    /// [`Plot::linked_axis_limits`].
    #[inline]
    pub fn linked_x1_limits(self, limits: Rc<RefCell<ImPlotRange>>) -> Self {
        self.linked_axis_limits(limits, AxisChoice::X1)
    }

    /// Convenience function to directly set linked Y limits for the first Y axis. To
    /// programmatically (or on demand) decide which axis to set limits for, use
    /// [`Plot::linked_axis_limits`].
    #[inline]
    pub fn linked_y1_limits(self, limits: Rc<RefCell<ImPlotRange>>) -> Self {
        self.linked_axis_limits(limits, AxisChoice::Y1)
    }

    /// Convenience function to directly set linked Y limits for the second Y axis. To
    /// programmatically (or on demand) decide which axis to set limits for, use
    /// [`Plot::linked_axis_limits`].
    #[inline]
    pub fn linked_y2_limits(self, limits: Rc<RefCell<ImPlotRange>>) -> Self {
        self.linked_axis_limits(limits, AxisChoice::Y2)
    }

    /// Convenience function to directly set linked Y limits for the third Y axis. To
    /// programmatically (or on demand) decide which axis to set limits for, use
    /// [`Plot::linked_axis_limits`].
    #[inline]
    pub fn linked_y3_limits(self, limits: Rc<RefCell<ImPlotRange>>) -> Self {
        self.linked_axis_limits(limits, AxisChoice::Y3)
    }

    /// Set X ticks without labels for the plot. The vector contains one label each in
    /// the form of a tuple `(label_position, label_string)`. The `show_default` setting
    /// determines whether the default ticks are also shown.
    #[inline]
    pub fn x_ticks(self, ticks: &[f64], show_default: bool) -> Self {
        self.axis_ticks(AxisChoice::X1, ticks, show_default)
    }

    /// Set X ticks without labels for the plot. The vector contains one label each in
    /// the form of a tuple `(label_position, label_string)`. The `show_default` setting
    /// determines whether the default ticks are also shown.
    #[inline]
    pub fn axis_ticks(
        mut self,
        axis_choice: AxisChoice,
        ticks: &[f64],
        show_default: bool,
    ) -> Self {
        let axis_index = axis_choice as usize;
        self.axis_enabled[axis_index] = true;
        self.axis_tick_positions[axis_index] = Some(ticks.into());
        self.show_axis_default_ticks[axis_index] = show_default;
        self
    }

    /// Set X ticks with labels for the plot. The vector contains one position and label
    /// each in the form of a tuple `(label_position, label_string)`. The `show_default`
    /// setting determines whether the default ticks are also shown.
    ///
    /// # Panics
    /// Will panic if any of the tick label strings contain internal null bytes.
    #[inline]
    pub fn x_ticks_with_labels(self, tick_labels: &[(f64, String)], show_default: bool) -> Self {
        self.axis_ticks_with_labels(AxisChoice::X1, tick_labels, show_default)
    }

    /// Set Y ticks with labels for the plot. The vector contains one position and label
    /// each in the form of a tuple `(label_position, label_string)`. The `show_default`
    /// setting determines whether the default ticks are also shown.
    ///
    /// # Panics
    /// Will panic if any of the tick label strings contain internal null bytes.
    #[inline]
    pub fn y_ticks_with_labels(self, tick_labels: &[(f64, String)], show_default: bool) -> Self {
        self.axis_ticks_with_labels(AxisChoice::Y1, tick_labels, show_default)
    }

    /// Set Y ticks with labels for the plot. The vector contains one position and label
    /// each in the form of a tuple `(label_position, label_string)`. The `show_default`
    /// setting determines whether the default ticks are also shown.
    ///
    /// # Panics
    /// Will panic if any of the tick label strings contain internal null bytes.
    #[inline]
    pub fn axis_ticks_with_labels(
        mut self,
        axis_choice: AxisChoice,
        tick_labels: &[(f64, String)],
        show_default: bool,
    ) -> Self {
        let axis_index = axis_choice as usize;
        self.axis_enabled[axis_index] = true;
        self.axis_tick_positions[axis_index] = Some(tick_labels.iter().map(|x| x.0).collect());
        self.axis_tick_labels[axis_index] = Some(
            tick_labels
                .iter()
                .map(|x| {
                    CString::new(x.1.as_str())
                        .unwrap_or_else(|_| panic!("String contains internal null bytes: {}", x.1))
                })
                .collect(),
        );
        self.show_axis_default_ticks[axis_index] = show_default;
        self
    }

    /// Set the plot flags, see the help for `PlotFlags` for what the available flags are
    #[inline]
    pub fn with_flags(mut self, flags: &PlotFlags) -> Self {
        self.plot_flags = flags.0 as sys::ImPlotFlags;
        self
    }

    /// Set the axis flags for the X axis in this plot
    #[inline]
    pub fn with_x1_flags(self, flags: &AxisFlags) -> Self {
        self.with_axis_flags(AxisChoice::X1, flags)
    }

    /// Set the axis flags for the selected Y axis in this plot
    #[inline]
    pub fn with_y1_flags(self, flags: &AxisFlags) -> Self {
        self.with_axis_flags(AxisChoice::Y1, flags)
    }

    /// Set the axis flags for the selected axis in this plot
    #[inline]
    pub fn with_axis_flags(mut self, axis_choice: AxisChoice, flags: &AxisFlags) -> Self {
        let axis_index = axis_choice as usize;
        self.axis_enabled[axis_index] = true;
        self.axis_flags[axis_index] = flags.0 as sys::ImPlotAxisFlags;
        self
    }

    /// Set the legend location, orientation and whether it is to be drawn outside the plot
    #[rustversion::attr(since(1.48), doc(alias = "SetupLegend"))]
    #[inline]
    pub fn with_legend_location(
        mut self,
        location: &PlotLocation,
        flags: &PlotLegendFlags,
    ) -> Self {
        self.legend_configuration = Some((*location, *flags));
        self
    }

    /// Set the axis scale for x1 in this plot
    #[inline]
    pub fn with_x1_scale(mut self, scale: &AxisScale) -> Self {
        let axis_index = AxisChoice::X1 as usize;
        self.axis_scales[axis_index] = *scale as sys::ImPlotScale;
        self
    }

    /// Set the axis scale for y1 in this plot
    #[inline]
    pub fn with_y1_scale(mut self, scale: &AxisScale) -> Self {
        let axis_index = AxisChoice::Y1 as usize;
        self.axis_scales[axis_index] = *scale as sys::ImPlotScale;
        self
    }

    /// Set the axis scale for the selected axis in this plot
    #[inline]
    pub fn with_axis_scale(mut self, axis_choice: AxisChoice, scale: &AxisScale) -> Self {
        let axis_index = axis_choice as usize;
        self.axis_enabled[axis_index] = true;
        self.axis_scales[axis_index] = *scale as sys::ImPlotScale;
        self
    }

    /// Internal helper function to set axis limits in case they are specified.
    fn maybe_set_axis_limits(&self) {
        // Limit-setting can either happen via direct limits or through linked limits. The version
        // of implot we link to here has different APIs for the two (separate per-axis calls for
        // direct, and one call for everything together for linked), hence the code here is a bit
        // clunky and takes the two approaches separately instead of a unified "match".

        for axis_index in 0..NUMBER_OF_AXES {
            let Some(limits) = self.axis_limits[axis_index].as_ref() else {
                continue;
            };
            match limits {
                AxisLimitSpecification::Single(limits, condition) => unsafe {
                    // --- Direct limit-setting ---
                    sys::ImPlot_SetupAxisLimits(
                        axis_index as ImAxis,
                        limits.Min,
                        limits.Max,
                        *condition as sys::ImPlotCond,
                    );
                },
                AxisLimitSpecification::Linked(range) => {
                    // --- Linked limit-setting ---
                    let mut borrowed = range.borrow_mut();
                    unsafe {
                        sys::ImPlot_SetupAxisLinks(
                            axis_index as ImAxis,
                            &mut borrowed.Min,
                            &mut borrowed.Max,
                        );
                    }
                }
            }
        }
    }

    /// Internal helper function to set tick labels in case they are specified. This does the
    /// preparation work that is the same for both the X and Y axis plots, then calls the
    /// "set next plot ticks" wrapper functions for both X and Y.
    fn maybe_set_tick_labels(&self) {
        self.axis_tick_positions
            .iter()
            .zip(self.axis_tick_labels.iter())
            .zip(self.show_axis_default_ticks.iter())
            .enumerate()
            .for_each(|(k, ((positions, labels), keep_default))| {
                if positions.is_some() && !positions.as_ref().unwrap().is_empty() {
                    // The vector of pointers we create has to have a longer lifetime
                    let mut pointer_vec;
                    let labels_pointer = if let Some(labels_value) = &labels {
                        pointer_vec = labels_value
                            .iter()
                            .map(|x| x.as_ptr() as *const c_char)
                            .collect::<Vec<*const c_char>>();
                        pointer_vec.as_mut_ptr()
                    } else {
                        std::ptr::null_mut()
                    };

                    unsafe {
                        sys::ImPlot_SetupAxisTicks_doublePtr(
                            k as ImAxis,
                            positions.as_ref().unwrap().as_ptr(),
                            positions.as_ref().unwrap().len() as i32,
                            labels_pointer,
                            *keep_default,
                        );
                    }
                }
            });
    }

    unsafe extern "C" fn axis_format_callback(
        value: f64,
        buff: *mut c_char,
        size: c_int,
        user_data: *mut c_void,
    ) -> c_int {
        let cb = user_data as *mut Box<FormatCallback>;

        let s = (*cb)(value);
        let s = s.as_bytes();

        let count = std::cmp::min(size.checked_sub(1).unwrap() as usize, s.len());
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), buff as *mut u8, count);
            *buff.add(count) = 0;
        };

        (count + 1) as c_int
    }

    /// Attempt to show the plot. If this returns a token, the plot will actually
    /// be drawn. In this case, use the drawing functionality to draw things on the
    /// plot, and then call `end()` on the token when done with the plot.
    /// If none was returned, that means the plot is not rendered.
    ///
    /// For a convenient implementation of all this, use [`build()`](struct.Plot.html#method.build)
    /// instead.
    #[rustversion::attr(since(1.48), doc(alias = "BeginPlot"))]
    pub fn begin(&self, plot_ui: &PlotUi) -> Option<PlotToken> {
        let should_render = unsafe {
            let size_vec: ImVec2 = ImVec2 {
                x: self.size[0],
                y: self.size[1],
            };
            sys::ImPlot_BeginPlot(self.title.as_ptr(), size_vec, self.plot_flags)
        };

        if should_render {
            for (axis, enabled) in self.axis_enabled.iter().enumerate() {
                if !enabled {
                    continue;
                }
                let ptr = self.labels[axis]
                    .as_ref()
                    .map_or_else(std::ptr::null, |s| s.as_ptr());
                unsafe {
                    sys::ImPlot_SetupAxis(axis as ImAxis, ptr, self.axis_flags[axis]);
                    sys::ImPlot_SetupAxisScale_PlotScale(axis as ImAxis, self.axis_scales[axis]);
                }

                if let Some(minmax) = self.axis_limits_constraints[axis] {
                    unsafe {
                        sys::ImPlot_SetupAxisLimitsConstraints(axis as ImAxis, minmax.0, minmax.1);
                    }
                }

                if let Some(minmax) = self.axis_zoom_constraints[axis] {
                    unsafe {
                        sys::ImPlot_SetupAxisZoomConstraints(axis as ImAxis, minmax.0, minmax.1);
                    }
                }

                if let Some(fmt) = self.axis_format[axis].as_ref() {
                    match fmt {
                        AxisFormat::FormatString(s) => unsafe {
                            sys::ImPlot_SetupAxisFormat_Str(axis as ImAxis, s.as_ptr());
                        },
                        AxisFormat::Callback(f) => {
                            let user_data = f.as_ref() as *const _ as *mut c_void;
                            unsafe {
                                sys::ImPlot_SetupAxisFormat_PlotFormatter(
                                    axis as ImAxis,
                                    Some(Self::axis_format_callback),
                                    user_data,
                                );
                            }
                        }
                    }
                }
            }

            self.maybe_set_axis_limits();
            self.maybe_set_tick_labels();

            // Configure legend location, if one was set. This has to be called between begin() and
            // end(), but since only the last call to it actually affects the outcome, I'm adding
            // it here instead of as a freestanding function. If this is too restrictive (for
            // example, if you want to set the location based on code running _during_ the plotting
            // for some reason), file an issue and we'll move it.
            if let Some(legend_config) = &self.legend_configuration {
                // We introduce variables with typechecks here to safeguard against accidental
                // changes in order in the config tuple
                let location: PlotLocation = legend_config.0;
                let flags: PlotLegendFlags = legend_config.1;
                unsafe {
                    sys::ImPlot_SetupLegend(location as ImPlotLocation, flags.0 as ImPlotFlags);
                }
            }

            Some(PlotToken {
                context: plot_ui.context,
                plot_title: self.title.clone(),
            })
        } else {
            // In contrast with imgui windows, end() does not have to be
            // called if we don't render. This is more like an imgui popup modal.
            None
        }
    }

    /// Creates a window and runs a closure to construct the contents. This internally
    /// calls `begin` and `end`.
    ///
    /// Note: the closure is not called if ImPlot::BeginPlot() returned
    /// false - TODO(4bb4) figure out if this is if things are not rendered
    #[rustversion::attr(since(1.48), doc(alias = "BeginPlot"))]
    #[rustversion::attr(since(1.48), doc(alias = "EndPlot"))]
    pub fn build<F: FnOnce(&PlotToken)>(self, plot_ui: &PlotUi, f: F) {
        if let Some(token) = self.begin(plot_ui) {
            f(&token);
            token.end()
        }
    }
}

/// Tracks a plot that must be ended by calling `.end()`
pub struct PlotToken {
    context: *const Context,
    /// For better error messages
    plot_title: CString,
}

pub type PlotDragToolFlags = sys::ImPlotDragToolFlags_;

impl PlotToken {
    /// End a previously begin()'ed plot.
    #[rustversion::attr(since(1.48), doc(alias = "EndPlot"))]
    pub fn end(mut self) {
        self.context = std::ptr::null();
        unsafe { sys::ImPlot_EndPlot() };
    }

    // --- Miscellaneous -----------------------------------------------------------------------------
    /// Returns true if the plot area in the current or most recent plot is hovered.
    #[rustversion::attr(since(1.48), doc(alias = "IsPlotHovered"))]
    pub fn is_plot_hovered(&self) -> bool {
        unsafe { sys::ImPlot_IsPlotHovered() }
    }

    /// Returns true if the user changed the coordinates.
    #[rustversion::attr(since(1.48), doc(alias = "DragRect"))]
    #[allow(clippy::too_many_arguments)]
    pub fn drag_rect(
        &self,
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

    /// Set the axis to be used for any upcoming plot elements
    #[rustversion::attr(since(1.48), doc(alias = "SetAxis"))]
    pub fn set_axis(&self, axis_choice: AxisChoice) {
        unsafe {
            sys::ImPlot_SetAxis(axis_choice as sys::ImAxis);
        }
    }

    /// Convert pixels, given as an `ImVec2`, to a position in the current plot's coordinate system.
    #[rustversion::attr(since(1.48), doc(alias = "PixelsToPlot"))]
    pub fn pixels_to_plot_vec2(
        &self,
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
        &self,
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
        &self,
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
        &self,
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

    /// Returns the current or most recent plot axis range for the specified choice of axes.
    #[rustversion::attr(since(1.48), doc(alias = "GetPlotLimits"))]
    pub fn get_plot_limits(
        &self,
        x_axis: Option<AxisChoice>,
        y_axis: Option<AxisChoice>,
    ) -> sys::ImPlotRect {
        let x_axis = x_axis.map_or_else(|| IMPLOT_AUTO as sys::ImAxis, |x| x as sys::ImAxis);
        let y_axis = y_axis.map_or_else(|| IMPLOT_AUTO as sys::ImAxis, |y| y as sys::ImAxis);

        // ImPlotLimits doesn't seem to have default()
        let mut limits = sys::ImPlotRect {
            X: ImPlotRange { Min: 0.0, Max: 0.0 },
            Y: ImPlotRange { Min: 0.0, Max: 0.0 },
        };
        unsafe {
            sys::ImPlot_GetPlotLimits(
                &mut limits as *mut sys::ImPlotRect,
                x_axis as sys::ImAxis,
                y_axis as sys::ImAxis,
            );
        }
        limits
    }

    /// Returns true if the axis plot area in the current plot is hovered.
    #[rustversion::attr(since(1.48), doc(alias = "IsAxisHovered"))]
    pub fn is_axis_hovered(&self, axis: AxisChoice) -> bool {
        unsafe { sys::ImPlot_IsAxisHovered(axis as sys::ImAxis) }
    }

    /// Returns true if the given item in the legend of the current plot is hovered.
    pub fn is_legend_entry_hovered(&self, legend_entry: &str) -> bool {
        unsafe { sys::ImPlot_IsLegendEntryHovered(legend_entry.as_ptr() as *const c_char) }
    }

    /// Returns the mouse position in x,y coordinates of the current or most recent plot,
    /// for the specified choice of axes.
    #[rustversion::attr(since(1.48), doc(alias = "GetPlotMousePos"))]
    pub fn get_plot_mouse_position(
        &self,
        x_axis: Option<AxisChoice>,
        y_axis: Option<AxisChoice>,
    ) -> ImPlotPoint {
        let mut point = ImPlotPoint { x: 0.0, y: 0.0 }; // doesn't seem to have default()
        let x_axis = x_axis.map_or_else(|| IMPLOT_AUTO as sys::ImAxis, |x| x as sys::ImAxis);
        let y_axis = y_axis.map_or_else(|| IMPLOT_AUTO as sys::ImAxis, |y| y as sys::ImAxis);
        unsafe {
            sys::ImPlot_GetPlotMousePos(&mut point as *mut ImPlotPoint, x_axis, y_axis);
        }
        point
    }

    pub fn hide_next_item(&self, hidden: bool, when: PlotCond) {
        unsafe {
            sys::ImPlot_HideNextItem(hidden, when as sys::ImPlotCond);
        }
    }

    pub fn annotation<S: Into<Vec<u8>>>(
        &self,
        x: f64,
        y: f64,
        color: Option<ImVec4>,
        pix_offset: ImVec2,
        clamp: bool,
        label: S,
    ) {
        let label = CString::new(label).unwrap();
        let color = color.unwrap_or(IMPLOT_AUTO_COL);
        unsafe { sys::ImPlot_Annotation_Str(x, y, color, pix_offset, clamp, label.as_ptr()) }
    }
}

impl Drop for PlotToken {
    fn drop(&mut self) {
        if !self.context.is_null() && !std::thread::panicking() {
            panic!(
                "Warning: A PlotToken for plot \"{:?}\" was not called end() on",
                self.plot_title
            );
        }
    }
}
