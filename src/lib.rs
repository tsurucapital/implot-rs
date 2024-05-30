pub use self::{context::*, plot::*, plot_elements::*};
pub use implot_sys as sys;
pub use sys::{ImPlotPoint, ImPlotRange, ImVec2, ImVec4};

mod context;
mod plot;
mod plot_elements;

const NUMBER_OF_AXES: usize = sys::ImAxis__ImAxis_COUNT as usize;

#[rustversion::attr(since(1.48), doc(alias = "ImPlotAxis"))]
#[derive(Clone)]
#[repr(u32)]
pub enum AxisChoice {
    X1 = sys::ImAxis__ImAxis_X1,
    X2 = sys::ImAxis__ImAxis_X2,
    X3 = sys::ImAxis__ImAxis_X3,
    Y1 = sys::ImAxis__ImAxis_Y1,
    Y2 = sys::ImAxis__ImAxis_Y2,
    Y3 = sys::ImAxis__ImAxis_Y3,
}

pub struct PlotUi<'ui> {
    context: &'ui Context,
}

/// Used to position items on a plot (e.g. legends, labels, etc.)
#[rustversion::attr(since(1.48), doc(alias = "ImPlotLocation"))]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum PlotLocation {
    /// Center-center
    Center = sys::ImPlotLocation__ImPlotLocation_Center,
    /// Top-center
    North = sys::ImPlotLocation__ImPlotLocation_North,
    /// Bottom-center
    South = sys::ImPlotLocation__ImPlotLocation_South,
    /// Center-left
    West = sys::ImPlotLocation__ImPlotLocation_West,
    /// Center-right
    East = sys::ImPlotLocation__ImPlotLocation_East,
    /// Top-left
    NorthWest = sys::ImPlotLocation__ImPlotLocation_NorthWest,
    /// Top-right
    NorthEast = sys::ImPlotLocation__ImPlotLocation_NorthEast,
    /// Bottom-left
    SouthWest = sys::ImPlotLocation__ImPlotLocation_SouthWest,
    /// Bottom-right
    SouthEast = sys::ImPlotLocation__ImPlotLocation_SouthEast,
}

pub type PlotLegendFlags = sys::ImPlotLegendFlags_;
