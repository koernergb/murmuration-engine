use bytemuck::{Pod, Zeroable};
use sim_core::Bird;

#[derive(Debug)]
pub struct BirdBuffers {
    pub buffer: wgpu::Buffer,
    pub bird_capacity: usize,
}

impl BirdBuffers {
    pub fn new(buffer: wgpu::Buffer, bird_capacity: usize) -> Self {
        Self {
            buffer,
            bird_capacity,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct BirdInstanceRaw {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub aux: [f32; 4],
}

impl BirdInstanceRaw {
    pub fn from_bird(bird: &Bird) -> Self {
        Self {
            position: [bird.position[0], bird.position[1], bird.position[2], 1.0],
            velocity: [bird.velocity[0], bird.velocity[1], bird.velocity[2], 0.0],
            aux: [bird.fear, bird.phase, bird.density, bird.seed],
        }
    }
}
