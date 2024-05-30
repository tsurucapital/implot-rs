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

pub type AxisChoice = sys::ImAxis_;
pub type Marker = sys::ImPlotMarker_;
pub type PlotColorElement = sys::ImPlotCol_;
pub type PlotLocation = sys::ImPlotLocation_;
pub type PlotLegendFlags = sys::ImPlotLegendFlags_;
