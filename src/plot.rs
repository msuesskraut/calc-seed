use rust_expression::{Area, Graph, Number};
use seed::prelude::*;
use seed::*;

use web_sys::HtmlCanvasElement;

use crate::{Message, MouseMessage, TouchMessage};

#[derive(Debug)]
pub struct PlotElement {
    graph: Graph,
    canvas: ElRef<HtmlCanvasElement>,
    screen: Area,
    area: Area,
}

impl PlotElement {
    pub fn new(graph: Graph) -> PlotElement {
        PlotElement {
            graph,
            canvas: ElRef::default(),
            screen: Area::new(0., 0., 400., 300.),
            area: Area::new(-100., -100., 100., 100.),
        }
    }

    pub fn process_touch(&mut self, _e: TouchMessage) -> bool {
        false
    }

    pub fn process_mouse(&mut self, m: MouseMessage) -> bool {
        match m {
            MouseMessage::MouseMove(e) => {
                if e.buttons() == 1 {
                    let x_delta: Number = e.movement_x().into();
                    let x_delta = -x_delta;
                    let y_delta: Number = e.movement_y().into();
                    seed::log!(format!("Move plot by ({}, {})", x_delta, y_delta));
                    self.area.move_by(x_delta, y_delta);
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
                ctx.move_to(tic.pos, y);
                ctx.line_to(tic.pos, y - 5.);
                ctx.fill_text(&format!("{}", tic.label), tic.pos, y - 15.)
                    .expect("drawing x axis label failed");
            }
        }

        if let Some(y_axis) = plot.y_axis {
            ctx.set_text_align("left");
            ctx.set_text_baseline("middle");
            let x = y_axis.pos;
            ctx.move_to(x, self.screen.y.min);
            ctx.line_to(x, self.screen.y.max);

            for tic in y_axis.tics {
                let pos = plot.screen.y.max - tic.pos;
                ctx.move_to(x, pos);
                ctx.line_to(x + 5., pos);
                ctx.fill_text(&format!("{}", tic.label), x + 7., pos)
                    .expect("drawing y axis label failed");
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
