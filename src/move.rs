pub struct MoveSystem;

impl wind::System for MoveSystem {
    fn update(&self) {
        println!("Hello World from a system!");
    }
}