use std::ffi::CString;

pub use self::{context::*, plot::*, plot_elements::*};
pub use implot_sys as sys;
pub use sys::{ImPlotColormap, ImPlotPoint, ImPlotRange, ImPlotRect, ImVec2, ImVec4};

mod context;
mod plot;
mod plot_elements;
mod tokens;

const NUMBER_OF_AXES: usize = sys::ImAxis_::COUNT as usize;

pub struct PlotUi<'ui> {
    context: &'ui Context,
}

impl PlotUi<'_> {
    /// Switch to a colormap preset.
    #[rustversion::attr(since(1.48), doc(alias = "PushColormap"))]
    pub fn push_colormap_from_preset(&self, colormap: ColormapPreset) -> ColormapToken {
        unsafe {
            sys::ImPlot_PushColormap_PlotColormap(colormap as sys::ImPlotColormap);
        }
        ColormapToken::new(self)
    }

    /// Switch to a different colormap.
    #[rustversion::attr(since(1.48), doc(alias = "PushColormap"))]
    pub fn push_colormap(&self, colormap: Colormap) -> ColormapToken {
        unsafe {
            sys::ImPlot_PushColormap_PlotColormap(colormap.to_index());
        }
        ColormapToken::new(self)
    }

    /// Switch to a colormap by name.
    #[rustversion::attr(since(1.48), doc(alias = "PushColormap"))]
    pub fn push_colormap_from_name(&self, name: &str) -> ColormapToken {
        let name = CString::new(name).unwrap();
        unsafe {
            sys::ImPlot_PushColormap_Str(name.as_ptr());
        }
        ColormapToken::new(self)
    }

    /// Push a f32 style variable to the stack. The returned token is used for removing
    /// the variable from the stack again:
    /// ```no_run
    /// # use implot::{PlotUi, StyleVar};
    /// # let plot_ui: PlotUi = todo!();
    /// let pushed_var = plot_ui.push_style_var_f32(&StyleVar::LineWeight, 11.0);
    /// // Plot some things
    /// std::mem::drop(pushed_var);
    /// ```
    #[rustversion::attr(since(1.48), doc(alias = "PushStyleVar"))]
    pub fn push_style_var_f32(&self, element: &StyleVar, value: f32) -> StyleVarToken {
        unsafe {
            sys::ImPlot_PushStyleVar_Float(*element as sys::ImPlotStyleVar, value);
        }
        StyleVarToken::new(self)
    }

    /// Push an u32 style variable to the stack. The only i32 style variable is Marker
    /// at the moment, for that, use something like
    /// ```no_run
    /// # use implot::{PlotUi, StyleVar, Marker};
    /// # let plot_ui: PlotUi = todo!();
    /// let markerchoice = plot_ui.push_style_var_i32(&StyleVar::Marker, Marker::Cross as i32);
    /// // plot things
    /// std::mem::drop(markerchoice)
    /// ```
    #[rustversion::attr(since(1.48), doc(alias = "PushStyleVar"))]
    pub fn push_style_var_i32(&self, element: &StyleVar, value: i32) -> StyleVarToken {
        unsafe {
            sys::ImPlot_PushStyleVar_Int(*element as sys::ImPlotStyleVar, value);
        }
        StyleVarToken::new(self)
    }

    /// Push an ImVec2 style variable to the stack. The returned token is used for removing
    /// the variable from the stack again.
    pub fn push_style_var_imvec2(&self, element: &StyleVar, value: ImVec2) -> StyleVarToken {
        unsafe {
            sys::ImPlot_PushStyleVar_Vec2(*element as sys::ImPlotStyleVar, value);
        }
        StyleVarToken::new(self)
    }

    // --- Push/pop utils -------------------------------------------------------------------------
    // Currently not in a struct yet. imgui-rs has some smarts about dealing with stacks, in particular
    // leak detection, which I'd like to replicate here at some point.
    /// Push a style color to the stack, giving an element and the four components of the color.
    /// The components should be between 0.0 (no intensity) and 1.0 (full intensity).
    /// The return value is a token that gets used for removing the style color from the stack again:
    /// ```no_run
    /// # use implot::{PlotUi, PlotColorElement};
    /// # let plot_ui: PlotUi = todo!();
    /// let pushed_var = plot_ui.push_style_color(&PlotColorElement::Line, 1.0, 1.0, 1.0, 0.2);
    /// // Plot some things
    /// std::mem::drop(pushed_var);
    /// ```
    #[rustversion::attr(since(1.48), doc(alias = "PushStyleColor"))]
    pub fn push_style_color(
        &self,
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
        StyleColorToken::new(self)
    }

    /// Get index of the given colormap
    pub fn get_colormap_index(&self, name: &str) -> Option<Colormap> {
        let name = CString::new(name).unwrap();
        let index = unsafe { sys::ImPlot_GetColormapIndex(name.as_ptr()) };
        if index >= 0 {
            Some(Colormap::Custom(index))
        } else {
            None
        }
    }

    /// Set a custom colormap in the form of a vector of colors.
    #[rustversion::attr(since(1.48), doc(alias = "AddColormap"))]
    pub fn add_colormap_from_vec(
        &self,
        name: &str,
        colors: Vec<ImVec4>,
        discrete: bool,
    ) -> Colormap {
        let name = CString::new(name).unwrap();
        let index = unsafe {
            sys::ImPlot_AddColormap_Vec4Ptr(
                name.as_ptr(),
                colors.as_ptr(),
                colors.len() as i32,
                discrete,
            )
        };
        Colormap::Custom(index)
    }

    // --- Demo window -------------------------------------------------------------------------------
    /// Show the demo window for poking around what functionality implot has to
    /// offer. Note that not all of this is necessarily implemented in implot-rs
    /// already - if you find something missing you'd really like, raise an issue.
    // This requires implot_demo.cpp to be in the list of sources in implot-sys.
    #[rustversion::attr(since(1.48), doc(alias = "ShowDemoWindow"))]
    pub fn show_demo_window(&self, show: &mut bool) {
        unsafe {
            implot_sys::ImPlot_ShowDemoWindow(show);
        }
    }
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
pub type ColormapPreset = sys::ImPlotColormap_;

/// Style variable choice, as in "which thing will be affected by a style setting".
pub type StyleVar = sys::ImPlotStyleVar_;

/// Used to position items on a plot (e.g. legends, labels, etc.)
pub type PlotLocation = sys::ImPlotLocation_;

/// Used to hide/show legends, shoe them horizontally, etc.
pub type PlotLegendFlags = sys::ImPlotLegendFlags_;

pub enum Colormap {
    Preset(ColormapPreset),
    Custom(i32),
}

impl Colormap {
    fn to_index(&self) -> sys::ImPlotColormap {
        match self {
            Colormap::Preset(preset) => *preset as sys::ImPlotColormap,
            Colormap::Custom(custom) => *custom as sys::ImPlotColormap,
        }
    }
}

create_token!(
    /// Tracks a colormap token that can be ended by calling `.end()`
    /// or by dropping
    pub struct ColormapToken<'ui>;

    /// Ends a main menu bar
    drop { sys::ImPlot_PopColormap(1) }
);

create_token!(
    /// Tracks a style color token that can be ended by calling `.end()`
    /// or by dropping
    pub struct StyleColorToken<'ui>;

    /// Ends a main menu bar
    drop { sys::ImPlot_PopStyleColor(1) }
);

create_token!(
    /// Tracks a style var token that can be ended by calling `.end()`
    /// or by dropping
    pub struct StyleVarToken<'ui>;

    /// Ends a main menu bar
    drop { sys::ImPlot_PopStyleVar(1) }
);
