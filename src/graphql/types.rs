use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct UserGQL {
    pub id: i32,
    pub username: String,
}
