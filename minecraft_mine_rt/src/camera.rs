use crate::math::{Vec3, Ray};

pub struct Camera {
    pub eye: Vec3, pub target: Vec3, pub up: Vec3,
    pub fov_deg: f32, pub aspect: f32
}
impl Camera {
    pub fn new(eye:Vec3, target:Vec3, up:Vec3, fov_deg:f32, aspect:f32)->Self{
        Self{eye,target,up,fov_deg,aspect}
    }
    pub fn ray_for(&self, x:usize, y:usize, w:usize, h:usize)->Ray{
        let fov = (self.fov_deg.to_radians()*0.5).tan();
        let px = ( ( (x as f32 + 0.5)/w as f32 )*2.0 - 1.0 ) * self.aspect * fov;
        let py = ( 1.0 - ( (y as f32 + 0.5)/h as f32 )*2.0 ) * fov;

        let fwd = self.target.sub(self.eye).norm();
        let right = fwd.cross(self.up).norm();
        let upv = right.cross(fwd).norm();

        let dir = right.mul(px).add(upv.mul(py)).add(fwd).norm();
        Ray{ o:self.eye, d:dir }
    }
}
