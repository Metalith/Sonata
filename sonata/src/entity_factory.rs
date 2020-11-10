use sketch::models::{MeshFactory, Vertex};
use specs::{Builder, World, WorldExt};

use crate::{Movement, Player, Renderable, Transform};

//TODO: Use a file loader instead of hardcoded vertices
//TODO: Need a way to modify vertices for a skeleton system
pub struct EntityFactory {
    mesh_factory: MeshFactory,
}

impl EntityFactory {
    pub fn new(mesh_factory: MeshFactory) -> EntityFactory {
        EntityFactory { mesh_factory }
    }

    pub fn create_player(&self, world: &mut World, pos: [f32; 3]) {
        world
            .create_entity()
            .with(Player::default())
            .with(Movement::default())
            .with(Transform {
                pos: pos.into(),
                dir: uv::Rotor3::from_euler_angles(0.0f32.to_radians(), 0.0, 180.0f32.to_radians()), // Look at center from above due to colinearity
            })
            .build();
    }

    pub fn create_grid(&self, world: &mut World) {
        let vertices = [
            Vertex {
                pos: [-0.5f32, -0.5f32],
                color: [1.0f32, 0.0f32, 0.0f32],
            },
            Vertex {
                pos: [0.5f32, -0.5f32],
                color: [0.0f32, 1.0f32, 0.0f32],
            },
            Vertex {
                pos: [0.5f32, 0.5f32],
                color: [0.0f32, 0.0f32, 1.0f32],
            },
            Vertex {
                pos: [-0.5f32, 0.5f32],
                color: [1.0f32, 1.0f32, 1.0f32],
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        world
            .create_entity()
            .with(Transform::default())
            .with(Renderable {
                mesh: self.mesh_factory.create_mesh(&vertices, Some(&indices)),
            })
            .build();
    }
}
