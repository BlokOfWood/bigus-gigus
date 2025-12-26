#[allow(unused)]
pub struct System {
    id: u32,
    tick_function: TickFunction,
}

pub type TickFunction = fn() -> ();

impl System {
    pub fn tick(&self) {
        (self.tick_function)();
    }
}