use crate::context::TreeProps;
use crate::prelude::*;

/// A view which represents a bar that can be dragged to manipulate a scrollview.
pub struct Scrollbar<L1> {
    value: L1,
    orientation: Orientation,

    reference_points: Option<(f32, f32)>,
    dragging: bool,

    on_changing: Option<Box<dyn Fn(&mut EventContext, f32)>>,

    scroll_to_cursor: bool,
}

enum ScrollBarEvent {
    SetScrollToCursor(bool),
}

impl<L1: Lens<Target = f32>> Scrollbar<L1> {
    /// Create a new [Scrollbar] view.
    pub fn new<F, L2: Lens<Target = f32>>(
        cx: &mut Context,
        value: L1,
        ratio: L2,
        orientation: Orientation,
        callback: F,
    ) -> Handle<Self>
    where
        F: 'static + Fn(&mut EventContext, f32),
    {
        Self {
            value,
            orientation,
            reference_points: None,
            on_changing: Some(Box::new(callback)),
            scroll_to_cursor: false,
            dragging: false,
        }
        .build(cx, move |cx| {
            Element::new(cx)
                .class("thumb")
                .focusable(true)
                .bind(value, move |handle, value| {
                    let value = value.get(&handle);
                    match orientation {
                        Orientation::Horizontal => {
                            handle.left(Units::Stretch(value)).right(Units::Stretch(1.0 - value))
                        }
                        Orientation::Vertical => {
                            handle.top(Units::Stretch(value)).bottom(Units::Stretch(1.0 - value))
                        }
                    };
                })
                .bind(ratio, move |handle, ratio| {
                    let ratio = ratio.get(&handle);
                    match orientation {
                        Orientation::Horizontal => handle.width(Units::Percentage(ratio * 100.0)),
                        Orientation::Vertical => handle.height(Units::Percentage(ratio * 100.0)),
                    };
                })
                .position_type(PositionType::Absolute);
        })
        .pointer_events(PointerEvents::Auto)
        .class(match orientation {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        })
    }

    fn container_and_thumb_size(&self, cx: &mut EventContext) -> (f32, f32) {
        let current = cx.current();
        let child = cx.tree.get_child(current, 0).unwrap();
        match &self.orientation {
            Orientation::Horizontal => (cx.cache.get_width(current), cx.cache.get_width(child)),
            Orientation::Vertical => (cx.cache.get_height(current), cx.cache.get_height(child)),
        }
    }

    fn thumb_bounds(&self, cx: &mut EventContext) -> BoundingBox {
        let child = cx.first_child();

        cx.with_current(child, |cx| cx.bounds())
    }

    fn compute_new_value(&self, cx: &mut EventContext, physical_delta: f32, value_ref: f32) -> f32 {
        // delta is moving within the negative space of the thumb: (1 - ratio) * container
        let (size, thumb_size) = self.container_and_thumb_size(cx);
        let negative_space = size - thumb_size;
        if negative_space == 0.0 {
            value_ref
        } else {
            // what percentage of negative space have we crossed?
            let logical_delta = physical_delta / negative_space;
            value_ref + logical_delta
        }
    }

    fn change(&mut self, cx: &mut EventContext, new_val: f32) {
        if let Some(callback) = &self.on_changing {
            callback(cx, new_val.clamp(0.0, 1.0));
        }
    }
}

