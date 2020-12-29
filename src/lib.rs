use rust_expression::{Calculator, Error};
use seed::prelude::*;
use seed::*;

type Number = f64;

#[derive(Debug)]
struct Command {
    cmd: String,
    res: Result<Option<Number>, Error>,
}

#[derive(Debug, Default)]
struct Model {
    cmds: Vec<Command>,
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
                Ok(Some(num)) => div![C!("col-12 success"), "=> ", num.to_string()],
                Ok(None) => seed::empty(),
                Err(err) => div![C!("col-12 failure"), pre![format!("Error: {:}", err)]],
            };
            vec![
                div![C!("row"), div![C!("col-12"), span![C!("prompt"), "> "], cmd.cmd.to_string()],],
                div![C!("row"), res],
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
        div![C!("col-12"), h1!("Calculator")],
        commands,
        view_footer()
    ]
}

fn update(message: Message, model: &mut Model, _: &mut impl Orders<Message>) {
    seed::log!(format!("Got {:?}", message));

    match message {
        Message::CommandUpdate(cmd) => model.current_command = cmd,
        Message::ClearCommand => model.current_command.clear(),
        Message::ExecuteCommand => {
            let res = model.calc.execute(&model.current_command);
            model.cmds.push(Command {
                cmd: model.current_command.clone(),
                res,
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
    }
}

fn init(_url: Url, _orders: &mut impl Orders<Message>) -> Model {
    Model::default()
}

#[wasm_bindgen]
pub fn render() {
    seed::App::start("app", init, update, view);
}
