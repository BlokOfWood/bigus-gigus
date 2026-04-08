use crate::app::App;

pub(crate) trait Plugin {
    fn new(app: &mut App) -> Self where Self:Sized;
    fn tick(&self);
}