impl<L1: 'static + Lens<Target = f32>> View for Scrollbar<L1> {
    fn element(&self) -> Option<&'static str> {
        Some("scrollbar")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|scrollbar_event, _| match scrollbar_event {
            ScrollBarEvent::SetScrollToCursor(flag) => {
                self.scroll_to_cursor = *flag;
            }
        });

        event.map(|window_event, meta| {
            let pos = match &self.orientation {
                Orientation::Horizontal => cx.mouse.cursor_x,
                Orientation::Vertical => cx.mouse.cursor_y,
            };
            match window_event {
                WindowEvent::MouseDown(MouseButton::Left) => {
                    if meta.target != cx.current() {
                        self.reference_points = Some((pos, self.value.get(cx)));
                        cx.capture();
                        cx.set_active(true);
                        self.dragging = true;
                        cx.with_current(Entity::root(), |cx| {
                            cx.set_pointer_events(false);
                        });
                    } else if self.scroll_to_cursor {
                        cx.capture();
                        cx.set_active(true);
                        self.dragging = true;
                        cx.with_current(Entity::root(), |cx| {
                            cx.set_pointer_events(false);
                        });
                        let thumb_bounds = self.thumb_bounds(cx);
                        let bounds = cx.bounds();
                        let sx = bounds.w - thumb_bounds.w;
                        let sy = bounds.h - thumb_bounds.h;
                        match self.orientation {
                            Orientation::Horizontal => {
                                let px = cx.mouse.cursor_x - cx.bounds().x - thumb_bounds.w / 2.0;
                                let x = (px / sx).clamp(0.0, 1.0);
                                if let Some(callback) = &self.on_changing {
                                    (callback)(cx, x);
                                }
                            }

                            Orientation::Vertical => {
                                let py = cx.mouse.cursor_y - cx.bounds().y - thumb_bounds.h / 2.0;
                                let y = (py / sy).clamp(0.0, 1.0);
                                if let Some(callback) = &self.on_changing {
                                    (callback)(cx, y);
                                }
                            }
                        }
                    } else {
                        let (_, jump) = self.container_and_thumb_size(cx);
                        // let (tx, ty, tw, th) = self.thumb_bounds(cx);
                        let t = self.thumb_bounds(cx);
                        let physical_delta = match &self.orientation {
                            Orientation::Horizontal => {
                                if cx.mouse.cursor_x < t.x {
                                    -jump
                                } else if cx.mouse.cursor_x >= t.x + t.w {
                                    jump
                                } else {
                                    return;
                                }
                            }
                            Orientation::Vertical => {
                                if cx.mouse.cursor_y < t.y {
                                    -jump
                                } else if cx.mouse.cursor_y >= t.y + t.h {
                                    jump
                                } else {
                                    return;
                                }
                            }
                        };
                        let changed =
                            self.compute_new_value(cx, physical_delta, self.value.get(cx));
                        self.change(cx, changed);
                    }
                }

                WindowEvent::MouseUp(MouseButton::Left) => {
                    self.reference_points = None;
                    cx.focus_with_visibility(false);
                    cx.release();
                    cx.set_active(false);
                    self.dragging = false;
                    cx.with_current(Entity::root(), |cx| {
                        cx.set_pointer_events(true);
                    });
                }

                WindowEvent::MouseMove(_, _) => {
                    if self.dragging {
                        if let Some((mouse_ref, value_ref)) = self.reference_points {
                            let physical_delta = pos - mouse_ref;
                            let changed = self.compute_new_value(cx, physical_delta, value_ref);
                            self.change(cx, changed);
                        } else if self.scroll_to_cursor {
                            let thumb_bounds = self.thumb_bounds(cx);
                            let bounds = cx.bounds();
                            let sx = bounds.w - thumb_bounds.w;
                            let sy = bounds.h - thumb_bounds.h;
                            match self.orientation {
                                Orientation::Horizontal => {
                                    let px =
                                        cx.mouse.cursor_x - cx.bounds().x - thumb_bounds.w / 2.0;
                                    let x = (px / sx).clamp(0.0, 1.0);
                                    if let Some(callback) = &self.on_changing {
                                        (callback)(cx, x);
                                    }
                                }

                                Orientation::Vertical => {
                                    let py =
                                        cx.mouse.cursor_y - cx.bounds().y - thumb_bounds.h / 2.0;
                                    let y = (py / sy).clamp(0.0, 1.0);
                                    if let Some(callback) = &self.on_changing {
                                        (callback)(cx, y);
                                    }
                                }
                            }
                        }
                    }
                }

                _ => {}
            }
        });
    }
}

impl<L1: 'static + Lens<Target = f32>> Handle<'_, Scrollbar<L1>> {
    /// Sets whether the scrollbar should move to the cursor when pressed.
    pub fn scroll_to_cursor(mut self, scroll_to_cursor: impl Res<bool>) -> Self {
        let entity = self.entity();
        scroll_to_cursor.set_or_bind(self.context(), entity, |cx, val| {
            let v = val.get(cx);
            cx.emit(ScrollBarEvent::SetScrollToCursor(v));
        });

        self
    }
}
