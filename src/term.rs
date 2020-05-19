use console::Term;

pub trait Sized {
    fn size(&self) -> (u32, u32);
    fn width(&self) -> u32 {
        let (_, w) = self.sized;

        w
    }
    fn height(&self) -> u32 {
        let (h, _) = self.sized;

        h
    }
}

impl Sized for Term {
    fn size(&self) -> (u32, u32) {
        self.0.size()
    }
}
