#[derive(Clone, Debug)]
pub struct DiffCalc<T> {
    pub old: T,
    pub new: T,
}

impl<T: Clone + PartialEq> DiffCalc<T> {
    pub fn new(current: T) -> Self {
        DiffCalc {
            old: current.clone(),
            new: current,
        }
    }

    pub fn changed(&self) -> bool {
        self.new != self.old
    }
}
