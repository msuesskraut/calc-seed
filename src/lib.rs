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
    ExecuteCommand
}

const ENTER_KEY: u32 = 13;

fn view(model: &Model) -> Vec<Node<Message>> {
    let mut commands: Vec<Node<Message>> = model.cmds.iter().map( |cmd| {
            let res = match &cmd.res {
                Ok(Some(num)) => div![C!("col success"), num.to_string()],
                Ok(None) => seed::empty(),
                Err(err) => div![C!("col failure"), format!("Error: {:?}", err)],
            };
            vec![
                div![C!("row"),
                    div![C!("col"), cmd.cmd.clone()],
                ],
                div![C!("row"), 
                    res
                ]
            ]
        }).flatten().collect();
    commands.push(
        div![C!("row"),
            div![C!("col"),
                input![
                    attrs![
                        At::Type => "text",
                        At::Name => "command",
                        At::Placeholder => "command",
                        At::AutoFocus => true.as_at_value(),
                    ],
                    input_ev(Ev::Input, Message::CommandUpdate),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        IF!(keyboard_event.key_code() == ENTER_KEY => Message::ExecuteCommand)
                    }),
            ]]
        ]
    );
    commands
}

fn update(message: Message, model: &mut Model, _: &mut impl Orders<Message>) {
    seed::log!(format!("Got {:?}", message));

    match message {
        Message::CommandUpdate(cmd) => model.current_command = cmd,
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
