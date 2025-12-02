use async_graphql::InputObject;

#[derive(InputObject)]
pub struct RegisterInput {
    pub username: String,
    pub password: String,
}

#[derive(InputObject)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}
