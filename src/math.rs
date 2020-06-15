extern crate gl;
use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Neg;
use core::ops::Sub;

#[derive(Copy, Clone)]
pub struct Vec1<T> {
    pub x: T,
}
#[allow(dead_code)]
pub type Vec1f = Vec1<f32>;
#[allow(dead_code)]
pub type Vec1u = Vec1<u32>;
#[allow(dead_code)]
pub type Vec1i = Vec1<i16>;

impl<T> Vec1<T>
where
    T: From<u16>,
{
    pub fn new(x: T) -> Vec1<T> {
        Vec1 { x }
    }

    #[allow(dead_code)]
    pub fn null() -> Vec1<T> {
        Vec1::<T> { x: T::from(0) }
    }
}

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
pub type Vec2i = Vec2<i16>;

impl<T> Vec2<T>
where
    T: From<u16>,
{
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { x, y }
    }

    #[allow(dead_code)]
    pub fn null() -> Vec2<T> {
        Vec2::<T> {
            x: T::from(0),
            y: T::from(0),
        }
    }
}

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
pub type Vec3i = Vec3<i16>;

impl<T> Vec3<T>
where
    T: From<u16>,
{
    pub fn new(x: T, y: T, z: T) -> Vec3<T> {
        Vec3 { x, y, z }
    }

    #[allow(dead_code)]
    pub fn null() -> Vec3<T> {
        Vec3::<T> {
            x: T::from(0),
            y: T::from(0),
            z: T::from(0),
        }
    }
}

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
pub type Vec4i = Vec4<i16>;

impl<T> Vec4<T>
where
    T: From<u16>,
{
    pub fn new(x: T, y: T, z: T, w: T) -> Vec4<T> {
        Vec4 { x, y, z, w }
    }

    #[allow(dead_code)]
    pub fn null() -> Vec4<T> {
        Vec4::<T> {
            x: T::from(0),
            y: T::from(0),
            z: T::from(0),
            w: T::from(0),
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Mat3x3<T> {
    pub r1: Vec3<T>,
    pub r2: Vec3<T>,
    pub r3: Vec3<T>,
}
#[allow(dead_code)]
pub type Mat3x3f = Mat3x3<f32>;
#[allow(dead_code)]
pub type Mat3x3u = Mat3x3<u32>;
#[allow(dead_code)]
pub type Mat3x3i = Mat3x3<i16>;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Mat4x4<T> {
    pub r1: Vec4<T>,
    pub r2: Vec4<T>,
    pub r3: Vec4<T>,
    pub r4: Vec4<T>,
}
#[allow(dead_code)]
pub type Mat4x4f = Mat4x4<f32>;
#[allow(dead_code)]
pub type Mat4x4u = Mat4x4<u32>;
#[allow(dead_code)]
pub type Mat4x4i = Mat4x4<i16>;

impl<T> Mat4x4<T>
where
    T: From<u16>,
{
    pub fn identity() -> Mat4x4<T> {
        Mat4x4::<T> {
            r1: Vec4::<T>::new(T::from(1), T::from(0), T::from(0), T::from(0)),
            r2: Vec4::<T>::new(T::from(0), T::from(1), T::from(0), T::from(0)),
            r3: Vec4::<T>::new(T::from(0), T::from(0), T::from(1), T::from(0)),
            r4: Vec4::<T>::new(T::from(0), T::from(0), T::from(0), T::from(1)),
        }
    }

    #[allow(dead_code)]
    pub fn null() -> Mat4x4<T>
    where
        T: From<u16>,
    {
        Mat4x4::<T> {
            r1: Vec4::<T>::new(T::from(0), T::from(0), T::from(0), T::from(0)),
            r2: Vec4::<T>::new(T::from(0), T::from(0), T::from(0), T::from(0)),
            r3: Vec4::<T>::new(T::from(0), T::from(0), T::from(0), T::from(0)),
            r4: Vec4::<T>::new(T::from(0), T::from(0), T::from(0), T::from(0)),
        }
    }
}

impl<T> Add for Vec1<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
        }
    }
}

impl<T> Add for Vec2<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> Add for Vec3<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T> Add for Vec4<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl<T> Sub for Vec1<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
        }
    }
}

