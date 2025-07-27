use num_traits::Num;

use crate::vectors::vector2::Vector2;

#[derive(Clone, Copy, Debug, Default)]
pub struct Aabb<T: Copy + Num> {
    pub a: Vector2<T>,
    pub b: Vector2<T>,
}
impl<T: Copy + Ord + Num> Aabb<T> {
    pub fn new(a: Vector2<T>, b: Vector2<T>) -> Self {
        Self {
            a: Vector2::<T>::new(a.x.min(b.x), a.y.min(b.y)),
            b: Vector2::<T>::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }
    pub fn intersects(&self, other: &Self) -> bool {
        self.a.x < other.b.x && self.b.x > other.a.x && self.a.y < other.b.y && self.b.y > other.a.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectors::Vector2f;

    #[test]
    fn intersects_true() {
        let a = Aabb::new(Vector2f::new(5., 5.), Vector2f::new(10., 10.));
        let b = Aabb::new(Vector2f::new(8., 7.), Vector2f::new(20., 30.));
        assert!(a.intersects(&b));
    }
    #[test]
    fn intersects_false() {
        let a = Aabb::new(Vector2f::new(5., 5.), Vector2f::new(10., 10.));
        let b = Aabb::new(Vector2f::new(12., 7.), Vector2f::new(20., 30.));
        assert!(!a.intersects(&b));
    }
    #[test]
    fn test_coord_swap() {
        let a = Aabb::new(Vector2f::new(10., 11.), Vector2f::new(5., 6.));
        assert!(a.a.x == 5.);
        assert!(a.a.y == 6.);
        assert!(a.b.x == 10.);
        assert!(a.b.y == 11.);
    }
}
