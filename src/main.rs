#[macro_use]
extern crate rocket;

mod auth;
mod entity;
mod graphql;

use async_graphql::Request as GqlRequest;
use rocket::{Request, State};

use async_graphql::http::graphiql_source;
use rocket::response::content::RawHtml;

use auth::AuthUser;
use rocket::request::{FromRequest, Outcome};

use async_graphql::Schema;
use async_graphql_rocket::{GraphQLRequest, GraphQLResponse};
use dotenvy::dotenv;
use sea_orm::{Database, DatabaseConnection};

use graphql::schema::{AppSchema, MutationRoot, QueryRoot};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(header) = req.headers().get_one("Authorization") {
            if header.starts_with("Bearer ") {
                let token = &header[7..];

                if let Ok(secret) = std::env::var("JWT_SECRET") {
                    if let Some(uid) = crate::auth::validate_jwt(token, &secret) {
                        return Outcome::Success(AuthUser(Some(uid)));
                    }
                }
            }
        }

        Outcome::Success(AuthUser(None))
    }
}

#[post("/graphql", data = "<request>")]
async fn graphql_handler(
    schema: &State<AppSchema>,
    request: GraphQLRequest,
    user: AuthUser,
    db: &State<DatabaseConnection>,
) -> GraphQLResponse {
    // Force the conversion target type
    let mut req: GqlRequest = request.0;

    req = req.data(db.inner().clone());
    if let Some(uid) = user.0 {
        req = req.data(uid);
    }

    schema.execute(req).await.into()
}

#[get("/graphiql")]
fn graphiql_ui() -> RawHtml<String> {
    RawHtml(graphiql_source("/graphql", None))
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    // Connect to PostgreSQL
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let db = Database::connect(&db_url).await.expect("DB connect failed");

    // Build GraphQL Schema
    let schema = Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(db.clone()) // base DB for resolvers
        .finish();

    rocket::build()
        .manage(schema)
        .manage(db)
        .mount("/", routes![graphql_handler, graphiql_ui])
}
