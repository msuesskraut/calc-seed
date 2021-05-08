use rust_expression::Number;
use web_sys::{Touch, TouchList};

#[derive(Debug)]
pub struct TouchPoint {
    id: i32,
    x: i32,
    y: i32,
}

impl TouchPoint {
    fn new(touch: &Touch) -> Self {
        let id = touch.identifier();
        let x = touch.client_x();
        let y = touch.client_y();
        Self { id, x, y }
    }
}

#[derive(Debug)]
pub enum TouchState {
    Empty,
    Move(TouchPoint),
    Zoom(TouchPoint, TouchPoint),
}

impl TouchState {
    pub fn new(tl: &TouchList) -> Self {
        use TouchState::*;

        match tl.length() {
            1 => Move(TouchPoint::new(&tl.item(0).unwrap())),
            2 => {
                let p1 = TouchPoint::new(&tl.item(0).unwrap());
                let p2 = TouchPoint::new(&tl.item(1).unwrap());
                if p1.id < p2.id {
                    Zoom(p1, p2)
                } else {
                    Zoom(p2, p1)
                }
            }
            _ => Empty,
        }
    }

    fn get_distance(&self) -> Option<Number> {
        match self {
            Self::Zoom(tp1, tp2) => {
                let x: Number = (tp1.x - tp2.x).into();
                let y: Number = (tp1.y - tp2.y).into();
                Some(x.hypot(y))
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum TouchEffect {
    Empty,
    Move(Number, Number),
    Zoom(Number),
}

impl TouchEffect {
    pub fn new(previous: &TouchState, current: &TouchState) -> Self {
        use TouchEffect::*;

        match previous {
            TouchState::Move(tp_previous) => match current {
                TouchState::Move(tp_current) if tp_previous.id == tp_current.id => {
                    let x = tp_previous.x - tp_current.x;
                    let y = tp_previous.y - tp_current.y;
                    Move(x.into(), y.into())
                }
                _ => Empty,
            },
            TouchState::Zoom(tp1_previous, tp2_previous) => match current {
                TouchState::Zoom(tp1_current, tp2_current)
                    if (tp1_previous.id == tp1_current.id)
                        && (tp2_previous.id == tp2_current.id) =>
                {
                    let prev_dist = previous.get_distance().unwrap();
                    let curr_dist = current.get_distance().unwrap();
                    Zoom(prev_dist / curr_dist)
                }
                _ => Empty,
            },
            TouchState::Empty => Empty,
        }
    }
}
