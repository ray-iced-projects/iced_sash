//! A sash for resizing containers.

use iced::border::{Border, Radius};
use iced::event::Event;
use iced::advanced::layout;
use iced::window;
use iced::{Background, Element};
use iced::advanced::renderer;
use iced::touch;
use iced::advanced::widget::tree::{self, Tree};
use iced::{
    self, Color, Length, 
    Rectangle, Size, Theme,
};
use iced::advanced::{mouse, Layout, Shell, Widget};
use iced::advanced::Renderer as AdvancedRenderer;
use iced::Point;
use std::sync::atomic::{AtomicU64, Ordering};


pub fn sash_horizontal<'a, Message, Theme>(
    id: usize,
    widths: Vec<f32>,
    handle_width: f32,
    handle_height: f32,
    on_change: impl Fn((usize, usize, f32)) -> Message + 'a,
) -> Sash<'a, Message, Theme>
where
    Message: Clone,
    Theme: Catalog + 'a,
{
    let mut handle_offsets = vec![-handle_width/2.0; widths.len()-1];
        handle_offsets.extend([-handle_width]);
    
    Sash::new(
        id,
        widths, 
        handle_width, 
        handle_height,
        handle_offsets,
        Direction::Horizontal,
        on_change)
}

pub fn sash_vertical<'a, Message, Theme>(
    id: usize,
    heights: Vec<f32>,
    handle_width: f32,
    handle_height: f32,
    on_change: impl Fn((usize, usize, f32)) -> Message + 'a,
) -> Sash<'a, Message, Theme>
where
    Message: Clone,
    Theme: Catalog + 'a,
{
    let widths = heights;
    let mut handle_offsets = vec![-handle_height/2.0; widths.len()-1];
        // last offset pulled in to keep in bounds
        handle_offsets.extend([-handle_height]);
        
    Sash::new(
        id,
        widths, 
        handle_width, 
        handle_height,
        handle_offsets,
        Direction::Vertical,
        on_change)
}

pub struct Sash<'a, Message, Theme = iced::Theme>
where
    Theme: Catalog,
{
    id: usize,
    widths: Vec<f32>,
    handle_width: f32,
    handle_height: f32,
    on_change: Box<dyn Fn((usize, usize, f32)) -> Message + 'a>,
    on_release: Option<Message>,
    on_release_fn: Option<Box<dyn Fn((usize, usize)) -> Message + 'a>>,
    width: Length,
    height: Length,
    handle_offsets: Vec<f32>,
    include_last_handle: bool,
    direction: Direction,
    class: Theme::Class<'a>,
    /// Per-handle statuses, updated on RedrawRequested.
    statuses: Vec<Status>,
}

