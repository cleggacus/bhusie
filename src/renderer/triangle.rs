use cgmath::{Vector3, Zero};

use super::array_buffer::{ArrayBuffer, ArrayBufferUniform};

pub const MAX_IDENT_LENGTH: usize = 64;
pub const MAX_MODELS: usize = 1;
pub const MAX_MODEL_VERTICES: usize = 512000;

#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub min_corner: Vector3<f32>,
    pub left_child: i32,
    pub max_corner: Vector3<f32>,
    pub obj_count: i32,
}

impl Node {
    pub fn new() -> Self {
        Node {
            min_corner: Vector3::zero(),
            left_child: 0,
            max_corner: Vector3::zero(),
            obj_count: 0,
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Node> for NodeUniform {
    fn from(val: Node) -> Self {
        NodeUniform {
            min_corner: val.min_corner.into(),
            left_child: val.left_child,
            max_corner: val.max_corner.into(),
            obj_count: val.obj_count,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NodeUniform {
    pub min_corner: [f32; 3],
    pub left_child: i32,
    pub max_corner: [f32; 3],
    pub obj_count: i32,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    pub p1: i32,
    pub p2: i32,
    pub p3: i32,
    pub n1: i32,
    pub n2: i32,
    pub n3: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Model {
    pub ident_size: usize,
    pub ident: [char; MAX_IDENT_LENGTH],
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub point_count: i32,
    pub normal_count: i32,
    pub triangle_count: i32,
    pub visible: i32,
    pub points: [[f32; 4]; MAX_MODEL_VERTICES],
    pub normals: [[f32; 4]; MAX_MODEL_VERTICES],
    pub triangles: [Triangle; MAX_MODEL_VERTICES],
    pub nodes: [Node; MAX_MODEL_VERTICES],
    pub bvh_lookup: [i32; MAX_MODEL_VERTICES],
    nodes_used: usize,
}

impl Model {
    pub fn new(ident_str: &str) -> Self {
        let mut ident_size = 0;
        let mut ident = [' '; MAX_IDENT_LENGTH];

        for c in ident_str.chars() {
            ident[ident_size] = c;
            ident_size += 1;

            if ident_size >= MAX_IDENT_LENGTH {
                break;
            }
        }

        Self {
            ident_size,
            ident,
            position: Vector3::new(-7.0, 0.0, 35.0),
            rotation: Vector3::zero(),
            point_count: 0,
            triangle_count: 0,
            normal_count: 0,
            points: [[0.0; 4]; MAX_MODEL_VERTICES],
            normals: [[0.0; 4]; MAX_MODEL_VERTICES],
            triangles: [Triangle::default(); MAX_MODEL_VERTICES],
            visible: 1,
            nodes: [Node::default(); MAX_MODEL_VERTICES],
            bvh_lookup: [0; MAX_MODEL_VERTICES],
            nodes_used: 0,
        }
    }

    pub fn set_ident(&mut self, ident_str: &str) {
        self.ident_size = 0;

        for c in ident_str.chars() {
            self.ident[self.ident_size] = c;
            self.ident_size += 1;

            if self.ident_size >= MAX_IDENT_LENGTH {
                break;
            }
        }
    }

    pub fn add_normal(&mut self, normal: [f32; 4]) {
        self.normals[self.normal_count as usize] = normal;
        self.normal_count += 1;
    }

    pub fn add_vertex(&mut self, point: [f32; 4]) {
        self.points[self.point_count as usize] = point;
        self.point_count += 1;
    }

    pub fn add_triangle(&mut self, triangle: Triangle) {
        self.triangles[self.triangle_count as usize] = triangle;
        self.triangle_count += 1;
    }

    pub fn build_bvh(&mut self) {
        self.nodes_used = 0;

        for i in 0..self.triangle_count {
            self.bvh_lookup[i as usize] = i;
        }

        let root = &mut self.nodes[0];
        root.left_child = 0;
        root.obj_count = self.triangle_count;
        self.nodes_used += 1;

        self.update_bounds(0);
        self.subdivide(0);
    }

    fn update_bounds(&mut self, node_index: usize) {
        let node = &mut self.nodes[node_index];
        let min = f32::MAX;
        let max = f32::MIN;
        node.min_corner = Vector3::new(min, min, min);
        node.max_corner = Vector3::new(max, max, max);

        for i in 0..node.obj_count {
            let triangle = &mut self.triangles[self.bvh_lookup[(node.left_child + i) as usize] as usize];
            
            let p1 = self.points[triangle.p1 as usize];
            let p2 = self.points[triangle.p2 as usize];
            let p3 = self.points[triangle.p3 as usize];

            node.min_corner[0] = node.min_corner[0].min(p1[0]);
            node.max_corner[0] = node.max_corner[0].max(p1[0]);
            node.min_corner[1] = node.min_corner[1].min(p1[1]);
            node.max_corner[1] = node.max_corner[1].max(p1[1]);
            node.min_corner[2] = node.min_corner[2].min(p1[2]);
            node.max_corner[2] = node.max_corner[2].max(p1[2]);

            node.min_corner[0] = node.min_corner[0].min(p2[0]);
            node.max_corner[0] = node.max_corner[0].max(p2[0]);
            node.min_corner[1] = node.min_corner[1].min(p2[1]);
            node.max_corner[1] = node.max_corner[1].max(p2[1]);
            node.min_corner[2] = node.min_corner[2].min(p2[2]);
            node.max_corner[2] = node.max_corner[2].max(p2[2]);

            node.min_corner[0] = node.min_corner[0].min(p3[0]);
            node.max_corner[0] = node.max_corner[0].max(p3[0]);
            node.min_corner[1] = node.min_corner[1].min(p3[1]);
            node.max_corner[1] = node.max_corner[1].max(p3[1]);
            node.min_corner[2] = node.min_corner[2].min(p3[2]);
            node.max_corner[2] = node.max_corner[2].max(p3[2]);
        }
    }

    fn subdivide(&mut self, node_index: usize) {
        if self.nodes[node_index].obj_count <= 2 {
            return;
        }

        let extent = self.nodes[node_index].max_corner - self.nodes[node_index].min_corner;
        let mut axis = 0;

        if extent[1] > extent[axis] {
            axis = 1;
        }

        if extent[2] > extent[axis] {
            axis = 2;
        }

        let split_position = self.nodes[node_index].min_corner[axis] + extent[axis] / 2.0;

        let mut i = self.nodes[node_index].left_child;
        let mut j = i + self.nodes[node_index].obj_count - 1;

        while i <= j {
            let triangle_i = self.triangles[self.bvh_lookup[i as usize] as usize];
            let triangle_i_centroid = Vector3::new(
                (self.points[triangle_i.p1 as usize][0] + self.points[triangle_i.p2 as usize][0] + self.points[triangle_i.p3 as usize][0]) / 3.0,
                (self.points[triangle_i.p1 as usize][1] + self.points[triangle_i.p2 as usize][1] + self.points[triangle_i.p3 as usize][1]) / 3.0,
                (self.points[triangle_i.p1 as usize][2] + self.points[triangle_i.p2 as usize][2] + self.points[triangle_i.p3 as usize][2]) / 3.0
            );

            if triangle_i_centroid[axis] < split_position {
                i += 1;
            } else {
                self.bvh_lookup.swap(i as usize, j as usize);
                j -= 1;
            }
        }

        let left_count = i - self.nodes[node_index].left_child;

        if left_count == 0 || left_count == self.nodes[node_index].obj_count {
            return;
        }

        let left_child_index = self.nodes_used;
        self.nodes_used += 1;

        let right_child_index = self.nodes_used;
        self.nodes_used += 1;

        self.nodes[left_child_index].left_child = self.nodes[node_index].left_child;
        self.nodes[left_child_index].obj_count = left_count;

        self.nodes[right_child_index].left_child = i;
        self.nodes[right_child_index].obj_count = self.nodes[node_index].obj_count - left_count;

        self.nodes[node_index].left_child = left_child_index as i32;
        self.nodes[node_index].obj_count = 0;

        self.update_bounds(left_child_index);
        self.update_bounds(right_child_index);

        self.subdivide(left_child_index);
        self.subdivide(right_child_index);
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new("unnamed")
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub position: [f32; 3],
    pub visible: i32,
    pub rotation: [f32; 3],
    pad3: [u32; 1],
    pub point_count: i32,
    pub normal_count: i32,
    pub triangle_count: i32,
    pad0: [u32; 1],
    pub points: [[f32; 4]; MAX_MODEL_VERTICES],
    pub normals: [[f32; 4]; MAX_MODEL_VERTICES],
    pub triangles: [Triangle; MAX_MODEL_VERTICES],
    pub nodes: [NodeUniform; MAX_MODEL_VERTICES],
    pub bvh_lookup: [i32; MAX_MODEL_VERTICES],
    pad2: [u32; 7],
}

impl Default for ModelUniform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            pad0: [0; 1],
            rotation: [0.0; 3],
            pad3: [0; 1],
            point_count: 0,
            normal_count: 0,
            triangle_count: 0,
            visible: 0,
            points: [[0.0; 4]; MAX_MODEL_VERTICES],
            normals: [[0.0; 4]; MAX_MODEL_VERTICES],
            triangles: [Triangle::default(); MAX_MODEL_VERTICES],
            nodes: [NodeUniform::default(); MAX_MODEL_VERTICES],
            bvh_lookup: [0; MAX_MODEL_VERTICES],
            pad2: [0; 7],
        }
    }
}

impl ArrayBufferUniform<Model> for ModelUniform {
    fn update(&mut self, model: &Model) {
        self.rotation = model.rotation.into();
        self.position = model.position.into();
        self.point_count = model.point_count;
        self.triangle_count = model.triangle_count;
        self.visible = model.visible;
        self.points = model.points;
        self.normals = model.normals;
        self.triangles = model.triangles;

        for i in 0..self.nodes.len() {
            self.nodes[i] = model.nodes[i].into()
        }

        self.bvh_lookup = model.bvh_lookup;
    }
}

pub type ModelArrayBuffer = ArrayBuffer<MAX_MODELS, Model, ModelUniform>;
