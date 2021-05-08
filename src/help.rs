use crate::Message;

use seed::prelude::*;
use seed::*;

pub fn view_help_button() -> Node<Message> {
    button![
        C!["btn btn-primary"],
        attrs! {
            At::Type => "button",
            "data-toggle" => "modal",
            "data-target" => "#help"
        },
        "?",
    ]
}

pub fn view_help() -> Node<Message> {
    /* Model dialog */
    div![
        C!["modal fade help"],
        attrs! {
            At::Id => "help",
            At::TabIndex => "-1",
            "role" => "dialog",
            "aria-labelledby" => "helpTitle",
            "aria-hidden" => "true",
        },
        div![
            C!["modal-dialog modal-xl modal-dialog-centered modal-dialog-scrollable"],
            attrs! {
                "role" => "document",
            },
            div![
                C!["modal-content"],
                div![
                    C!["modal-header"],
                    h5![
                        C!["modal-title"],
                        attrs![
                            At::Id => "helpTitle",
                        ],
                        "Calculator help",
                    ],
                    button![
                        C!["close"],
                        attrs! {
                            At::Type => "button",
                            "data-dismiss" => "modal",
                            "aria-label" => "Close",
                        },
                        span![
                            attrs! {
                                "aria-hidden" => "true",
                            },
                            "X",
                        ]
                    ]
                ],
                div![
                    C!["modal-body"],
                    El::from_markdown(rust_expression::HELP_SUMMARY),
                ],
                div![
                    C!["modal-footer"],
                    button![
                        C!["btn btn-secondary"],
                        attrs! {
                            At::Type => "button",
                            "data-dismiss" => "modal",
                        },
                        "Close",
                    ],
                ],
            ]
        ]
    ]
}
