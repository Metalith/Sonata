pub trait System {
    fn update(&self);
}

pub struct Engine {
    systems: Vec<Box<dyn System>>
}


impl Engine {
    pub fn new() -> Self {
        Engine {
            systems: Vec::new()
        }
    }

    pub fn add_system<T: System + 'static>(&mut self, sys: T) {
        self.systems.push(Box::new(sys));
    }

    pub fn update(&self) {
        for sys in self.systems.iter() {
            sys.update();
        }
    }
}