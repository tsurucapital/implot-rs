pub use self::{context::*, plot::*, plot_elements::*};
pub use implot_sys as sys;
pub use sys::{ImPlotPoint, ImPlotRange, ImVec2, ImVec4};

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