impl<'a, Message, Theme> Sash<'a, Message, Theme>
where
    Theme: Catalog,
{
    /// Sets the release message of the [`Sash`].
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets a release callback of the [`Sash`] that receives `(id, handle_index)`.
    /// Use this instead of [`on_release`] when you need to know which handle was released.
    pub fn on_release_fn(mut self, f: impl Fn((usize, usize)) -> Message + 'a) -> Self {
        self.on_release_fn = Some(Box::new(f));
        self
    }
    /// Sets the width of the [`Sash`] which usually spans the entire width of the items.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Sash`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the handle offsets for alignment of the [`Sash`].
    pub fn handle_offsets(mut self, handle_offsets: Vec<f32>) -> Self {
        self.handle_offsets = handle_offsets;
        self
    }

    /// Sets the include_last_handle of the [`Sash`].
    /// If not included, the total width or height will not change
    pub fn include_last_handle(mut self, include: bool) -> Self {
        self.include_last_handle = include;
        self
    }

    /// Sets the direction of the [`Sash`].
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the style of the [`Sash`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Sash`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

#[derive(Default)]
struct State {
    is_dragging: bool,
    index: usize,
    handle_bounds: Vec<Rectangle>,
    width_height_bounds: Vec<Rectangle>,
}

impl<'a, Message, Theme> Sash<'a, Message, Theme>
where
    Theme: Catalog,
{
    /// The default height of a [`Sash`].
    pub const DEFAULT_HEIGHT: f32 = 21.0;

    /// Creates a new [`Sash`].
    pub fn new<F>(
        id: usize,
        widths: Vec<f32>,
        handle_width: f32,
        handle_height: f32,
        handle_offsets: Vec<f32>,
        direction: Direction, 
        on_change: F) 
        -> Self
    where
        F: 'a + Fn((usize, usize, f32)) -> Message,
    {
        Sash {
            id,
            widths,
            handle_width,
            handle_height,
            on_change: Box::new(on_change),
            on_release: None,
            on_release_fn: None,
            width: Length::Fill,
            height: Length::Fill,
            handle_offsets,
            include_last_handle: true,
            direction,
            class: Theme::default(),
            statuses: vec![],
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Sash<'_, Message, Theme>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        update::<Message, Theme, Renderer>(self, tree, event, layout, cursor, shell);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let is_mouse_over = 
            find_mouse_over_handle_bounds(
                &state.handle_bounds,  
                cursor);

        if state.is_dragging || is_mouse_over.is_some(){
            match self.direction {
                Direction::Horizontal => mouse::Interaction::ResizingHorizontally,
                Direction::Vertical => mouse::Interaction::ResizingVertically,
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _renderer_style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        
        for i in 0..self.widths.len() {
            let status = self.statuses.get(i).copied().unwrap_or(Status::Active);
            let style = theme.style(&self.class, status);
            
            renderer.fill_quad(
                renderer::Quad {
                    bounds: state.width_height_bounds[i],
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
            // fill with the handle
            if !self.include_last_handle && i == self.widths.len()-1{
                break;
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds: state.handle_bounds[i],
                    border: Border {
                        radius: style.border_radius,
                        width: style.border_width,
                        color: style.border_color,
                    },
                    ..renderer::Quad::default()
                },
                style.background,
            );
        }
    }

}

impl<'a, Message, Theme, Renderer> From<Sash<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(
        sash: Sash<'a, Message, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(sash)
    }
}

/// Processes the given [`Event`] and updates the [`State`] of an [`Sash`]
/// accordingly.
fn update<Message: Clone, Theme, Renderer>(
    widget: &mut Sash<'_, Message, Theme>,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) 
where
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    
    let state = tree.state.downcast_mut::<State>();
    let is_dragging = state.is_dragging;
    let total_bounds = layout.bounds();
    
    // stores the state
    let mut widths = vec![];
    for width in widget.widths.iter() {
        match widget.direction {
            Direction::Horizontal => {
                widths.push(*width);
            },
            Direction::Vertical => {
                widths.push(*width);
            },
        }
    }
    state.handle_bounds = 
        get_handle_bounds(
            total_bounds,
            &widths,
            widget.handle_width, 
            widget.handle_height,
            &widget.handle_offsets,
            widget.include_last_handle,
            widget.direction);

    state.width_height_bounds =
        get_width_height_bounds(
            total_bounds,
            &widths,
            widget.handle_width, 
            widget.handle_height, 
            widget.direction);

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let index = 
                find_mouse_over_handle_bounds(
                    &state.handle_bounds, cursor);
            
            if index.is_some() {
                state.is_dragging = true;
                state.index = index.unwrap();
                shell.capture_event();
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. })
        | Event::Touch(touch::Event::FingerLost { .. }) => {
            if is_dragging {
                if let Some(f) = &widget.on_release_fn {
                    shell.publish(f((widget.id, state.index)));
                } else if let Some(on_release) = widget.on_release.clone() {
                    shell.publish(on_release);
                }
                state.is_dragging = false;
                state.index = 0;

                shell.request_redraw();
                shell.capture_event();
            }
        }
        Event::Mouse(mouse::Event::CursorMoved { position })
        | Event::Touch(touch::Event::FingerMoved { id: _, position }) => {
            if is_dragging {
                let end_x = total_bounds.x+total_bounds.width;
                let end_y = total_bounds.y+total_bounds.height;
                let handle_bounds = state.handle_bounds[state.index];
                let w_h_bounds = state.width_height_bounds[state.index];
                let handle_count = state.handle_bounds.len();
                let w_h_count = state.width_height_bounds.len();

                match widget.direction {
                    Direction::Horizontal => {
                        if (position.x - handle_bounds.x + handle_bounds.width/2.0).abs() > 0.99 {
                            let new_value = 
                                // Moving left
                                if position.x < w_h_bounds.x && state.index == 0 {

                                    state.handle_bounds[state.index].x = w_h_bounds.x;
                                    (state.index, 0.0)
                                } else 
                                // Moving left stopping at next sash
                                if state.index > 0 && position.x < state.handle_bounds[state.index-1].x {

                                    state.handle_bounds[state.index].x = state.handle_bounds[state.index-1].x;
                                    (state.index, 0.0)
                                } else
                                // Moving right: stop at next sash
                                if  state.index < handle_count-1 && (state.index < handle_count) && 
                                    (position.x > state.handle_bounds[state.index+1].x) {

                                    state.handle_bounds[state.index].x = state.handle_bounds[state.index+1].x;
                                    let new_value = (state.handle_bounds[state.index+1].x - w_h_bounds.x).round();
                                    (state.index, new_value)
                                } else 
                                // Moving right: last index and no sash at end
                                if (handle_count < w_h_count) && 
                                    (position.x > end_x-handle_bounds.width/2.0) {

                                    state.handle_bounds[state.index].x = end_x-handle_bounds.width/2.0;
                                    let new_value = (end_x-handle_bounds.width/2.0-w_h_bounds.x).round();
                                    (state.index, new_value)
                                }
                                    else {
                                    // moving
                                    state.handle_bounds[state.index].x = position.x;
                                    let new_value = (position.x - w_h_bounds.x).round();
                                    (state.index, new_value)
                                };
                            let new_value = (widget.id, new_value.0, new_value.1);
                            shell.publish((widget.on_change)(new_value));
                            shell.capture_event();
                        }
                    },
                    Direction::Vertical => {
                        if (position.y - handle_bounds.y + handle_bounds.height/2.0).abs() > 0.99 {
                            let new_value = 
                                // Moving up
                                if position.y < w_h_bounds.y && state.index == 0 {

                                    state.handle_bounds[state.index].y = w_h_bounds.y;
                                    (state.index, 0.0)
                                } else 
                                // Moving left stopping at next sash
                                if state.index > 0 && position.y < state.handle_bounds[state.index-1].y {

                                    state.handle_bounds[state.index].y = state.handle_bounds[state.index-1].y;
                                    (state.index, 0.0)
                                } else
                                // Moving right: stop at next sash
                                if  state.index < handle_count-1 && (state.index < handle_count) && 
                                    (position.y > state.handle_bounds[state.index+1].y) {

                                    state.handle_bounds[state.index].y = state.handle_bounds[state.index+1].y;
                                    let new_value = (state.handle_bounds[state.index+1].y - w_h_bounds.y).round();
                                    (state.index, new_value)
                                } else 
                                // Moving right: last index and no sash at end
                                if (handle_count < w_h_count) && 
                                    (position.y > end_y-handle_bounds.height/2.0) {
                                        
                                    state.handle_bounds[state.index].y = end_y-handle_bounds.height/2.0;
                                    let new_value = (end_y-handle_bounds.height/2.0-w_h_bounds.y).round();
                                    (state.index, new_value)
                                }
                                    else {
                                    // moving
                                    state.handle_bounds[state.index].y = position.y;
                                    let new_value = (position.y - w_h_bounds.y).round();
                                    (state.index, new_value)
                                };
                            let new_value = (widget.id, new_value.0, new_value.1);
                            shell.publish((widget.on_change)(new_value));
                            shell.capture_event();
                        }
                    },
                }
            }
        },
        _ => {}
    }

    // Compute the current per-handle status from live state + cursor.
    let is_mouse_over = find_mouse_over_handle_bounds(&state.handle_bounds, cursor);
    let current_statuses: Vec<Status> = (0..widget.widths.len())
        .map(|i| {
            if state.is_dragging && i == state.index {
                Status::Dragged
            } else if Some(i) == is_mouse_over {
                Status::Hovered
            } else {
                Status::Active
            }
        })
        .collect();

    if let Event::Window(window::Event::RedrawRequested(_)) = event {
        widget.statuses = current_statuses;
    } else if widget.statuses != current_statuses {
        shell.request_redraw();
    }
}

fn get_handle_bounds(
    bounds: Rectangle,
    widths_heights: &[f32],
    handle_width: f32,
    handle_height: f32,
    handle_offsets: &[f32],
    include_last_handle: bool,
    direction: Direction,
    ) -> Vec<Rectangle> 
{
    let mut handle_bounds = vec![];
    let mut start = match direction {
            Direction::Horizontal => bounds.x,
            Direction::Vertical => bounds.y,
        };
        for (i, width_height) in widths_heights.iter().enumerate() {
            
            if i == widths_heights.len()-1 {
                if include_last_handle {
                    start += width_height;
                } else {
                    break;
                }
            } else {
                start += width_height;
            }

            let rect = match direction {
                Direction::Horizontal => {
                    Rectangle{ 
                        x: start+handle_offsets[i], 
                        y: bounds.y, 
                        width: handle_width, 
                        height: handle_height,
                    }
                },
                Direction::Vertical => {
                    Rectangle{
                        x: bounds.x,
                        y: start+handle_offsets[i],
                        width: handle_width,
                        height: handle_height,
                    }
                },
            };
                
            handle_bounds.push(rect);

        }
        handle_bounds
}

fn get_width_height_bounds(
    bounds: Rectangle,
    widths_heights: &[f32],
    handle_width: f32,
    handle_height: f32,
    direction: Direction,
    ) -> Vec<Rectangle> 
{
    let mut w_h_bounds = vec![];
    let mut start = match direction {
            Direction::Horizontal => bounds.x,
            Direction::Vertical => bounds.y,
        };
        for width_height in widths_heights.iter() {
            let rect = match direction {
                Direction::Horizontal => {
                    Rectangle{ 
                        x: start, 
                        y: bounds.y, 
                        width: *width_height, 
                        height: handle_height,
                    }
                },
                Direction::Vertical => {
                    Rectangle{
                        x: bounds.x,
                        y: start,
                        width: handle_width,
                        height: *width_height,
                    }
                },
            };
                
            w_h_bounds.push(rect);

            match direction {
                Direction::Horizontal => {
                    start += width_height;
                },
                Direction::Vertical => {
                    start += width_height;
                },
            }
            
        }
        w_h_bounds
}

fn find_mouse_over_handle_bounds(
    handle_bounds: &[Rectangle],
    cursor: mouse::Cursor) 
    -> Option<usize> {
        for (index, bounds) in handle_bounds.iter().enumerate() {
            if cursor.is_over(*bounds) {
                return Some(index)
            }
        }
        None
}

/// The direction of [`Sash`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Direction {
    /// Horizontal resizing
    #[default]
    Horizontal,
    /// Vertical resizing
    Vertical,
}

/// The possible status of a [`Sash`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Sash`] can be interacted with.
    Active,
    /// The [`Sash`] is being hovered.
    Hovered,
    /// The [`Sash`] is being dragged.
    Dragged,
    /// The [`Sash`] is disabled.
    Disabled,
}

