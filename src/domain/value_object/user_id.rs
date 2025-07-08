//domain/value_object/user_id.rs
// UserId タイプセーフID
// 2025/7/8

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}
