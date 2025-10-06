use crate::color::Color;

pub struct Material {
    pub tex_id: usize,
    pub albedo: f32,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: f32,
    pub shininess: f32,
    pub emissive: Color, // (0,0,0) para no emisivo
}
