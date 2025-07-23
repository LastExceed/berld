use protocol::nalgebra::Point3;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct Instance {
    pub filename: String,
    pub position: Point3<i64>
}