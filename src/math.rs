extern crate gl;

#[derive(Copy, Clone)]
pub struct Vec1<T> {
    pub x: T,
}
#[allow(dead_code)]
pub type Vec1f = Vec1<f32>;
#[allow(dead_code)]
pub type Vec1u = Vec1<u32>;
#[allow(dead_code)]
pub type Vec1i = Vec1<i32>;

#[derive(Copy, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}
#[allow(dead_code)]
pub type Vec2f = Vec2<f32>;
#[allow(dead_code)]
pub type Vec2u = Vec2<u32>;
#[allow(dead_code)]
pub type Vec2i = Vec2<i32>;

#[derive(Copy, Clone)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}
#[allow(dead_code)]
pub type Vec3f = Vec3<f32>;
#[allow(dead_code)]
pub type Vec3u = Vec3<u32>;
#[allow(dead_code)]
pub type Vec3i = Vec3<i32>;

#[derive(Copy, Clone)]
pub struct Vec4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}
#[allow(dead_code)]
pub type Vec4f = Vec4<f32>;
#[allow(dead_code)]
pub type Vec4u = Vec4<u32>;
#[allow(dead_code)]
pub type Vec4i = Vec4<i32>;

pub trait VecDimensions<T> {
    const DIMENSIONS: u32;
}

impl<T> VecDimensions<T> for Vec1f {
    const DIMENSIONS: u32 = 1;
}

impl<T> VecDimensions<T> for Vec1i {
    const DIMENSIONS: u32 = 1;
}

impl<T> VecDimensions<T> for Vec1u {
    const DIMENSIONS: u32 = 1;
}

impl<T> VecDimensions<T> for Vec2f {
    const DIMENSIONS: u32 = 2;
}

impl<T> VecDimensions<T> for Vec2i {
    const DIMENSIONS: u32 = 2;
}

impl<T> VecDimensions<T> for Vec2u {
    const DIMENSIONS: u32 = 2;
}

impl<T> VecDimensions<T> for Vec3f {
    const DIMENSIONS: u32 = 3;
}

impl<T> VecDimensions<T> for Vec3i {
    const DIMENSIONS: u32 = 3;
}

impl<T> VecDimensions<T> for Vec3u {
    const DIMENSIONS: u32 = 3;
}

impl<T> VecDimensions<T> for Vec4f {
    const DIMENSIONS: u32 = 4;
}

impl<T> VecDimensions<T> for Vec4i {
    const DIMENSIONS: u32 = 4;
}

impl<T> VecDimensions<T> for Vec4u {
    const DIMENSIONS: u32 = 4;
}

pub trait VecGLTypeValue {
    const GL_VALUE: u32;
}

pub trait VecGLTypeTrait {
    const GL_TYPE: u32;
}

impl<T: VecGLTypeValue> VecGLTypeTrait for Vec1<T> {
    const GL_TYPE: u32 = T::GL_VALUE;
}

impl<T: VecGLTypeValue> VecGLTypeTrait for Vec2<T> {
    const GL_TYPE: u32 = T::GL_VALUE;
}

impl<T: VecGLTypeValue> VecGLTypeTrait for Vec3<T> {
    const GL_TYPE: u32 = T::GL_VALUE;
}

impl<T: VecGLTypeValue> VecGLTypeTrait for Vec4<T> {
    const GL_TYPE: u32 = T::GL_VALUE;
}

impl VecGLTypeValue for i32 {
    const GL_VALUE: u32 = gl::INT;
}

impl VecGLTypeValue for u32 {
    const GL_VALUE: u32 = gl::UNSIGNED_INT;
}

impl VecGLTypeValue for f32 {
    const GL_VALUE: u32 = gl::FLOAT;
}
