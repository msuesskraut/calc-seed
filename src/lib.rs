use rust_expression::{ Calculator, Error };
use seed::*;
use seed::prelude::*;

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
}

#[derive(Debug, Clone)]
pub enum Message {
    CommandUpdate(String),
    ExecuteCommand,
    ClearCommand,
}

const ENTER_KEY: &str = "Enter";
const ESC_KEY: &str = "Escape";

fn view(model: &Model) -> Node<Message> {
    let mut commands: Vec<Node<Message>> = model.cmds.iter().map( |cmd| {
            let res = match &cmd.res {
                Ok(Some(num)) => div![C!("col-12 success"), num.to_string()],
                Ok(None) => seed::empty(),
                Err(err) => div![C!("col-12 failure"), format!("Error: {:?}", err)],
            };
            vec![
                div![C!("row"),
                    div![C!("col-12"), cmd.cmd.clone()],
                ],
                div![C!("row"), 
                    res
                ]
            ]
        }).flatten().collect();
    commands.push(
        div![C!("row"),
            div![C!("col-12 input-group"),
                input![
                    C!("form-control no-outline"),
                    attrs![
                        At::Type => "text",
                        At::Name => "command",
                        At::Placeholder => "command",
                        At::AutoFocus => true.as_at_value(),
                        "aria-label" => "Command",
                        "aria-describedby" => "basic-addon2",
                    ],
                    input_ev(Ev::Input, Message::CommandUpdate),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        Some(match keyboard_event.key().as_str() {
                                ENTER_KEY => Message::ExecuteCommand,
                                ESC_KEY => Message::ClearCommand,
                                _ => return None
                        })
                    }),
                ],
                div![C!("input-group-append"),
                    button![C!("btn btn-outline-secondary"),
                        attrs!(At::Type => "button"),
                        "Execute",
                        ev(Ev::Click, |_| Message::ExecuteCommand)
                    ]
                ]
            ]
        ]
    );
    div![C!("container"),
        commands
    ]
}

fn update(message: Message, model: &mut Model, _: &mut impl Orders<Message>) {
    seed::log!(format!("Got {:?}", message));

    match message {
        Message::CommandUpdate(cmd) => model.current_command = cmd,
        Message::ClearCommand => model.current_command.clear(),
        Message::ExecuteCommand => {
            let res = model.calc.execute(&model.current_command);
            model.cmds.push(Command { cmd: model.current_command.clone(), res });
            model.current_command.clear();
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