/// The appearance of a Sash.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the handle.
    pub background: Background,
    /// The border width of the handle.
    pub border_width: f32,
    /// The border [`Color`] of the handle.
    pub border_color: Color,
    /// The border [`Radius`] of the handle.
    pub border_radius: Radius,
}

/// The theme catalog of a [`Sash`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Sash`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(subtle)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Sash`].
pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let color = match status {
        Status::Active => palette.primary.base.color,
        Status::Hovered => palette.primary.weak.color,
        Status::Dragged => palette.primary.strong.color,
        Status::Disabled => palette.primary.weak.color,
    };

    Style {
        background: color.into(),
        border_color: Color::TRANSPARENT,
        border_width: 0.0,
        border_radius: 0.0.into()
    }
}

pub fn transparent(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();

    let color = match status {
        Status::Active => Color::TRANSPARENT,
        Status::Hovered => palette.background.weak.color,
        Status::Dragged => palette.background.weakest.color,
        Status::Disabled => palette.background.base.color,
    };

    Style {
        background: color.into(),
        border_color: Color::TRANSPARENT,
        border_width: 0.0,
        border_radius: 0.0.into()
    }
}

pub fn subtle(theme: &Theme, status: Status) -> Style {
    let palette = theme.palette();
    
    let color = match status {
        Status::Active => palette.background.weak.color,
        Status::Hovered => palette.background.strong.color,
        Status::Dragged => palette.background.strongest.color,
        Status::Disabled => palette.background.base.color,
    };

    Style {
        background: color.into(),
        border_color: Color::TRANSPARENT,
        border_width: 0.0,
        border_radius: 0.0.into()
    }
}

