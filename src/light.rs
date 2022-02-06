use nalgebra::{Matrix4, Translation3, UnitVector3, Vector3};
use crate::program::Program;

pub trait Light {
    fn set_light_in_program(&self, program: &Program, name: &str);
    fn set_light_drawing_program(&self, color_name: &str, model_name: &str, view: (&str, &Matrix4<f32>), projection: (&str, &Matrix4<f32>));
}

pub struct DirectionalLight {
    direction: UnitVector3<f32>,
    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
}

impl DirectionalLight {
    pub fn new(
        direction: UnitVector3<f32>,
        ambient: Vector3<f32>,
        diffuse: Vector3<f32>,
        specular: Vector3<f32>,
    ) -> DirectionalLight {
        DirectionalLight {
            direction,
            ambient,
            diffuse,
            specular,
        }
    }
}

impl Light for DirectionalLight {
    fn set_light_in_program(&self, program: &Program, name: &str) {
        program.use_program();
        program.set_uniform_v3(&(name.to_string() + ".direction"), self.direction.xyz());
        program.set_uniform_v3(&(name.to_string() + ".ambient"), self.ambient);
        program.set_uniform_v3(&(name.to_string() + ".diffuse"), self.diffuse);
        program.set_uniform_v3(&(name.to_string() + ".specular"), self.specular);
    }

    fn set_light_drawing_program(&self, _color_name: &str, _model_name: &str, _view: (&str, &Matrix4<f32>), _projection: (&str, &Matrix4<f32>)) {
    }
}

pub struct PointLight<'a> {
    position: Vector3<f32>,
    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
    constant: f32,
    linear: f32,
    quadratic: f32,
    model: Matrix4<f32>,
    program: &'a Program,
}

impl<'a> PointLight<'a> {
    pub fn new(
        position: Vector3<f32>,
        ambient: Vector3<f32>,
        diffuse: Vector3<f32>,
        specular: Vector3<f32>,
        constant: f32,
        linear: f32,
        quadratic: f32,
        program: &'a Program,
    ) -> PointLight<'a> {
        let model = Translation3::from(position).to_homogeneous();
        PointLight {
            position,
            model,
            ambient,
            diffuse,
            specular,
            constant,
            linear,
            quadratic,
            program,
        }
    }
}

impl<'a> Light for PointLight<'a> {
    fn set_light_in_program(&self, program: &Program, name: &str) {
        program.use_program();
        program.set_uniform_v3(&(name.to_string() + ".position"), self.position);
        program.set_uniform_v3(&(name.to_string() + ".ambient"), self.ambient);
        program.set_uniform_v3(&(name.to_string() + ".diffuse"), self.diffuse);
        program.set_uniform_v3(&(name.to_string() + ".specular"), self.specular);
        program.set_uniform_f1(&(name.to_string() + ".constant"), self.constant);
        program.set_uniform_f1(&(name.to_string() + ".linear"), self.linear);
        program.set_uniform_f1(&(name.to_string() + ".quadratic"), self.quadratic);
    }

    fn set_light_drawing_program(&self, color_name: &str, model_name: &str, view: (&str, &Matrix4<f32>), projection: (&str, &Matrix4<f32>)) {
        self.program.use_program();
        self.program.set_uniform_matrix4(model_name, &self.model);
        self.program.set_uniform_matrix4(projection.0, &projection.1);
        self.program.set_uniform_matrix4(view.0, &view.1);
        self.program.set_uniform_v3(color_name, self.specular);
    }
}

pub struct SpotLight<'a> {
    direction: UnitVector3<f32>,
    position: Vector3<f32>,
    cut_ff: f32,
    outer_cut_off: f32,
    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
    constant: f32,
    linear: f32,
    quadratic: f32,
    model: Matrix4<f32>,
    program: &'a Program,
}

impl<'a> SpotLight<'a> {
    pub fn new(
        direction: UnitVector3<f32>,
        position: Vector3<f32>,
        cut_ff: f32,
        outer_cut_off: f32,
        ambient: Vector3<f32>,
        diffuse: Vector3<f32>,
        specular: Vector3<f32>,
        constant: f32,
        linear: f32,
        quadratic: f32,
        program: &'a Program,
    ) -> SpotLight<'a> {
        let model = Translation3::from(position).to_homogeneous();
        SpotLight {
            direction,
            position,
            cut_ff,
            outer_cut_off,
            model,
            ambient,
            diffuse,
            specular,
            constant,
            linear,
            quadratic,
            program,
        }
    }

    pub fn set_position(&mut self, new_position: Vector3<f32>) {
        self.position = new_position;
        self.model = Translation3::from(self.position).to_homogeneous();
    }

    pub fn set_direction(&mut self, new_direction: UnitVector3<f32>) {
        self.direction = new_direction;
    }
}

impl<'a> Light for SpotLight<'a> {
    fn set_light_in_program(&self, program: &Program, name: &str) {
        program.use_program();
        program.set_uniform_v3(&(name.to_string() + ".direction"), self.direction.xyz());
        program.set_uniform_v3(&(name.to_string() + ".position"), self.position);
        program.set_uniform_f1(&(name.to_string() + ".cutOff"), self.cut_ff);
        program.set_uniform_f1(&(name.to_string() + ".outerCutOff"), self.outer_cut_off);
        program.set_uniform_v3(&(name.to_string() + ".ambient"), self.ambient);
        program.set_uniform_v3(&(name.to_string() + ".diffuse"), self.diffuse);
        program.set_uniform_v3(&(name.to_string() + ".specular"), self.specular);
        program.set_uniform_f1(&(name.to_string() + ".constant"), self.constant);
        program.set_uniform_f1(&(name.to_string() + ".linear"), self.linear);
        program.set_uniform_f1(&(name.to_string() + ".quadratic"), self.quadratic);
    }

    fn set_light_drawing_program(&self, color_name: &str, model_name: &str, view: (&str, &Matrix4<f32>), projection: (&str, &Matrix4<f32>)) {
        self.program.use_program();
        self.program.set_uniform_matrix4(model_name, &self.model);
        self.program.set_uniform_matrix4(projection.0, &projection.1);
        self.program.set_uniform_matrix4(view.0, &view.1);
        self.program.set_uniform_v3(color_name, self.specular);
    }
}
