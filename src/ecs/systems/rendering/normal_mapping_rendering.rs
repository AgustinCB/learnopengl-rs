use crate::program::Program;
use crate::shader_loader::{ShaderLoader, ShaderType};

const NORMAL_MAPPING_VERTEX: &'static str = "23.1-normal_mapping_vertex.glsl";
const NORMAL_MAPPING_GEOMETRY: &'static str = "23.1-normal_mapping_geometry.glsl";
const NORMAL_MAPPING_FRAGMENT: &'static str = "23.1-normal_mapping_fragment.glsl";

pub struct NormalMappingRendering {
    pub(crate) program: Program,
}

impl NormalMappingRendering {
    pub fn new(shader_loader: &ShaderLoader) -> Result<NormalMappingRendering, String> {
        let program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, NORMAL_MAPPING_VERTEX)?,
            shader_loader.load(ShaderType::Geometry, NORMAL_MAPPING_GEOMETRY)?,
            shader_loader.load(ShaderType::Fragment, NORMAL_MAPPING_FRAGMENT)?,
            /*
            shader_loader.load(ShaderType::Vertex, "17.1-uniform_buffer_objects_vertex.glsl")?,
            shader_loader.load(ShaderType::Fragment, "12.1-modelloading.glsl")?,
            */
        ])?;
        program.bind_uniform_block("Matrices", 0);
        Ok(NormalMappingRendering {
            program
        })
    }
}