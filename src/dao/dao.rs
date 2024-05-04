// https://doc.rust-lang.org/std/option/
// Borrow immutably - Read only
// Borrow mutably - mutate (Add smth to the data)
// Take ownership - consume (Receive then drop) - Rare cases should be used sparsely due to higher overhead than other options in both mental context and performance - In general, if a data is being borrowed immutably then never being used again, then it is logically, truthfully equivalent to taking ownership.
pub trait Dao<T> {
    fn get_all(&self) -> Vec<T>;
    fn get_by_id(&self, id: i64) -> Option<T>;
    fn save(&self, data: T) -> Result<T, E>;
    fn delete(&self, id: i64) -> Result<T, E>;
    fn update(&self, id: i64, data: T) -> Result<T, E>;
}
