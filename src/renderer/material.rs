use super::array_buffer::{ArrayBuffer, ArrayBufferUniform};

pub const MAX_MATERIALS: usize = 8;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub color: [f32; 4],
}

impl Material {
    pub fn new(color: [f32; 4]) -> Self {
        Self {
            color,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub color: [f32; 4],
}

impl Default for MaterialUniform {
    fn default() -> Self {
        Self {
            color: [1.0; 4],
        }
    }
}

impl ArrayBufferUniform<Material> for MaterialUniform {
    fn update(&mut self, material: &Material) {
        self.color = material.color;
    }
}

pub type MaterialArrayBuffer = ArrayBuffer<MAX_MATERIALS, Material, MaterialUniform>;
