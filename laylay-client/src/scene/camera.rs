use crate::math::matrix::{self, Matrix};

use super::node::{Node, NodePtr};

pub struct Camera {
    pub node: NodePtr,
    pub transform: Matrix,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn perspective(eye: &[f32; 3], target: &[f32; 3], aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        let mut m = matrix::new();
        matrix::identity(&mut m);
        matrix::translate(&mut m, eye);
        matrix::look_at(&mut m, target, &[0.0, 1.0, 0.0]);

        let mut p = matrix::new();
        matrix::identity(&mut p);
        matrix::perspective(&mut p, fovy, aspect, znear, zfar);
        matrix::mul_assign(&mut m, &p);
        
        Self {
            node: Node::new(),
            transform: m,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }
}
