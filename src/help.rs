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
    /*
    div![
        C!["alert alert-info"],
        attrs! {
            "role" => "alert",
        },
        h4![
            C!["alert-heading"],
            "Well done!",
        ],
        p![
            "Aww yeah, you successfully read this important alert message.
            This example text is going to run a bit longer so that you can see how spacing within an alert works with this kind of content.",
        ],
        hr![],
        p![
            C!["mb-0"],
            "Whenever you need to, be sure to use margin utilities to keep things nice and tidy.",
        ],
    ]
    */
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

/*

<!-- Modal -->
  <div class="modal-dialog model-xl modal-dialog-centered" role="document">
    <div class="modal-content">
      <div class="modal-header">
        <h5 class="modal-title" id="helpTitle">Modal title</h5>
        <button type="button" class="close" data-dismiss="modal" aria-label="Close">
          <span aria-hidden="true">&times;</span>
        </button>
      </div>
      <div class="modal-body">
        ...
      </div>
      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" data-dismiss="modal">Close</button>
        <button type="button" class="btn btn-primary">Save changes</button>
      </div>
    </div>
  </div>
  */