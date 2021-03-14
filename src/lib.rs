use rust_expression::{Area, Calculator, Error, Graph, Number, Value};
use seed::prelude::*;
use seed::*;

use web_sys::HtmlCanvasElement;

#[derive(Debug)]
struct PlotElement {
    graph: Graph,
    canvas: ElRef<HtmlCanvasElement>,
    screen: Area,
    area: Area,
}

#[derive(Debug)]
enum CalcResult {
    Void,
    Number(Number),
    Solved { variable: String, value: Number },
    Plot(Box<PlotElement>),
    Error(Error),
}

impl From<Result<Value, Error>> for CalcResult {
    fn from(res: Result<Value, Error>) -> CalcResult {
        match res {
            Ok(Value::Void) => CalcResult::Void,
            Ok(Value::Number(num)) => CalcResult::Number(num),
            Ok(Value::Solved { variable, value }) => CalcResult::Solved { variable, value },
            Ok(Value::Graph(graph)) => CalcResult::Plot(Box::new(PlotElement {
                graph,
                canvas: ElRef::default(),
                screen: Area::new(0., 0., 400., 300.),
                area: Area::new(-100., -100., 100., 100.),
            })),
            Err(err) => CalcResult::Error(err),
        }
    }
}

#[derive(Debug)]
struct CalcCommand {
    cmd: String,
    res: CalcResult,
}

#[derive(Debug, Default)]
struct Model {
    cmds: Vec<CalcCommand>,
    calc: Calculator,
    current_command: String,
    history: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    CommandUpdate(String),
    ExecuteCommand,
    ClearCommand,
    HistoryUp,
    HistoryDown,
    RenderPlot(usize),
    DragPlot(usize, f64, f64),
}

const ENTER_KEY: &str = "Enter";
const ESC_KEY: &str = "Escape";
const UP_KEY: &str = "ArrowUp";
const DOWN_KEY: &str = "ArrowDown";

fn view_footer() -> Node<Message> {
    const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
    const REPO: Option<&'static str> = option_env!("CARGO_PKG_REPOSITORY");

    div![
        C!["row"],
        div![
            C!["col footer"],
            "Calculator",
            if let Some(url) = REPO {
                span![
                    " | ",
                    a![
                        attrs! {
                            At::Href => url;
                            At::Rel => "norefferer noopener external";
                            At::Target => "_blank"
                        },
                        "Github"
                    ],
                ]
            } else {
                seed::empty()
            },
            format!(
                " | {}: {}",
                "version",
                VERSION.unwrap_or("<version unknown>")
            )
        ]
    ]
}

fn view(model: &Model) -> Node<Message> {
    let mut commands: Vec<Node<Message>> = model
        .cmds
        .iter()
        .enumerate()
        .map(|(idx, cmd)| {
            let res = match &cmd.res {
                CalcResult::Void => seed::empty(),
                CalcResult::Number(num) => div![C!("success"), "=> ", num.to_string()],
                CalcResult::Solved { variable, value } => {
                    div![C!("success"), "=> ", variable, " = ", value.to_string()]
                }
                CalcResult::Plot(plot) => {
                    let width = plot.screen.x.get_distance();
                    let height = plot.screen.y.get_distance();
                    canvas![
                        el_ref(&plot.canvas),
                        attrs![
                            At::Width => px(width),
                            At::Height => px(height),
                        ],
                        style![
                            St::Border => "1px solid black",
                        ],
                        mouse_ev(Ev::MouseMove, move |e| {
                            if e.buttons() == 1 {
                                Some(Message::DragPlot(
                                    idx,
                                    e.movement_x().into(),
                                    e.movement_y().into(),
                                ))
                            } else {
                                None
                            }
                        }),
                    ]
                }
                CalcResult::Error(err) => div![C!("failure"), format!("{:?}", err)],
            };
            vec![
                div![
                    C!("row"),
                    div![C!("col-12"), span![C!("prompt"), "> "], cmd.cmd.to_string()],
                ],
                div![C!("row"), div![C!("col-12"), res]],
            ]
        })
        .flatten()
        .collect();
    commands.push(div![
        C!("row"),
        div![
            C!("col-12 input-group"),
            input![
                C!("form-control no-outline"),
                attrs![
                    At::Type => "text",
                    At::Name => "command",
                    At::Placeholder => "command",
                    At::AutoFocus => true.as_at_value(),
                    At::Value => model.current_command,
                    "aria-label" => "Command",
                    "aria-describedby" => "basic-addon2",
                    "autocapitalize" => "off",  
                ],
                input_ev(Ev::Input, Message::CommandUpdate),
                keyboard_ev(Ev::KeyDown, |keyboard_event| {
                    Some(match keyboard_event.key().as_str() {
                        ENTER_KEY => Message::ExecuteCommand,
                        ESC_KEY => Message::ClearCommand,
                        UP_KEY => Message::HistoryUp,
                        DOWN_KEY => Message::HistoryDown,
                        _ => return None,
                    })
                }),
            ],
            div![
                C!("input-group-append"),
                button![
                    C!("btn btn-outline-secondary"),
                    attrs!(At::Type => "button"),
                    "Execute",
                    ev(Ev::Click, |_| Message::ExecuteCommand)
                ]
            ]
        ]
    ]);

    div![
        C!("container"),
        div![C!("row"), div![C!("col-12"), h1!("Calculator")],],
        commands,
        view_footer()
    ]
}

