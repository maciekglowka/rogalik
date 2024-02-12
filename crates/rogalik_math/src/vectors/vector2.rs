use num_traits::Num;
use std::{
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign}
};

pub type Vector2i = Vector2<i32>;
pub type Vector2f = Vector2<f32>;

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Vector2<T: Num + Copy> {
    pub x: T,
    pub y: T
}
impl<T: Num + Copy> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2::<T> {x, y}
    }
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }
}

impl Vector2<i32> {
    pub const ZERO: Vector2<i32> = Vector2::<i32> { x: 0, y: 0 };
    pub const UP: Vector2<i32> = Vector2::<i32> { x: 0, y: 1 };
    pub const DOWN: Vector2<i32> = Vector2::<i32> { x: 0, y: -1 };
    pub const LEFT: Vector2<i32> = Vector2::<i32> { x: -1, y: 0 };
    pub const RIGHT: Vector2<i32> = Vector2::<i32> { x: 1, y: 0 };
    pub fn manhattan(&self, other: Vector2<i32>) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    pub fn as_f32(&self) -> Vector2<f32> {
        Vector2::<f32>::new(self.x as f32, self.y as f32)
    }
    pub fn len(&self) -> f32 {
        self.as_f32().len()
    }
    pub fn angle(&self, other: &Self) -> f32 {
        self.as_f32().angle(&other.as_f32())
    }
    pub fn signed_angle(&self, other: &Self) -> f32 {
        self.as_f32().signed_angle(&other.as_f32())
    }
    pub fn clamped(&self) -> Self {
        let x = if self.x != 0 {
            self.x / self.x.abs()
        } else {
            0
        };
        let y = if self.y != 0 {
            self.y / self.y.abs()
        } else {
            0
        };
        Vector2i { x, y }
    }
}

impl Vector2<f32> {
    pub const UP: Vector2<f32> = Vector2::<f32> { x: 0., y: 1. };
    pub const DOWN: Vector2<f32> = Vector2::<f32> { x: 0., y: -1. };
    pub const LEFT: Vector2<f32> = Vector2::<f32> { x: -1., y: 0. };
    pub const RIGHT: Vector2<f32> = Vector2::<f32> { x: 1., y: 0. };
    pub const ZERO: Vector2<f32> = Vector2::<f32> { x: 0., y: 0.};

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn angle(&self, other: &Self) -> f32 {
        (self.dot(other) / (self.len() * other.len())).acos()
    }
    pub fn signed_angle(&self, other: &Self) -> f32 {
        other.y.atan2(other.x) - self.y.atan2(self.x)
    }
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Vector2f::new(
            lerp(self.x, other.x, t),
            lerp(self.y, other.y, t)
        )
    }
    pub fn normalized(&self) -> Self {
        let m = self.len();
        if m == 0. { return Self::ZERO };
        Vector2f::new(
            self.x / m,
            self.y / m
        )
    }
}

impl<T: Num + Copy> Add for Vector2<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        return Vector2::<T>::new(self.x + other.x, self.y + other.y);
    }
}

impl<T: Num + Copy> AddAssign for Vector2<T> {
    fn add_assign(&mut self, other: Self) {
        *self = Self{x: self.x + other.x, y: self.y + other.y};
    }
}

impl<T: Num + Copy> Sub for Vector2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        return Vector2::<T>::new(self.x - other.x, self.y - other.y)
    }
}

impl<T: Num + Copy> SubAssign for Vector2<T> {
    fn sub_assign(&mut self, other: Self) {
        *self = Self{x: self.x - other.x, y: self.y - other.y};
    }
}

impl<T: Num + Copy> Div<T> for Vector2<T> {
    type Output = Self;

    fn div(self, other: T) -> Self {
        return Vector2::<T>::new(self.x / other, self.y / other)
    }
}

impl<T: Num + Copy> Mul<T> for Vector2<T> {
    type Output = Self;

    fn mul(self, other: T) -> Self {
        return Vector2::<T>::new(self.x * other, self.y * other)
    }
}

// generic reverse multiplication didn't work due to orphan rules
impl Mul<Vector2<f32>> for f32 {
    type Output = Vector2<f32>;

    fn mul(self, other: Vector2<f32>) -> Vector2<f32> {
        return Vector2::<f32>::new(other.x * self, other.y * self)
    }
}
impl Mul<Vector2<i32>> for i32 {
    type Output = Vector2<i32>;

    fn mul(self, other: Vector2<i32>) -> Vector2<i32> {
        return Vector2::<i32>::new(other.x * self, other.y * self)
    }
}

pub const ORTHO_DIRECTIONS: [Vector2i; 4] = [
    Vector2i::UP, Vector2i::DOWN,
    Vector2i::LEFT, Vector2i::RIGHT
];

fn lerp(a: f32, b: f32, t:f32) -> f32 {
    a * (1.0 - t) + t * b
}