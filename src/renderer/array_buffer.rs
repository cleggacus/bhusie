use std::fmt::Debug;

use wgpu::util::DeviceExt;

pub struct ArrayBuffer<const MAX_ENTITIES: usize, T, U>
where
    T: Copy + Debug,
    U: ArrayBufferUniform<T> + Default + Copy + bytemuck::Pod + bytemuck::Zeroable,
{
    entities: [Option<T>; MAX_ENTITIES],
    entity_uniforms: [U; MAX_ENTITIES],
    size: usize,
}

impl<const MAX_ENTITIES: usize, T, U> ArrayBuffer<MAX_ENTITIES, T, U>
where
    T: Copy + Debug,
    U: ArrayBufferUniform<T> + Default + Copy + bytemuck::Pod + bytemuck::Zeroable,
    [U; MAX_ENTITIES]: bytemuck::Pod
{
    pub fn new() -> Self {
        let size = 0;
        let entities = [None; MAX_ENTITIES];
        let entity_uniforms = [U::default(); MAX_ENTITIES];

        Self {
            entities,
            entity_uniforms,
            size,
        }
    }

    pub fn insert(&mut self, entity: T) -> Option<&T> {
        if self.size < MAX_ENTITIES {
            self.entities[self.size] = Some(entity);
            let val = self.entities[self.size].as_ref();
            self.size += 1;
            return val;
        }

        None
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.size {
            self.entities[index] = self.entities[self.size - 1].take();
            self.size -= 1;
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index <= self.size {
            return None;
        }

        self.entities[index].as_ref()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.size {
            return None;
        }

        self.entities[index].as_mut()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn update_buffer(&mut self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        for i in 0..self.size {
            if let Some(entity) = &self.entities[i] {
                self.entity_uniforms[i].update(entity);
            }
        }

        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[self.entity_uniforms]));
    }

    pub fn create_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[self.entity_uniforms]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        )
    }
}

impl<const MAX_ENTITIES: usize, T, U> Default for ArrayBuffer<MAX_ENTITIES, T, U>
where
    T: Copy + Debug,
    U: ArrayBufferUniform<T> + Default + Copy + bytemuck::Pod + bytemuck::Zeroable,
    [U; MAX_ENTITIES]: bytemuck::Pod
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait ArrayBufferUniform<T> {
    fn update(&mut self, entity: &T);
}
