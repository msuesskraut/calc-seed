mod plot;
mod seed_helpers;
mod touch;

use rust_expression::{Calculator, Error, Number, Value};
use seed::prelude::*;
use seed::*;

use plot::{PlotElement, PlotMessage};

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
            Ok(Value::Graph(graph)) => CalcResult::Plot(Box::new(PlotElement::new(graph))),
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
    PlotMessage(usize, PlotMessage),
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
                CalcResult::Plot(plot) => plot.view(idx),
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
                    plot.draw();
                }
            }
        }
        Message::PlotMessage(idx, m) => {
            if let Some(cmd) = model.cmds.get_mut(idx) {
                if let CalcResult::Plot(ref mut plot_element) = cmd.res {
                    let must_render = plot_element.process(m);
                    if must_render {
                        orders.after_next_render(move |_| Message::RenderPlot(idx));
                    }
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