// ============================================================================
// Id
// ============================================================================

static NEXT_SASH_ID: AtomicU64 = AtomicU64::new(0);

/// A unique identifier for a [`SashH`] or [`SashV`] widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    /// Generates a new unique [`Id`] using an atomic counter.
    pub fn unique() -> Self {
        Id(NEXT_SASH_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Creates an [`Id`] from a specific number.
    pub fn new(n: u64) -> Self {
        Id(n)
    }
}

// ============================================================================
// resize() — size-conservation helper (available for manual use)
// ============================================================================

/// Applies size-conservation resize math to a panel sizes vector.
///
/// The panel at `index` is set to `value`; the adjacent panel absorbs the
/// difference. `min_size` clamps both panels — pass `0.0` for no minimum.
pub fn resize(sizes: &mut Vec<f32>, index: usize, value: f32, min_size: f32) {
    if index >= sizes.len() {
        return;
    }
    let panel_count = sizes.len();
    let value = value.max(min_size);
    let diff = sizes[index] - value;
    if index + 1 < panel_count {
        let next_ideal = sizes[index + 1] + diff;
        let next_actual = next_ideal.max(min_size);
        let excess = (next_actual - next_ideal).max(0.0);
        sizes[index] = (value - excess).max(min_size);
        sizes[index + 1] = next_actual;
    } else {
        sizes[index] = value;
    }
}

// ============================================================================
// Shared helpers
// ============================================================================

fn apply_max_size(sizes: &[f32], max_size: Option<f32>) -> Vec<f32> {
    let total: f32 = sizes.iter().sum();
    match max_size {
        Some(max) if total > max && total > 0.0 => {
            sizes.iter().map(|s| s * max / total).collect()
        }
        _ => sizes.to_vec(),
    }
}

fn max_size_scale(sizes: &[f32], max_size: Option<f32>) -> f32 {
    let total: f32 = sizes.iter().sum();
    match max_size {
        Some(max) if total > max && total > 0.0 => total / max,
        _ => 1.0,
    }
}

// ============================================================================
// Axis
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn cursor_coord(self, p: Point) -> f32 {
        match self { Axis::Horizontal => p.x, Axis::Vertical => p.y }
    }
    fn bounds_end(self, b: Rectangle) -> f32 {
        match self { Axis::Horizontal => b.x + b.width, Axis::Vertical => b.y + b.height }
    }
    fn main_start(self, r: Rectangle) -> f32 {
        match self { Axis::Horizontal => r.x, Axis::Vertical => r.y }
    }
    fn handle_main_size(self, r: Rectangle) -> f32 {
        match self { Axis::Horizontal => r.width, Axis::Vertical => r.height }
    }
    // Returns (handle_width, handle_height) for get_handle_bounds / get_width_height_bounds.
    fn handle_dims(self, sash_size: f32, cross_size: f32) -> (f32, f32) {
        match self {
            Axis::Horizontal => (sash_size, cross_size),
            Axis::Vertical   => (cross_size, sash_size),
        }
    }
    fn child_limit(self, panel_size: f32, cross_size: f32) -> Size {
        match self {
            Axis::Horizontal => Size::new(panel_size, cross_size),
            Axis::Vertical   => Size::new(cross_size, panel_size),
        }
    }
    fn child_offset(self, main: f32) -> Point {
        match self {
            Axis::Horizontal => Point::new(main, 0.0),
            Axis::Vertical   => Point::new(0.0, main),
        }
    }
    fn total_size(self, main: f32, cross_size: f32) -> Size {
        match self {
            Axis::Horizontal => Size::new(main, cross_size),
            Axis::Vertical   => Size::new(cross_size, main),
        }
    }
    fn direction(self) -> Direction {
        match self {
            Axis::Horizontal => Direction::Horizontal,
            Axis::Vertical   => Direction::Vertical,
        }
    }
    fn resize_interaction(self) -> mouse::Interaction {
        match self {
            Axis::Horizontal => mouse::Interaction::ResizingHorizontally,
            Axis::Vertical   => mouse::Interaction::ResizingVertically,
        }
    }
}

