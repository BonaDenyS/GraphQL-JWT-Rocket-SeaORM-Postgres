use crate::auth::*;
use crate::entity::prelude::User as UserEntity;
use crate::graphql::{input::*, types::UserGQL};
use async_graphql::{Context, Object, Schema};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn me(&self, ctx: &Context<'_>) -> Option<UserGQL> {
        let user_id = ctx.data_opt::<i32>()?;

        let db = ctx.data::<DatabaseConnection>().unwrap();

        let user = UserEntity::find_by_id(*user_id).one(db).await.ok()??;

        Some(UserGQL {
            id: user.id,
            username: user.username,
        })
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn register(
        &self,
        ctx: &Context<'_>,
        input: RegisterInput,
    ) -> async_graphql::Result<UserGQL> {
        let db = ctx.data::<DatabaseConnection>()?;
        println!("Register called!");

        let active = crate::entity::user::ActiveModel {
            username: sea_orm::ActiveValue::set(input.username.clone()),
            password: sea_orm::ActiveValue::set(input.password.clone()),
            ..Default::default()
        };

        let res = UserEntity::insert(active).exec(db).await?;

        let model = UserEntity::find_by_id(res.last_insert_id)
            .one(db)
            .await?
            .expect("Inserted row missing");

        Ok(UserGQL {
            id: model.id,
            username: model.username,
        })
    }

    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> async_graphql::Result<String> {
        let db = ctx.data::<DatabaseConnection>()?;
        let secret = std::env::var("JWT_SECRET")?;
        let filter = sea_orm::Condition::all()
            .add(crate::entity::user::Column::Username.eq(input.username.clone()));

        let user = UserEntity::find()
            .filter(filter)
            .one(db)
            .await?
            .ok_or("User not found")?;

        if user.password != input.password {
            return Err("Invalid credentials".into());
        }

        let token = create_jwt(user.id, &secret);
        Ok(token)
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;
