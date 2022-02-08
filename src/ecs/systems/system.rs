use hecs::World;

pub trait System {
    fn name(&self) -> &str;
    fn start(&self, world: &mut World) -> Result<(), String>;
    fn update(&self, world: &mut World) -> Result<(), String>;
    fn late_update(&self, world: &mut World) -> Result<(), String>;
}