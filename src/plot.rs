use std::rc::Rc;

use rust_expression::{Area, Graph, Number};
use seed::prelude::*;
use seed::*;

use web_sys::{HtmlCanvasElement, TouchList, MouseEvent, Touch, TouchEvent, WheelEvent};

use crate::Message;

#[derive(Debug, Clone)]
pub enum MouseMessage {
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseMove(MouseEvent),
    Wheel(WheelEvent),
}

#[derive(Debug, Clone)]
pub enum TouchMessage {
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
    TouchCancel(TouchEvent),
    TouchMove(TouchEvent),
}

#[derive(Debug)]
struct TouchPoint {
    id: i32,
    x: i32,
    y: i32,
}

impl TouchPoint {
    fn new(touch: &Touch) -> TouchPoint {
        let id = touch.identifier();
        let x = touch.client_x();
        let y = touch.client_y();
        TouchPoint { id, x, y }
    }
}

#[derive(Debug)]
enum TouchState {
    None,
    Move(TouchPoint),
    Zoom(TouchPoint, TouchPoint),
}

impl TouchState {
    fn new(tl: TouchList) -> TouchState {
        match tl.length() {
            1 => TouchState::Move(TouchPoint::new(&tl.item(0).unwrap())),
            2 => {
                let p1 = TouchPoint::new(&tl.item(0).unwrap());
                let p2 = TouchPoint::new(&tl.item(1).unwrap());
                if p1.id < p2.id {
                    TouchState::Zoom(p1, p2)
                } else {
                    TouchState::Zoom(p2, p1)
                }
            },
            _ => TouchState::None
        }
    }

