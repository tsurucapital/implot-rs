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

#[rustversion::attr(since(1.48), doc(alias = "ImPlotLegendFlags"))]
/// Used to orient items on a plot (e.g. legends, labels, etc.)
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum PlotLegendFlags {
    // Default
    None = sys::ImPlotLegendFlags__ImPlotLegendFlags_None,
    // Legend icons will not function as hide/show buttons
    NoButtons = sys::ImPlotLegendFlags__ImPlotLegendFlags_NoButtons,
    // Plot items will not be highlighted when their legend entry is hovered
    NoHighlightItem = sys::ImPlotLegendFlags__ImPlotLegendFlags_NoHighlightItem,
    // Axes will not be highlighted when legend entries are hovered (only relevant if x/y-axis count > 1)
    NoHighlightAxis = sys::ImPlotLegendFlags__ImPlotLegendFlags_NoHighlightAxis,
    // The user will not be able to open context menus with right-click
    NoMenus = sys::ImPlotLegendFlags__ImPlotLegendFlags_NoMenus,
    // Legend will be rendered outside of the plot area
    Outside = sys::ImPlotLegendFlags__ImPlotLegendFlags_Outside,
    // Legend entries will be displayed horizontally
    Horizontal = sys::ImPlotLegendFlags__ImPlotLegendFlags_Horizontal,
    // Legend entries will be displayed in alphabetical order
    Sort = sys::ImPlotLegendFlags__ImPlotLegendFlags_Sort,
}
