use crate::{
    datetime::DateTimeDerived, thing_derived::ThingDerived, thing_wrapper::ObjectWithThing,
};
use async_graphql::{InputObject, SimpleObject};
use chrono::{Duration, Utc};
use common::{
    ctx::Ctx,
    error::{ApiError, ApiResult, Error},
    mw_ctx::{Claims, JWT_KEY},
    role::{Role, Roles},
};
use db::Db;
use fake::faker::internet::en::{FreeEmail, Password};
use fake::Dummy;

use jsonwebtoken::{encode, EncodingKey, Header};

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use tower_cookies::Cookie;

pub const RESOURCE: &str = "User";

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct User {
    pub id: Option<ThingDerived>,
    pub email: String,
    #[graphql(skip)]
    pub password: Option<String>,
    pub roles: Roles,

    pub created_at: Option<DateTimeDerived>,
    pub updated_at: Option<DateTimeDerived>,
}

impl ObjectWithThing for User {
    fn thing(&self, ctx: &dyn Ctx) -> ApiResult<Thing> {
        self.id
            .as_ref()
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::Generic {
                    description: "Can't get thing. Get none instead".to_string(),
                },
            })?
            .thing(ctx)
    }
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn fetch_user_by_email<'a>(
        email: &'a str,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User>;

    async fn update_user<'a>(
        user: User,
        id: ThingDerived,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User>;

    async fn get_user_by_user_id<'a>(
        user_id: Thing,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User>;

    async fn create_user<'a>(
        email: String,
        hashed_pass: Option<String>,
        roles: Vec<Role>,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User>;
}

struct UserRepositoryImpl {
    //pub ctx: &'a dyn Ctx,
}

impl UserRepositoryImpl {
    pub async fn find_user_by_email<'a>(
        email: &String,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<Option<User>> {
        let user: Option<User> = db
            .query("SELECT * FROM User WHERE email = $email;")
            .bind(("email", email))
            .await
            .map_err(ApiError::from(ctx))?
            .take::<Option<User>>(0)
            .map_err(ApiError::from(ctx))?;
        Ok(user)
    }
}

#[async_trait::async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn fetch_user_by_email<'a>(
        email: &'a str,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User> {
        let user: User = db
            .query("SELECT * FROM User WHERE email = $email;")
            .bind(("email", email))
            .await
            .map_err(ApiError::from(ctx))?
            .take::<Option<User>>(0)
            .map_err(ApiError::from(ctx))?
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: "internal".to_string(),
                    id: email.to_string(),
                },
            })?;
        return Ok(user);
    }

    async fn update_user<'a>(
        user: User,
        id: ThingDerived,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User> {
        db.update((RESOURCE, id.id().to_string()))
            .content(user)
            .await
            .map_err(ApiError::from(ctx))?
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: "internal".to_string(),
                    id: id.to_string(),
                },
            })
    }

    async fn get_user_by_user_id<'a>(
        user_id: Thing,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User> {
        db.select((RESOURCE, user_id))
            .await
            .map_err(ApiError::from(ctx))?
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: "internal".to_string(),
                    id: ctx.user_id()?,
                },
            })
    }

    async fn create_user<'a>(
        email: String,
        hashed_pass: Option<String>,
        roles: Vec<Role>,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User> {
        db.create(RESOURCE)
            .content(User {
                id: None,
                email,
                password: hashed_pass,
                roles,
                created_at: None,
                updated_at: None,
            })
            .await
            .map_err(ApiError::from(ctx))
            .map(|v: Vec<User>| v.into_iter().next().expect("created user"))
    }
}

#[derive(Deserialize, InputObject, Clone, Dummy)]
pub struct CreateUserInput {
    #[dummy(faker = "FreeEmail()")]
    #[graphql(validator(email))]
    pub email: String,

    /// TODO: better validation
    #[dummy(faker = "Password((6..12))")]
    #[graphql(validator(min_length = 4), secret)]
    pub password: String,
}

pub struct UserUseCases {
    //pub db: &'a Db,
    //pub ctx: &'a dyn Ctx,
}