// ============================================================================
// Shared state
// ============================================================================

struct SashState {
    id: Id,
    sizes: Vec<f32>,
    is_dragging: bool,
    drag_index: usize,
    hovered: Option<usize>,
}

// ============================================================================
// SashWidget — single implementation for both orientations
// ============================================================================

/// A resizable panel widget. Construct with [`SashH`] or [`SashV`].
pub struct SashWidget<'a, Message, Theme = iced::Theme>
where
    Theme: Catalog,
{
    children: Vec<Element<'a, Message, Theme>>,
    initial_sizes: Vec<f32>,
    cross_size: f32,
    sash_size: f32,
    axis: Axis,
    id: Id,
    max_size: Option<f32>,
    min_size: f32,
    on_resize: Option<Box<dyn Fn(Id, usize, f32) -> Message + 'a>>,
    on_release: Option<Box<dyn Fn(Id, usize) -> Message + 'a>>,
    sync_sizes: Option<Vec<f32>>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme> SashWidget<'a, Message, Theme>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
{
    /// Overrides the auto-generated [`Id`]. Only needed for multi-sash routing.
    pub fn id(mut self, id: Id) -> Self { self.id = id; self }

    /// Maximum total size; panels scale proportionally when exceeded.
    pub fn max_size(mut self, max: f32) -> Self { self.max_size = Some(max); self }

    /// Minimum panel size enforced while dragging. Default: `0.0`.
    pub fn min_size(mut self, min: f32) -> Self { self.min_size = min; self }

    /// Optional notification fired on every drag tick: `(id, handle_index, new_size)`.
    pub fn on_resize(mut self, f: impl Fn(Id, usize, f32) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(f)); self
    }

    /// Optional notification fired on mouse release: `(id, handle_index)`.
    pub fn on_release(mut self, f: impl Fn(Id, usize) -> Message + 'a) -> Self {
        self.on_release = Some(Box::new(f)); self
    }

    /// Sets the visual style of the sash handles.
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into(); self
    }

    /// Pushes external sizes into tree state each layout pass.
    /// Use this to synchronise two or more sashes from `on_resize` callbacks.
    pub fn sync_sashes(mut self, sizes: Vec<f32>) -> Self {
        self.sync_sizes = Some(sizes); self
    }
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer>
    for SashWidget<'_, Message, Theme>