impl<T> Sub for Vec2<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> Sub for Vec3<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T> Sub for Vec4<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl<T> Div<T> for Vec1<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        Self { x: self.x / other }
    }
}

impl<T> Div<T> for Vec2<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl<T> Div<T> for Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        Self::Output {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl<T> Div<T> for Vec4<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
            w: self.w / other,
        }
    }
}

impl<T> Mul<T> for Vec1<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Self { x: self.x * other }
    }
}

impl<T> Mul<T> for Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<T> Mul<T> for Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Self::Output {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl<T> Mul<T> for Vec4<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
            w: self.w * other,
        }
    }
}

impl<T> Neg for Vec1<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec1::<T> { x: -self.x }
    }
}

impl<T> Neg for Vec2<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec2::<T> {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> Neg for Vec3<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3::<T> {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> Neg for Vec4<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec4::<T> {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

#[allow(dead_code)]
pub fn dot_vec1<T>(a: Vec1<T>, b: Vec1<T>) -> T
where
    T: Mul<Output = T> + Copy,
{
    a.x * b.x
}

#[allow(dead_code)]
pub fn dot_vec2<T>(a: Vec2<T>, b: Vec2<T>) -> T
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{
    a.x * b.x + a.y * b.y
}

#[allow(dead_code)]
pub fn dot_vec3<T>(a: Vec3<T>, b: Vec3<T>) -> T
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[allow(dead_code)]
pub fn dot_vec4<T>(a: Vec4<T>, b: Vec4<T>) -> T
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}

#[allow(dead_code)]
pub fn cross<T>(a: Vec3<T>, b: Vec3<T>) -> Vec3<T>
where
    T: Mul<Output = T> + Sub<Output = T> + Copy,
{
    Vec3::<T> {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

#[allow(dead_code)]
pub fn length_squared_vec1(a: Vec1f) -> f32 {
    dot_vec1(a, a)
}

#[allow(dead_code)]
pub fn length_squared_vec2(a: Vec2f) -> f32 {
    dot_vec2(a, a)
}

#[allow(dead_code)]
pub fn length_squared_vec3(a: Vec3f) -> f32 {
    dot_vec3(a, a)
}

#[allow(dead_code)]
pub fn length_squared_vec4(a: Vec4f) -> f32 {
    dot_vec4(a, a)
}

#[allow(dead_code)]
pub fn length_vec1(a: Vec1f) -> f32 {
    length_squared_vec1(a).sqrt()
}

#[allow(dead_code)]
pub fn length_vec2(a: Vec2f) -> f32 {
    length_squared_vec2(a).sqrt()
}

#[allow(dead_code)]
pub fn length_vec3(a: Vec3f) -> f32 {
    length_squared_vec3(a).sqrt()
}

#[allow(dead_code)]
pub fn length_vec4(a: Vec4f) -> f32 {
    length_squared_vec4(a).sqrt()
}

#[allow(dead_code)]
pub fn normalize_vec1(a: Vec1f) -> Vec1f {
    a / length_vec1(a)
}

#[allow(dead_code)]
pub fn normalize_vec2(a: Vec2f) -> Vec2f {
    a / length_vec2(a)
}

#[allow(dead_code)]
pub fn normalize_vec3(a: Vec3f) -> Vec3f {
    a / length_vec3(a)
}

#[allow(dead_code)]
pub fn normalize_vec4(a: Vec4f) -> Vec4f {
    a / length_vec4(a)
}

#[allow(dead_code)]
pub fn zero_vec1<T>() -> Vec1<T>
where
    T: From<i16>,
{
    Vec1::<T> { x: T::from(0) }
}

#[allow(dead_code)]
pub fn zero_vec2<T>() -> Vec2<T>
where
    T: From<i16>,
{
    Vec2::<T> {
        x: T::from(0),
        y: T::from(0),
    }
}

#[allow(dead_code)]
pub fn zero_vec3<T>() -> Vec3<T>
where
    T: From<i16>,
{
    Vec3::<T> {
        x: T::from(0),
        y: T::from(0),
        z: T::from(0),
    }
}

#[allow(dead_code)]
pub fn zero_vec4<T>() -> Vec4<T>
where
    T: From<i16>,
{
    Vec4::<T> {
        x: T::from(0),
        y: T::from(0),
        z: T::from(0),
        w: T::from(0),
    }
}

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

impl VecGLTypeValue for i16 {
    const GL_VALUE: u32 = gl::INT;
}

impl VecGLTypeValue for u32 {
    const GL_VALUE: u32 = gl::UNSIGNED_INT;
}

impl VecGLTypeValue for f32 {
    const GL_VALUE: u32 = gl::FLOAT;
}

impl<T> Mul<Vec3<T>> for Mat3x3<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{
    type Output = Vec3<T>;

    fn mul(self, other: Vec3<T>) -> Self::Output {
        Self::Output {
            x: dot_vec3(self.r1, other),
            y: dot_vec3(self.r2, other),
            z: dot_vec3(self.r3, other),
        }
    }
}

impl<T> Mul<Vec4<T>> for Mat4x4<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{
    type Output = Vec4<T>;

    fn mul(self, other: Vec4<T>) -> Self::Output {
        Self::Output {
            x: dot_vec4(self.r1, other),
            y: dot_vec4(self.r2, other),
            z: dot_vec4(self.r3, other),
            w: dot_vec4(self.r4, other),
        }
    }
}

impl<T> Mul<Mat3x3<T>> for Mat3x3<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + From<u16>,
{
    type Output = Mat3x3<T>;

    fn mul(self, other: Mat3x3<T>) -> Self::Output {
        Mat3x3::<T> {
            r1: Vec3::<T> {
                x: dot_vec3(self.r1, Vec3::<T>::new(other.r1.x, other.r2.x, other.r3.x)),
                y: dot_vec3(self.r1, Vec3::<T>::new(other.r1.y, other.r2.y, other.r3.y)),
                z: dot_vec3(self.r1, Vec3::<T>::new(other.r1.z, other.r2.z, other.r3.z)),
            },
            r2: Vec3::<T> {
                x: dot_vec3(self.r2, Vec3::<T>::new(other.r1.x, other.r2.x, other.r3.x)),
                y: dot_vec3(self.r2, Vec3::<T>::new(other.r1.y, other.r2.y, other.r3.y)),
                z: dot_vec3(self.r2, Vec3::<T>::new(other.r1.z, other.r2.z, other.r3.z)),
            },
            r3: Vec3::<T> {
                x: dot_vec3(self.r3, Vec3::<T>::new(other.r1.x, other.r2.x, other.r3.x)),
                y: dot_vec3(self.r3, Vec3::<T>::new(other.r1.y, other.r2.y, other.r3.y)),
                z: dot_vec3(self.r3, Vec3::<T>::new(other.r1.z, other.r2.z, other.r3.z)),
            },
        }
    }
}

impl<T> Mul<Mat4x4<T>> for Mat4x4<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + From<u16>,
{
    type Output = Mat4x4<T>;

    fn mul(self, other: Mat4x4<T>) -> Self::Output {
        Mat4x4::<T> {
            r1: Vec4::<T> {
                x: dot_vec4(
                    self.r1,
                    Vec4::<T>::new(other.r1.x, other.r2.x, other.r3.x, other.r4.x),
                ),
                y: dot_vec4(
                    self.r1,
                    Vec4::<T>::new(other.r1.y, other.r2.y, other.r3.y, other.r4.y),
                ),
                z: dot_vec4(
                    self.r1,
                    Vec4::<T>::new(other.r1.z, other.r2.z, other.r3.z, other.r4.z),
                ),
                w: dot_vec4(
                    self.r1,
                    Vec4::<T>::new(other.r1.w, other.r2.w, other.r3.w, other.r4.w),
                ),
            },
            r2: Vec4::<T> {
                x: dot_vec4(
                    self.r2,
                    Vec4::<T>::new(other.r1.x, other.r2.x, other.r3.x, other.r4.x),
                ),
                y: dot_vec4(
                    self.r2,
                    Vec4::<T>::new(other.r1.y, other.r2.y, other.r3.y, other.r4.y),
                ),
                z: dot_vec4(
                    self.r2,
                    Vec4::<T>::new(other.r1.z, other.r2.z, other.r3.z, other.r4.z),
                ),
                w: dot_vec4(
                    self.r2,
                    Vec4::<T>::new(other.r1.w, other.r2.w, other.r3.w, other.r4.w),
                ),
            },
            r3: Vec4::<T> {
                x: dot_vec4(
                    self.r3,
                    Vec4::<T>::new(other.r1.x, other.r2.x, other.r3.x, other.r4.x),
                ),
                y: dot_vec4(
                    self.r3,
                    Vec4::<T>::new(other.r1.y, other.r2.y, other.r3.y, other.r4.y),
                ),
                z: dot_vec4(
                    self.r3,
                    Vec4::<T>::new(other.r1.z, other.r2.z, other.r3.z, other.r4.z),
                ),
                w: dot_vec4(
                    self.r3,
                    Vec4::<T>::new(other.r1.w, other.r2.w, other.r3.w, other.r4.w),
                ),
            },
            r4: Vec4::<T> {
                x: dot_vec4(
                    self.r4,
                    Vec4::<T>::new(other.r1.x, other.r2.x, other.r3.x, other.r4.x),
                ),
                y: dot_vec4(
                    self.r4,
                    Vec4::<T>::new(other.r1.y, other.r2.y, other.r3.y, other.r4.y),
                ),
                z: dot_vec4(
                    self.r4,
                    Vec4::<T>::new(other.r1.z, other.r2.z, other.r3.z, other.r4.z),
                ),
                w: dot_vec4(
                    self.r4,
                    Vec4::<T>::new(other.r1.w, other.r2.w, other.r3.w, other.r4.w),
                ),
            },
        }
    }
}

impl<T> From<Mat3x3<T>> for Mat4x4<T>
where
    T: From<u16>,
{
    fn from(src: Mat3x3<T>) -> Mat4x4<T> {
        Mat4x4::<T> {
            r1: Vec4::<T>::new(src.r1.x, src.r1.y, src.r1.z, T::from(0)),
            r2: Vec4::<T>::new(src.r2.x, src.r2.y, src.r2.z, T::from(0)),
            r3: Vec4::<T>::new(src.r3.x, src.r3.y, src.r3.z, T::from(0)),
            r4: Vec4::<T>::new(T::from(0), T::from(0), T::from(0), T::from(1)),
        }
    }
}

#[allow(dead_code)]
pub fn null_mat3x3<T>() -> Mat3x3<T>
where
    T: From<u16>,
{
    Mat3x3::<T> {
        r1: Vec3::<T>::new(T::from(0), T::from(0), T::from(0)),
        r2: Vec3::<T>::new(T::from(0), T::from(0), T::from(0)),
        r3: Vec3::<T>::new(T::from(0), T::from(0), T::from(0)),
    }
}

#[allow(dead_code)]
pub fn identity_mat3x3<T>() -> Mat3x3<T>
where
    T: From<u16>,
{
    Mat3x3::<T> {
        r1: Vec3::<T>::new(T::from(1), T::from(0), T::from(0)),
        r2: Vec3::<T>::new(T::from(0), T::from(1), T::from(0)),
        r3: Vec3::<T>::new(T::from(0), T::from(0), T::from(1)),
    }
}

#[allow(dead_code)]
pub fn x_rotation_mat3x3(angle: f32) -> Mat3x3f {
    Mat3x3f {
        r1: Vec3f::new(1., 0., 0.),
        r2: Vec3f::new(0., angle.cos(), -angle.sin()),
        r3: Vec3f::new(0., angle.sin(), angle.cos()),
    }
}

#[allow(dead_code)]
pub fn y_rotation_mat3x3(angle: f32) -> Mat3x3f {
    Mat3x3f {
        r1: Vec3f::new(angle.cos(), 0., angle.sin()),
        r2: Vec3f::new(0., 1., 0.),
        r3: Vec3f::new(-angle.sin(), 0., angle.cos()),
    }
}

#[allow(dead_code)]
pub fn z_rotation_mat3x3(angle: f32) -> Mat3x3f {
    Mat3x3f {
        r1: Vec3f::new(angle.cos(), -angle.sin(), 0.),
        r2: Vec3f::new(angle.sin(), angle.cos(), 0.),
        r3: Vec3f::new(0., 0., 1.),
    }
}

#[allow(dead_code)]
pub fn x_rotation_mat4x4(angle: f32) -> Mat4x4f {
    Mat4x4f::from(x_rotation_mat3x3(angle))
}

#[allow(dead_code)]
pub fn y_rotation_mat4x4(angle: f32) -> Mat4x4f {
    Mat4x4f::from(y_rotation_mat3x3(angle))
}

#[allow(dead_code)]
pub fn z_rotation_mat4x4(angle: f32) -> Mat4x4f {
    Mat4x4f::from(z_rotation_mat3x3(angle))
}

#[allow(dead_code)]
pub fn scale_mat3x3(scale: Vec3f) -> Mat3x3f {
    let mut mat = identity_mat3x3();

    mat.r1.x = scale.x;
    mat.r2.y = scale.y;
    mat.r3.z = scale.z;

    mat
}

#[allow(dead_code)]
pub fn scale_uniform_mat3x3(scale: f32) -> Mat3x3f {
    scale_mat3x3(Vec3f {
        x: scale,
        y: scale,
        z: scale,
    })
}

#[allow(dead_code)]
pub fn scale_mat4x4(scale: Vec3f) -> Mat4x4f {
    Mat4x4f::from(scale_mat3x3(scale))
}

#[allow(dead_code)]
pub fn scale_uniform_mat4x4(scale: f32) -> Mat4x4f {
    Mat4x4::from(scale_uniform_mat3x3(scale))
}

#[allow(dead_code)]
pub fn tranlation_mat4x4(offset: Vec3f) -> Mat4x4f {
    let mut mat = Mat4x4f::identity();
    mat.r1.w = offset.x;
    mat.r2.w = offset.y;
    mat.r3.w = offset.z;
    mat
}

#[allow(dead_code)]
pub fn orthographics_projection_planes_mat4x4(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4x4f {
    let mut proj = Mat4x4f::identity();

    proj.r1.x = 2_f32 / (right - left);
    proj.r1.w = -(right + left) / (right - left);
    proj.r2.y = 2_f32 / (top - bottom);
    proj.r2.w = -(top + bottom) / (top - bottom);
    proj.r3.z = 2_f32 / (far - near);
    proj.r3.w = -(far + near) / (far - near);

    proj
}

#[allow(dead_code)]
pub fn perspective_projection_planes_mat4x4(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4x4f {
    let mut proj = Mat4x4f::null();

    proj.r1.x = 2_f32 * near / (right - left);
    proj.r1.z = (right + left) / (right - left);
    proj.r2.y = 2_f32 * near / (top - bottom);
    proj.r2.z = (top + bottom) / (top - bottom);
    proj.r3.z = -(far + near) / (far - near);
    proj.r3.w = -2_f32 * far * near / (far - near);
    proj.r4.z = -1_f32;

    proj
}

#[allow(dead_code)]
pub fn perspective_projection_mat4x4(vfov: f32, aspect: f32, near: f32, far: f32) -> Mat4x4f {
    let c = 1_f32 / (vfov / 2_f32).tan();

    let mut proj = Mat4x4f::null();
    proj.r1.x = c / aspect;
    proj.r2.y = c;
    proj.r3.z = -(far + near) / (far - near);
    proj.r3.w = -(2_f32 * far * near) / (far - near);
    proj.r4.z = -1_f32;

    proj
}

#[allow(dead_code)]
pub fn create_camera_mat4x4(pos: Vec3f, yaw: f32, pitch: f32) -> Mat4x4f {
    tranlation_mat4x4(pos) * y_rotation_mat4x4(yaw) * x_rotation_mat4x4(pitch)
}

#[allow(dead_code)]
pub fn create_view_mat4x4(pos: Vec3f, yaw: f32, pitch: f32) -> Mat4x4f {
    x_rotation_mat4x4(-pitch) * y_rotation_mat4x4(-yaw) * tranlation_mat4x4(-pos)
}
