pub trait System {
    fn update(&self);
}

pub struct Engine<'a> {
    systems: Vec<Box<dyn System + 'a>>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        Engine { systems: Vec::new() }
    }

    pub fn add_system<T: System + 'a>(&mut self, sys: T) {
        self.systems.push(Box::new(sys));
    }

    pub fn update(&self) {
        for sys in self.systems.iter() {
            sys.update();
        }
    }
}
