use crate::color::Color;
use crate::math::Vec3;

// Mezcla de "día" y "noche" en función de k∈[0,1] (0=day,1=night)
pub fn sample_sky(dir:Vec3, k:f32)->Color{
    let y = dir.y.clamp(-1.0,1.0);
    // Día: azul degradado; Noche: azul oscuro con leve tono
    let day = Color::new(0.45, 0.65, 0.95).mul(0.6 + 0.4*(y*0.5+0.5));
    let night = Color::new(0.05, 0.08, 0.15).mul(0.5 + 0.5*(y*0.5+0.5));
    Color::lerp(day, night, k)
}
