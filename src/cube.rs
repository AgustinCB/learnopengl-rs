const VERTICES: [f32; 108] = [
    -0.5f32, -0.5f32, -0.5f32,
    0.5f32, -0.5f32, -0.5f32,
    0.5f32,  0.5f32, -0.5f32,
    0.5f32,  0.5f32, -0.5f32,
    -0.5f32,  0.5f32, -0.5f32,
    -0.5f32, -0.5f32, -0.5f32,

    -0.5f32, -0.5f32,  0.5f32,
    0.5f32, -0.5f32,  0.5f32,
    0.5f32,  0.5f32,  0.5f32,
    0.5f32,  0.5f32,  0.5f32,
    -0.5f32,  0.5f32,  0.5f32,
    -0.5f32, -0.5f32,  0.5f32,

    -0.5f32,  0.5f32,  0.5f32,
    -0.5f32,  0.5f32, -0.5f32,
    -0.5f32, -0.5f32, -0.5f32,
    -0.5f32, -0.5f32, -0.5f32,
    -0.5f32, -0.5f32,  0.5f32,
    -0.5f32,  0.5f32,  0.5f32,

    0.5f32,  0.5f32,  0.5f32,
    0.5f32,  0.5f32, -0.5f32,
    0.5f32, -0.5f32, -0.5f32,
    0.5f32, -0.5f32, -0.5f32,
    0.5f32, -0.5f32,  0.5f32,
    0.5f32,  0.5f32,  0.5f32,

    -0.5f32, -0.5f32, -0.5f32,
    0.5f32, -0.5f32, -0.5f32,
    0.5f32, -0.5f32,  0.5f32,
    0.5f32, -0.5f32,  0.5f32,
    -0.5f32, -0.5f32,  0.5f32,
    -0.5f32, -0.5f32, -0.5f32,

    -0.5f32,  0.5f32, -0.5f32,
    0.5f32,  0.5f32, -0.5f32,
    0.5f32,  0.5f32,  0.5f32,
    0.5f32,  0.5f32,  0.5f32,
    -0.5f32,  0.5f32,  0.5f32,
    -0.5f32,  0.5f32, -0.5f32,
];

const NORMALS: [f32; 108] = [
    0f32, 0f32, -1f32,
    0f32, 0f32, -1f32,
    0f32, 0f32, -1f32,
    0f32, 0f32, -1f32,
    0f32, 0f32, -1f32,
    0f32, 0f32, -1f32,

    0f32, 0f32, 1f32,
    0f32, 0f32, 1f32,
    0f32, 0f32, 1f32,
    0f32, 0f32, 1f32,
    0f32, 0f32, 1f32,
    0f32, 0f32, 1f32,

    -1f32, 0f32, 0f32,
    -1f32, 0f32, 0f32,
    -1f32, 0f32, 0f32,
    -1f32, 0f32, 0f32,
    -1f32, 0f32, 0f32,
    -1f32, 0f32, 0f32,

    1f32, 0f32, 0f32,
    1f32, 0f32, 0f32,
    1f32, 0f32, 0f32,
    1f32, 0f32, 0f32,
    1f32, 0f32, 0f32,
    1f32, 0f32, 0f32,

    0f32, -1f32, 0f32,
    0f32, -1f32, 0f32,
    0f32, -1f32, 0f32,
    0f32, -1f32, 0f32,
    0f32, -1f32, 0f32,
    0f32, -1f32, 0f32,

    0f32, 1f32, 0f32,
    0f32, 1f32, 0f32,
    0f32, 1f32, 0f32,
    0f32, 1f32, 0f32,
    0f32, 1f32, 0f32,
    0f32, 1f32, 0f32,
];

pub struct Cube {
    content: Vec<f32>,
    size: u8,
}

impl Cube {
    pub fn new() -> Cube {
        Cube {
            content: VERTICES.to_vec(),
            size: 3,
        }
    }

    pub fn with_normals() -> Cube {
        let mut content = Vec::with_capacity(VERTICES.len() + NORMALS.len());
        for i in 0..VERTICES.len() / 3 {
            let temp = vec![
                VERTICES[i * 3 + 0],
                VERTICES[i * 3 + 1],
                VERTICES[i * 3 + 2],
                NORMALS[i * 3 + 0],
                NORMALS[i * 3 + 1],
                NORMALS[i * 3 + 2],
            ];
            content.extend(temp);
        }
        Cube {
            content,
            size: 6,
        }
    }

    pub fn content(&self) -> &[f32] {
        &self.content
    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn add_texture(&mut self, texture_coordinates: &[f32]) {
        let mut new_content = Vec::new();
        for i in 0..VERTICES.len() / 3 as usize {
            let mut temp = Vec::with_capacity(self.size as _);
            for j in 0..(self.size as _) {
                temp.push(self.content[i * self.size as usize + j])
            }
            temp.extend([texture_coordinates[i * 2], texture_coordinates[i * 2 + 1]]);
            new_content.extend(&temp);
        }
        self.content = new_content;
        self.size += 2;
    }
}