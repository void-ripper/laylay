
use winit::dpi::PhysicalSize;

use crate::math::matrix::{self, Matrix};

use super::node::{Node, NodePtr};

pub struct Camera {
    pub node: NodePtr,
    pub projection: Matrix,
    pub transform: Matrix,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
      
    pub async fn perspective( eye: &[f32; 3], target: &[f32; 3], aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        let node = Node::new();
        let mut m = matrix::new();
        matrix::translate(&mut m, eye);
        matrix::look_at(&mut m, target, &[0.0, 1.0, 0.0]);
        *node.transform.write().await = m;
        let inv = matrix::inverse(&m);

        let mut p = matrix::new();
        matrix::perspective(&mut p, fovy, aspect, znear, zfar);
        // matrix::transpose(&mut p);
        // matrix::mul_assign(&mut p, &m);

        #[rustfmt::skip]
        let mut opengl_to_wgpu = [ 
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        ];
        // matrix::mul_assign(&mut m, &opengl_to_wgpu);
        matrix::mul_assign(&mut opengl_to_wgpu, &m);
        matrix::mul_assign(&mut opengl_to_wgpu, &p);
        // matrix::translate(&mut opengl_to_wgpu, &[0.0, 0.5, 0.0]);
        
        Self {
            node,
            projection: p,
            transform: opengl_to_wgpu,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
        matrix::perspective(&mut self.projection, self.fovy, self.aspect, self.znear, self.zfar);
    }

    pub async fn update(&mut self) {
        #[rustfmt::skip]
        let mut opengl_to_wgpu = [ 
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        ];
        // matrix::mul_assign(&mut m, &opengl_to_wgpu);
        matrix::mul_assign(&mut opengl_to_wgpu, &self.projection);
        let inv = matrix::inverse(&*self.node.transform.read().await);
        matrix::mul_assign(&mut opengl_to_wgpu, &inv);

        self.transform = opengl_to_wgpu;
    }
}
