pub trait GroupDirectory: Send + Sync {
    fn groups_of(&self, targets: &[String]) -> Vec<String>;
}