    fn get_distance(&self) -> Option<Number> {
        match self {
            TouchState::Zoom(tp1, tp2) => {
                let x: Number = (tp1.x - tp2.x).into();
                let y: Number = (tp1.y - tp2.y).into();
                Some((x.powi(2) + y.powi(2)).sqrt())
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
enum TouchEffect {
    None,
    Move(Number, Number),
    Zoom(Number)
}

impl TouchEffect {
    fn new(previous: &TouchState, current: &TouchState) -> TouchEffect {
        match previous {
            TouchState::Move(tp_previous) => {
                match current {
                    TouchState::Move(tp_current) if tp_previous.id == tp_current.id => {
                        let x= tp_previous.x - tp_current.x;
                        let y= tp_previous.y - tp_current.y;
                        TouchEffect::Move(x.into(), y.into())
                    },
                    _ => TouchEffect::None
                }
            },
            TouchState::Zoom(tp1_previous, tp2_previous) => {
                match current {
                    TouchState::Zoom(tp1_current, tp2_current) if (tp1_previous.id == tp1_current.id) && (tp2_previous.id == tp2_current.id) => {
                        let prev_dist = previous.get_distance().unwrap();
                        let curr_dist = current.get_distance().unwrap();
                        TouchEffect::Zoom(prev_dist / curr_dist)
                    },
                    _ => TouchEffect::None
                }
            },
            TouchState::None => TouchEffect::None,
        }
    }
}

#[derive(Debug)]
pub struct PlotElement {
    graph: Graph,
    canvas: ElRef<HtmlCanvasElement>,
    screen: Area,
    area: Area,
    touch_state: TouchState,
}

macro_rules! map_callback_return_to_option_ms {
    ($cb_type:ty, $callback:expr, $panic_text:literal, $output_type:tt) => {{
        let t_type = std::any::TypeId::of::<MsU>();
        if t_type == std::any::TypeId::of::<Ms>() {
            $output_type::new(move |value| {
                (&mut Some($callback(value)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Ms>>() {
            $output_type::new(move |value| {
                (&mut $callback(value) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Ms>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            $output_type::new(move |value| {
                $callback(value);
                None
            }) as $output_type<$cb_type>
        } else {
            panic!($panic_text);
        }
    }};
}

#[allow(clippy::shadow_unrelated)]
pub fn wheel_ev<Ms: 'static, MsU: 'static>(
    trigger: impl Into<Ev>,
    handler: impl FnOnce(web_sys::WheelEvent) -> MsU + 'static + Clone,
) -> EventHandler<Ms> {
    let handler = map_callback_return_to_option_ms!(
        dyn Fn(web_sys::WheelEvent) -> Option<Ms>,
        handler.clone(),
        "Handler can return only Msg, Option<Msg> or ()!",
        Rc
    );
    let handler = move |event: web_sys::Event| {
        handler(event.dyn_ref::<web_sys::WheelEvent>().unwrap().clone())
    };
    EventHandler::new(trigger, handler)
}

const ZOOM_FACTOR_IN: Number = 0.8;
const ZOOM_FACTOR_OUT: Number = 1.2;

impl PlotElement {
    pub fn new(graph: Graph) -> PlotElement {
        PlotElement {
            graph,
            canvas: ElRef::default(),
            screen: Area::new(0., 0., 400., 300.),
            area: Area::new(-100., -100., 100., 100.),
            touch_state: TouchState::None,
        }
    }

    fn process_touch_intern(&mut self, e: TouchEvent) -> bool {
        let prev = &self.touch_state;
        let curr = TouchState::new(e.target_touches());
        let touch_effect = TouchEffect::new(prev, &curr);
        self.touch_state = curr;
        match touch_effect {
            TouchEffect::Move(x, y) => {
                self.move_by(-x, -y);
                true
            },
            TouchEffect::Zoom(factor) => {
                self.area.zoom_by(factor);
                true
            },
            TouchEffect::None => false
        }
    }

    pub fn process_touch(&mut self, e: TouchMessage) -> bool {
        match e {
            TouchMessage::TouchStart(e) => {
                self.process_touch_intern(e)
            },
            TouchMessage::TouchEnd(e) => {
                self.process_touch_intern(e)
            },
            TouchMessage::TouchCancel(e) => {
                self.process_touch_intern(e)
            },
            TouchMessage::TouchMove(e) => {
                self.process_touch_intern(e)
            },
        }
    }

    fn move_by(&mut self, x_delta: Number, y_delta: Number) {
        let x_delta = -x_delta;
        let x_delta = x_delta * self.area.x.get_distance() / self.screen.x.get_distance();
        let y_delta = y_delta * self.area.y.get_distance() / self.screen.y.get_distance();
        self.area.move_by(x_delta, y_delta);
    }

    pub fn process_mouse(&mut self, m: MouseMessage) -> bool {
        match m {
            MouseMessage::MouseMove(e) => {
                if e.buttons() == 1 {
                    self.move_by(e.movement_x().into(), e.movement_y().into());
                    true
                } else {
                    false
                }
            }
            MouseMessage::Wheel(e) => {
                let delta = e.delta_y();
                if delta < 0. {
                    self.area.zoom_by(ZOOM_FACTOR_IN);
                    true
                } else if delta > 0. {
                    self.area.zoom_by(ZOOM_FACTOR_OUT);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn view(&self, idx: usize) -> Node<Message> {
        let width = self.screen.x.get_distance();
        let height = self.screen.y.get_distance();
        canvas![
            el_ref(&self.canvas),
            attrs![
                At::Width => px(width),
                At::Height => px(height),
            ],
            style![
                St::Border => "1px solid black",
            ],
            touch_ev(Ev::TouchStart, move |e| {
                Some(Message::TouchMessage(idx, TouchMessage::TouchStart(e)))
            }),
            touch_ev(Ev::TouchEnd, move |e| {
                Some(Message::TouchMessage(idx, TouchMessage::TouchEnd(e)))
            }),
            touch_ev(Ev::TouchCancel, move |e| {
                Some(Message::TouchMessage(idx, TouchMessage::TouchCancel(e)))
            }),
            touch_ev(Ev::TouchMove, move |e| {
                e.prevent_default();
                Some(Message::TouchMessage(idx, TouchMessage::TouchMove(e)))
            }),
            mouse_ev(Ev::MouseDown, move |e| {
                Some(Message::MouseMessage(idx, MouseMessage::MouseDown(e)))
            }),
            mouse_ev(Ev::MouseUp, move |e| {
                Some(Message::MouseMessage(idx, MouseMessage::MouseUp(e)))
            }),
            mouse_ev(Ev::MouseMove, move |e| {
                Some(Message::MouseMessage(idx, MouseMessage::MouseMove(e)))
            }),
            wheel_ev(Ev::Wheel, move |e| {
                Some(Message::MouseMessage(idx, MouseMessage::Wheel(e)))
            })
        ]
    }

    pub fn draw(&self) {
        let canvas = self.canvas.get().expect("get canvas element");
        let ctx = seed::canvas_context_2d(&canvas);

        let plot = self.graph.plot(&self.area, &self.screen).unwrap();

        ctx.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
        let gray: JsValue = "#aaaaaa".into();
        ctx.begin_path();
        ctx.set_line_width(0.5);
        ctx.set_stroke_style(&gray);
        ctx.set_font("8px Arial");
        ctx.set_fill_style(&gray);

        if let Some(x_axis) = plot.x_axis {
            ctx.set_text_align("center");
            ctx.set_text_baseline("bottom");
            let y = plot.screen.y.max - x_axis.pos;
            ctx.move_to(self.screen.x.min, y);
            ctx.line_to(self.screen.x.max, y);

            for tic in x_axis.tics {
                if tic.label != 0. {
                    ctx.move_to(tic.pos, y);
                    ctx.line_to(tic.pos, y - 5.);
                    ctx.fill_text(&format!("{}", tic.label), tic.pos, y - 15.)
                        .expect("drawing x axis label failed");
                }
            }
        }

        if let Some(y_axis) = plot.y_axis {
            ctx.set_text_align("left");
            ctx.set_text_baseline("middle");
            let x = y_axis.pos;
            ctx.move_to(x, self.screen.y.min);
            ctx.line_to(x, self.screen.y.max);

            for tic in y_axis.tics {
                if tic.label != 0. {
                    let pos = plot.screen.y.max - tic.pos;
                    ctx.move_to(x, pos);
                    ctx.line_to(x + 5., pos);
                    ctx.fill_text(&format!("{}", tic.label), x + 7., pos)
                        .expect("drawing y axis label failed");
                }
            }
        }
        ctx.stroke();

        ctx.begin_path();
        ctx.set_line_width(1.5);
        ctx.set_stroke_style(&"blue".into());

        let points = plot.points;
        let mut close_stroke = false;

        for (x, y) in points.iter().enumerate() {
            match y {
                Some(y) => {
                    let y = plot.screen.y.max - y;
                    if close_stroke {
                        ctx.line_to(x as f64, y);
                    } else {
                        ctx.move_to(x as f64, y);
                    }
                    close_stroke = true;
                }
                None => {
                    if close_stroke {
                        ctx.stroke();
                        close_stroke = false;
                    }
                }
            }
        }
        if close_stroke {
            ctx.stroke();
        }
    }
}
