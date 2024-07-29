#[derive(Default)]
pub struct Ready<T>(Option<T>);

impl<T> Ready<T> {
    pub fn new() -> Self {
        Ready(None)
    }

    pub fn set(&mut self, t: T) {
        if self.0.is_none() {
            self.0 = Some(t);
        } else {
            panic!("Already set.")
        }
    }
}

impl<T> std::ops::Deref for Ready<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for Ready<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