fn draw(plot_element: &PlotElement) {
    let canvas = plot_element.canvas.get().expect("get canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let plot = plot_element
        .graph
        .plot(&plot_element.area, &plot_element.screen)
        .unwrap();

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
        ctx.move_to(plot_element.screen.x.min, y);
        ctx.line_to(plot_element.screen.x.max, y);

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
        ctx.move_to(x, plot_element.screen.y.min);
        ctx.line_to(x, plot_element.screen.y.max);

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

fn update(message: Message, model: &mut Model, orders: &mut impl Orders<Message>) {
    match message {
        Message::CommandUpdate(cmd) => model.current_command = cmd,
        Message::ClearCommand => model.current_command.clear(),
        Message::ExecuteCommand => {
            if !model.current_command.is_empty() {
                let res = model.calc.execute(&model.current_command);
                if matches!(&res, Ok(Value::Graph(_))) {
                    let next_idx = model.cmds.len();
                    orders.after_next_render(move |_| Message::RenderPlot(next_idx));
                }
                model.cmds.push(CalcCommand {
                    cmd: model.current_command.clone(),
                    res: res.into(),
                });
                model.history = model.cmds.len();
                model.current_command.clear();
            }
        }
        Message::HistoryDown => {
            let mut history_entry = model.history;
            if history_entry < model.cmds.len() {
                history_entry += 1;
            }
            model.history = history_entry;
            if model.history < model.cmds.len() {
                model.current_command = model.cmds[model.history].cmd.clone();
            } else {
                model.current_command.clear();
            }
        }
        Message::HistoryUp => {
            let mut history_entry = model.history;
            if history_entry > 0 {
                history_entry -= 1;
            }
            model.history = history_entry;
            if model.history < model.cmds.len() {
                model.current_command = model.cmds[model.history].cmd.clone();
            }
        }
        Message::RenderPlot(idx) => {
            seed::log!(format!("renderPlot({})", idx));
            if let Some(cmd) = model.cmds.get(idx) {
                if let CalcResult::Plot(ref plot) = cmd.res {
                    draw(plot);
                }
            }
        }
        Message::DragPlot(index, x, y) => {
            seed::log!(format!("Move plot {} by ({}, {})", index, x, y));
            if let Some(cmd) = model.cmds.get_mut(index) {
                if let CalcResult::Plot(ref mut plot_element) = cmd.res {
                    plot_element.area.move_by(-x, y);
                    orders.after_next_render(move |_| Message::RenderPlot(index));
                }
            }
        }
    }
}

fn init(_url: Url, _orders: &mut impl Orders<Message>) -> Model {
    Model::default()
}

#[wasm_bindgen]
pub fn render() {
    seed::App::start("app", init, update, view);
}
