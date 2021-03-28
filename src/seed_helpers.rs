use std::rc::Rc;

use seed::prelude::*;
use seed::*;

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
