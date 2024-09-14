pub trait Game {
    fn setup(&mut self, context: &mut super::Context);
    fn update(&mut self, context: &mut super::Context);
    fn resize(&mut self, _context: &mut super::Context) {}
    fn resume(&mut self, _context: &mut super::Context) {}
}