impl UserUseCases {
    async fn fetch_user_by_email<'a>(email: &str, db: &'a Db, ctx: &'a dyn Ctx) -> ApiResult<User> {
        UserRepositoryImpl::fetch_user_by_email(email, db, ctx).await
    }

    pub async fn update_user<'a>(
        user: User,
        id: ThingDerived,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User> {
        UserRepositoryImpl::update_user(user, id, db, ctx).await
    }

    pub async fn me<'a>(db: &'a Db, ctx: &'a dyn Ctx) -> ApiResult<User> {
        let user_id = ctx.user_id_thing()?;
        UserRepositoryImpl::get_user_by_user_id(user_id, db, ctx).await
    }

    pub async fn create_user<'a>(
        ct_input: CreateUserInput,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User> {
        //let hashed_pass = argon2rs::argon2i_simple(&ct_input.password, SALT.as_str());
        //let hashed_pass: String = hashed_pass.iter().fold(String::new(), |mut output, byte| {
        //    let _ = write!(output, "{byte:02x}");
        //    output
        //});
        let salt = SaltString::generate(&mut OsRng);
        let hashed_pass = Argon2::default()
            .hash_password(ct_input.password.as_bytes(), &salt)
            .map_err(|_| ApiError {
                req_id: ctx.req_id(),
                error: Error::LoginFail,
            })?
            .to_string();

        UserRepositoryImpl::create_user(
            ct_input.email,
            Some(hashed_pass.to_string()),
            vec![Role::User],
            db,
            ctx,
        )
        .await
    }

    pub async fn login<'a>(
        ct_input: CreateUserInput,
        key_enc: EncodingKey,
        db: &'a Db,
        ctx: &'a dyn Ctx,
    ) -> ApiResult<User> {
        let user: User = Self::fetch_user_by_email(&ct_input.email, db, ctx)
            .await
            .map_err(|_| ApiError {
                req_id: ctx.req_id(),
                error: Error::LoginFail,
            })?;

        match user.password {
            Some(ref pass) => {
                let parsed_hash = PasswordHash::new(pass).map_err(|_| ApiError {
                    req_id: ctx.req_id(),
                    error: Error::LoginFail,
                })?;
                if Argon2::default()
                    .verify_password(ct_input.password.as_bytes(), &parsed_hash)
                    .is_err()
                {
                    return Err(ApiError {
                        req_id: ctx.req_id(),
                        error: Error::LoginFail,
                    });
                }
            }
            None => {
                return Err(ApiError {
                    req_id: ctx.req_id(),
                    error: Error::LoginFail,
                });
            }
        }
        // NOTE: set to a reasonable number after testing
        // NOTE when testing: the default validation.leeway is 2min
        let exp = Utc::now() + Duration::minutes(120);
        let claims = Claims {
            exp: exp.timestamp() as usize,
            email: user.email.clone(),
            id: user
                .id
                .clone()
                .ok_or(ApiError {
                    req_id: ctx.req_id(),
                    error: Error::LoginFail,
                })?
                .to_string(),
            roles: user.roles.clone(),
        };
        let token_str =
            encode(&Header::default(), &claims, &key_enc).expect("JWT encode should work");

        ctx.cookies_add(
            Cookie::build(JWT_KEY, token_str)
                // if not set, the path defaults to the path from which it was called - prohibiting gql on root if login is on /api
                .path("/")
                .http_only(true)
                .finish(),
        )?;

        Ok(user)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use common::ctx::MockCtx;
    use db::set_test_db;
    use fake::{Fake, Faker};
    use rstest::*;

    #[fixture]
    fn ctx() -> MockCtx {
        let mut ctx = MockCtx::new();
        ctx.expect_req_id().return_const(uuid::Uuid::new_v4());
        ctx.expect_cookies_add().returning(|_| Ok(()));
        ctx
    }

    #[fixture]
    async fn tdb() -> Db {
        set_test_db().await
    }

    #[rstest]
    #[tokio::test]
    #[awt]
    async fn user_use_case_create_test(ctx: impl Ctx, #[future] tdb: Db) {
        let data: CreateUserInput = Faker.fake();
        let user = UserUseCases::create_user(data.clone(), &tdb, &ctx).await;
        assert!(user.is_ok());

        let user = UserUseCases::create_user(data, &tdb, &ctx).await;
        assert!(user.is_err());
    }

    #[rstest]
    #[tokio::test]
    #[awt]
    async fn user_repository_create_test(ctx: impl Ctx, #[future] tdb: Db) {
        let data: CreateUserInput = Faker.fake();
        let user = UserRepositoryImpl::create_user(
            data.email.clone(),
            Some(data.password.clone()),
            vec![Role::User],
            &tdb,
            &ctx,
        )
        .await;
        assert!(user.is_ok());

        let user = UserRepositoryImpl::create_user(
            data.email,
            Some(data.password),
            vec![Role::User],
            &tdb,
            &ctx,
        )
        .await;
        assert!(user.is_err());
    }

    pub async fn create_user(ctx: &impl Ctx, tdb: &Db) -> ApiResult<(User, CreateUserInput)> {
        let data: CreateUserInput = Faker.fake();
        let user = UserUseCases::create_user(data.clone(), tdb, ctx).await?;
        Ok((user, data))
    }

    #[rstest]
    #[tokio::test]
    #[awt]
    async fn user_usecase_fetch_by_email_test(ctx: impl Ctx, #[future] tdb: Db) {
        let (user, _) = create_user(&ctx, &tdb).await.unwrap();

        let fetched_user = UserUseCases::fetch_user_by_email(&user.email, &tdb, &ctx)
            .await
            .unwrap();
        assert_eq!(user.id, fetched_user.id);
    }

    #[rstest]
    #[tokio::test]
    #[awt]
    async fn login_user_test(ctx: impl Ctx, #[future] tdb: Db) {
        let (_user, mut user_ct) = create_user(&ctx, &tdb).await.unwrap();

        use jsonwebtoken::EncodingKey;
        let key_enc = EncodingKey::from_secret(b"dfopij2oij0ij2f");

        assert!(
            UserUseCases::login(user_ct.clone(), key_enc.clone(), &tdb, &ctx)
                .await
                .is_ok()
        );

        user_ct.password = "definetely_not_equal_to_original_password".to_string();
        assert!(UserUseCases::login(user_ct, key_enc, &tdb, &ctx)
            .await
            .is_err());
    }

    //#[fixture]
    //#[awt]
    //async fn user_repository(#[future] ctx: MockCtx) -> UserRepositoryImpl<'static> {
    //    UserRepositoryImpl { ctx: &ctx }
    //}

    pub mod utility {
        use super::*;

        pub async fn get_db() -> Db {
            set_test_db().await
        }

        // pub async fn create_user(
        //     email: &str,
        //     pass: &str,
        //     db: &Db,
        //     ctx: &dyn Ctx,
        // ) -> ApiResult<User> {
        //     let email = email.to_string();
        //     let password = pass.to_string();
        //     let ct_input = CreateUserInput { email, password };

        //     UserService { db: &db, ctx }.create_user(ct_input).await
        // }
    }
}
