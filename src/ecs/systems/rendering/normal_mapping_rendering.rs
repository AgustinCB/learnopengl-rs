use crate::ecs::components::Mesh;
use crate::program::Program;
use crate::shader_loader::{ShaderLoader, ShaderType};

const DYNAMIC_CALCULATION_NORMAL_MAPPING_VERTEX: &'static str = "23.1-normal_mapping_vertex.glsl";
const DYNAMIC_CALCULATION_NORMAL_MAPPING_GEOMETRY: &'static str = "23.1-normal_mapping_geometry.glsl";
const DYNAMIC_CALCULATION_NORMAL_MAPPING_FRAGMENT: &'static str = "23.1-normal_mapping_fragment.glsl";

const PRECOMPUTED_NORMAL_MAPPING_VERTEX: &'static str = "23.2-normal_mapping_vertex.glsl";
const PRECOMPUTED_NORMAL_MAPPING_FRAGMENT: &'static str = "23.2-normal_mapping_fragment.glsl";

pub struct NormalMappingRendering {
    pub(crate) dynamic_calculation_program: Program,
    pub(crate) precomputed_program: Program,
}

impl NormalMappingRendering {
    pub fn new(shader_loader: &ShaderLoader) -> Result<NormalMappingRendering, String> {
        let dynamic_calculation_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, DYNAMIC_CALCULATION_NORMAL_MAPPING_VERTEX)?,
            shader_loader.load(ShaderType::Geometry, DYNAMIC_CALCULATION_NORMAL_MAPPING_GEOMETRY)?,
            shader_loader.load(ShaderType::Fragment, DYNAMIC_CALCULATION_NORMAL_MAPPING_FRAGMENT)?,
        ])?;
        let precomputed_program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, PRECOMPUTED_NORMAL_MAPPING_VERTEX)?,
            shader_loader.load(ShaderType::Fragment, PRECOMPUTED_NORMAL_MAPPING_FRAGMENT)?,
        ])?;
        dynamic_calculation_program.bind_uniform_block("Matrices", 0);
        precomputed_program.bind_uniform_block("Matrices", 0);
        dynamic_calculation_program.use_program();
        dynamic_calculation_program.set_uniform_f1("height_scale", 0.1f32);
        precomputed_program.use_program();
        precomputed_program.set_uniform_f1("height_scale", 0.1f32);
        Ok(NormalMappingRendering {
            dynamic_calculation_program,
            precomputed_program,
        })
    }

    pub fn get_program_for_mesh(&self, mesh: &Mesh) -> &Program {
        if mesh.tangents.is_some() && mesh.bitangents.is_some() {
            &self.precomputed_program
        } else {
            &self.dynamic_calculation_program
        }
    }
}