mod math;      use math::{Vec3, Ray};
mod color;     use color::Color;
mod ppm;       use ppm::write_ppm;
mod aabb;
mod material;
mod texture;
mod skybox;
mod camera;    use camera::Camera;
mod renderer;  use renderer::trace;
mod scene;     use scene::{build_scene_minecraft, build_scene_basic};

use std::fs;

fn main() {
    // Config render
    let width: usize = 640;
    let height: usize = 360;
    let frames: usize = 60; 
    fs::create_dir_all("out").ok();

    for f in 0..frames {
        let t = f as f32 / (frames as f32 - 1.0); // 0..1

        // let (scene, anim) = build_scene_basic(t);
        let (scene, anim) = build_scene_minecraft(t);

        // Cámara en órbita alrededor del centro de la escena
        let target = Vec3::new(8.0, 2.0, 8.0);
        let eye = Vec3::new(
            target.x + anim.radius * anim.angle.cos(),
            anim.eye_h,
            target.z + anim.radius * anim.angle.sin(),
        );
        let cam = Camera::new(eye, target, Vec3::new(0.0, 1.0, 0.0), 60.0, width as f32 / height as f32);

        // Framebuffer
        let mut rgb = vec![0u8; width * height * 3];

        for y in 0..height {
            for x in 0..width {
                let ray: Ray = cam.ray_for(x, y, width, height);
                let col: Color = trace(&scene, ray);
                let p = (y * width + x) * 3;
                let [r, g, b] = col.to_u8_gamma();
                rgb[p] = r; rgb[p + 1] = g; rgb[p + 2] = b;
            }
        }

        let path = format!("out/frame_{:04}.ppm", f);
        write_ppm(&path, width, height, &rgb).expect("no pude escribir el PPM");
        println!("Frame {} listo: {}", f, path);
    }

    println!("Listo. Combina los frames con ffmpeg:");
    println!(r#"  ffmpeg -framerate 30 -i out/frame_%04d.ppm -pix_fmt yuv420p -crf 18 diorama.mp4"#);
}
