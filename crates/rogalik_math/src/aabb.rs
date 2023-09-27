use crate::vectors::Vector2F;

pub struct Aabb {
    pub a: Vector2F,
    pub b: Vector2F
}
impl Aabb {
    pub fn new(a: Vector2F, b: Vector2F) -> Self {
        Self { 
            a: Vector2F::new(a.x.min(b.x), a.y.min(b.y)),
            b: Vector2F::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }
    pub fn intersects(&self, other: &Self) -> bool {
        self.a.x < other.b.x &&
        self.b.x > other.a.x &&
        self.a.y < other.b.y &&
        self.b.y > other.a.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersects_true() {
        let a = Aabb::new(
            Vector2F::new(5., 5.),
            Vector2F::new(10., 10.)
        );
        let b = Aabb::new(
            Vector2F::new(8., 7.),
            Vector2F::new(20., 30.)
        );
        assert!(a.intersects(&b));
    }
    #[test]
    fn intersects_false() {
        let a = Aabb::new(
            Vector2F::new(5., 5.),
            Vector2F::new(10., 10.)
        );
        let b = Aabb::new(
            Vector2F::new(12., 7.),
            Vector2F::new(20., 30.)
        );
        assert!(!a.intersects(&b));
    }
    #[test]
    fn test_coord_swap() {
        let a = Aabb::new(
            Vector2F::new(10., 11.),
            Vector2F::new(5., 6.)
        );
        assert!(a.a.x == 5.);
        assert!(a.a.y == 6.);
        assert!(a.b.x == 10.);
        assert!(a.b.y == 11.);
    }
}