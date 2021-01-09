use rust_expression::{Calculator, Error, Value, Number, Plot};
use seed::prelude::*;
use seed::*;

use web_sys::HtmlCanvasElement;

#[derive(Debug)]
struct PlotElement {
    plot: Plot,
    canvas: ElRef<HtmlCanvasElement>,
}

#[derive(Debug)]
enum CalcResult {
    Void,
    Number(Number),
    Solved { variable: String, value: Number },
    Plot(PlotElement),
    Error(Error)    
}

impl From<Result<Value, Error>> for CalcResult {
    fn from(res: Result<Value, Error>) -> CalcResult {
        match res {
            Ok(Value::Void) => CalcResult::Void,
            Ok(Value::Number(num)) => CalcResult::Number(num),
            Ok(Value::Solved { variable, value}) => CalcResult::Solved { variable, value },
            Ok(Value::Plot(plot)) => CalcResult::Plot(PlotElement { plot, canvas: ElRef::default() }),
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
    RenderPlot,
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
        .map(|cmd| {
            let res = match &cmd.res {
                CalcResult::Void => seed::empty(),
                CalcResult::Number(num) => div![C!("success"), "=> ", num.to_string()],
                CalcResult::Solved { variable, value} => div![C!("success"), "=> ", variable, " = ", value.to_string()],
                CalcResult::Plot(plot) => canvas![
                    el_ref(&plot.canvas),
                    attrs![
                        At::Width => px(400),
                        At::Height => px(300),
                    ],
                    style![
                        St::Border => "1px solid black",
                    ],
                ],
                CalcResult::Error(err) => div![C!("failure"), format!("{:?}", err)],
            };
            vec![
                div![C!("row"), div![C!("col-12"), span![C!("prompt"), "> "], cmd.cmd.to_string()],],
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
        div![C!("row"),
            div![C!("col-12"), h1!("Calculator")],
        ],
        commands,
        view_footer()
    ]
}

fn draw(plot: &PlotElement) {
    let canvas = plot.canvas.get().expect("get canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    ctx.rect(0., 0., 200., 100.);
    ctx.set_fill_style(&JsValue::from_str("blue"));
    ctx.fill();

    ctx.move_to(0., 0.);
    ctx.line_to(200., 100.);
    ctx.stroke();
}

fn update(message: Message, model: &mut Model, orders: &mut impl Orders<Message>) {
    match message {
        Message::CommandUpdate(cmd) => model.current_command = cmd,
        Message::ClearCommand => model.current_command.clear(),
        Message::ExecuteCommand => {
            let res = model.calc.execute(&model.current_command);
            model.cmds.push(CalcCommand {
                cmd: model.current_command.clone(),
                res: res.into(),
            });
            model.history = model.cmds.len();
            model.current_command.clear();
        },
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
        },
        Message::HistoryUp => {
            let mut history_entry = model.history;
            if history_entry > 0 {
                history_entry -= 1;
            }
            model.history = history_entry;
            if model.history < model.cmds.len() {
                model.current_command = model.cmds[model.history].cmd.clone();
            }
        },
        Message::RenderPlot => {
            for cmd in &model.cmds {
                match cmd.res {
                    CalcResult::Plot(ref plot) => draw(plot),
                    _ => {}
                }
            }
            orders.after_next_render(|_| Message::RenderPlot).skip();
        }
    }
}

fn init(_url: Url, orders: &mut impl Orders<Message>) -> Model {
    orders.after_next_render(|_| Message::RenderPlot);
    Model::default()
}

#[wasm_bindgen]
pub fn render() {
    seed::App::start("app", init, update, view);
}
