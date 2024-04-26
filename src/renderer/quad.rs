use super::vertex::Vertex;

pub struct Quad {
    vertices: [Vertex; 6],
}

impl Quad {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Quad {
            vertices: Quad::generate_vertices(x, y, w, h),
        }
    }

    pub fn get_vertices(&self) -> &[Vertex; 6] {
        &self.vertices
    }

    fn generate_vertices(x: f32, y: f32, w: f32, h: f32) -> [Vertex; 6] {
        let x1 = x * 2.0 - 1.0;
        let y1 = y * 2.0 - 1.0;
        let x2 = (x + w) * 2.0 - 1.0;
        let y2 = (y + h) * 2.0 - 1.0;
        
        [
            Vertex {
                position: [x1, y1, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [x2, y1, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [x2, y2, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [x1, y1, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [x2, y2, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [x1, y2, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ]
    }
}