where
    Message: Clone,
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag { tree::Tag::of::<SashState>() }

    fn state(&self) -> tree::State {
        tree::State::new(SashState {
            id: self.id,
            sizes: self.initial_sizes.clone(),
            is_dragging: false,
            drag_index: 0,
            hovered: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size { width: Length::Shrink, height: Length::Shrink }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        if let Some(new) = &self.sync_sizes {
            let st = tree.state.downcast_mut::<SashState>();
            if &st.sizes != new { st.sizes = new.clone(); }
        }
        let display = {
            let st = tree.state.downcast_ref::<SashState>();
            let s = if st.sizes.is_empty() { &self.initial_sizes } else { &st.sizes };
            apply_max_size(s, self.max_size)
        };

        let ax = self.axis;
        let mut child_nodes = Vec::with_capacity(self.children.len());
        let mut main = 0.0_f32;
        for (i, child) in self.children.iter_mut().enumerate() {
            let panel_size = display.get(i).copied().unwrap_or(0.0);
            let lim = layout::Limits::new(Size::ZERO, ax.child_limit(panel_size, self.cross_size));
            let node = child
                .as_widget_mut()
                .layout(&mut tree.children[i], renderer, &lim)
                .move_to(ax.child_offset(main));
            child_nodes.push(node);
            main += panel_size;
        }
        layout::Node::with_children(ax.total_size(main, self.cross_size), child_nodes)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, child_layout), child_tree) in self
            .children.iter()
            .zip(layout.children())
            .zip(tree.children.iter())
        {
            child.as_widget().draw(child_tree, renderer, theme, style, child_layout, cursor, viewport);
        }

        let ax = self.axis;
        let st = tree.state.downcast_ref::<SashState>();
        let display = apply_max_size(&st.sizes, self.max_size);
        let bounds = layout.bounds();
        let mut offsets = vec![-self.sash_size / 2.0; display.len().saturating_sub(1)];
        offsets.push(-self.sash_size);
        let (hw, hh) = ax.handle_dims(self.sash_size, self.cross_size);
        let hbs = get_handle_bounds(bounds, &display, hw, hh, &offsets, false, ax.direction());
        let hover = st.hovered;
        for (i, hb) in hbs.iter().enumerate() {
            let status = if st.is_dragging && i == st.drag_index { Status::Dragged }
                else if Some(i) == hover { Status::Hovered }
                else { Status::Active };
            let sty = theme.style(&self.class, status);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: *hb,
                    border: Border { radius: sty.border_radius, width: sty.border_width, color: sty.border_color },
                    ..renderer::Quad::default()
                },
                sty.background,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, child_layout), child_tree) in self
            .children.iter_mut()
            .zip(layout.children())
            .zip(tree.children.iter_mut())
        {
            child.as_widget_mut().update(child_tree, event, child_layout, cursor, renderer, shell, viewport);
        }

        let ax = self.axis;
        let bounds = layout.bounds();
        let st = tree.state.downcast_mut::<SashState>();
        let is_dragging = st.is_dragging;

        let display = apply_max_size(&st.sizes, self.max_size);
        let scale = max_size_scale(&st.sizes, self.max_size);
        let end = ax.bounds_end(bounds);
        let mut offsets = vec![-self.sash_size / 2.0; display.len().saturating_sub(1)];
        offsets.push(-self.sash_size);
        let (hw, hh) = ax.handle_dims(self.sash_size, self.cross_size);
        let hbs = get_handle_bounds(bounds, &display, hw, hh, &offsets, false, ax.direction());
        let pbs = get_width_height_bounds(bounds, &display, hw, hh, ax.direction());

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(idx) = find_mouse_over_handle_bounds(&hbs, cursor) {
                    st.is_dragging = true;
                    st.drag_index = idx;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if is_dragging {
                    let id = st.id;
                    if let Some(f) = &self.on_release { shell.publish(f(id, st.drag_index)); }
                    st.is_dragging = false;
                    st.drag_index = 0;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { id: _, position }) => {
                if is_dragging {
                    let idx = st.drag_index;
                    let id = st.id;
                    let hb = hbs[idx];
                    let pb = pbs[idx];
                    let hc = hbs.len();
                    let pc = pbs.len();
                    let pos = ax.cursor_coord(*position);
                    let pb_start = ax.main_start(pb);
                    let hb_main = ax.handle_main_size(hb);
                    let v = if pos < pb_start && idx == 0 {
                        0.0_f32
                    } else if idx > 0 && pos < ax.main_start(hbs[idx - 1]) {
                        0.0_f32
                    } else if idx < hc - 1 && pos > ax.main_start(hbs[idx + 1]) {
                        (ax.main_start(hbs[idx + 1]) - pb_start).round()
                    } else if hc < pc && pos > end - hb_main / 2.0 {
                        (end - hb_main / 2.0 - pb_start).round()
                    } else {
                        (pos - pb_start).round()
                    };
                    resize(&mut st.sizes, idx, v * scale, self.min_size);
                    if let Some(f) = &self.on_resize { shell.publish(f(id, idx, v * scale)); }
                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                } else {
                    let new_hover = find_mouse_over_handle_bounds(&hbs, cursor);
                    if new_hover != st.hovered {
                        st.hovered = new_hover;
                        shell.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let ax = self.axis;
        let st = tree.state.downcast_ref::<SashState>();
        if st.is_dragging { return ax.resize_interaction(); }
        let display = apply_max_size(&st.sizes, self.max_size);
        let bounds = layout.bounds();
        let mut offsets = vec![-self.sash_size / 2.0; display.len().saturating_sub(1)];
        offsets.push(-self.sash_size);
        let (hw, hh) = ax.handle_dims(self.sash_size, self.cross_size);
        let hbs = get_handle_bounds(bounds, &display, hw, hh, &offsets, false, ax.direction());
        if find_mouse_over_handle_bounds(&hbs, cursor).is_some() {
            return ax.resize_interaction();
        }
        self.children.iter().zip(layout.children()).zip(tree.children.iter())
            .map(|((c, l), t)| c.as_widget().mouse_interaction(t, l, cursor, viewport, renderer))
            .max()
            .unwrap_or_default()
    }
}

impl<'a, Message, Theme> From<SashWidget<'a, Message, Theme>>
    for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
{
    fn from(w: SashWidget<'a, Message, Theme>) -> Self {
        Element::new(w)
    }
}

// Type aliases preserve the existing public API.
pub type SashHWidget<'a, Message, Theme = iced::Theme> = SashWidget<'a, Message, Theme>;
pub type SashVWidget<'a, Message, Theme = iced::Theme> = SashWidget<'a, Message, Theme>;

// ============================================================================
// SashH / SashV — constructor namespaces
// ============================================================================

/// Namespace for constructing horizontal sash widgets.
pub struct SashH;

impl SashH {
    /// Creates a horizontal sash widget.
    ///
    /// - `children` — panel contents; one per entry in `initial_sizes`.
    /// - `initial_sizes` — starting widths used on the first render only.
    /// - `height` — shared panel height in pixels.
    /// - `sash_size` — thickness of each vertical drag handle in pixels.
    pub fn new<'a, Message, Theme>(
        children: Vec<Element<'a, Message, Theme>>,
        initial_sizes: Vec<f32>,
        height: f32,
        sash_size: f32,
    ) -> SashWidget<'a, Message, Theme>
    where
        Message: Clone + 'a,
        Theme: Catalog + 'a,
    {
        SashWidget {
            children,
            initial_sizes,
            cross_size: height,
            sash_size,
            axis: Axis::Horizontal,
            id: Id::unique(),
            max_size: None,
            min_size: 0.0,
            on_resize: None,
            on_release: None,
            sync_sizes: None,
            class: Theme::default(),
        }
    }
}

/// Namespace for constructing vertical sash widgets.
pub struct SashV;

impl SashV {
    /// Creates a vertical sash widget.
    ///
    /// - `children` — panel contents; one per entry in `initial_sizes`.
    /// - `initial_sizes` — starting heights used on the first render only.
    /// - `width` — shared panel width in pixels.
    /// - `sash_size` — thickness of each horizontal drag handle in pixels.
    pub fn new<'a, Message, Theme>(
        children: Vec<Element<'a, Message, Theme>>,
        initial_sizes: Vec<f32>,
        width: f32,
        sash_size: f32,
    ) -> SashWidget<'a, Message, Theme>
    where
        Message: Clone + 'a,
        Theme: Catalog + 'a,
    {
        SashWidget {
            children,
            initial_sizes,
            cross_size: width,
            sash_size,
            axis: Axis::Vertical,
            id: Id::unique(),
            max_size: None,
            min_size: 0.0,
            on_resize: None,
            on_release: None,
            sync_sizes: None,
            class: Theme::default(),
        }
    }
}

