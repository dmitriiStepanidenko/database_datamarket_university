use async_graphql::{Enum};
use fake::{Dummy};
use serde::{Deserialize, Serialize};


pub type Roles = Vec<Role>;

#[derive(Enum, Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Dummy)]
pub enum Role {
    User,
    Admin,
}
