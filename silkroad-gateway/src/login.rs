use sqlx::PgPool;

#[derive(sqlx::FromRow, Clone)]
struct LoginDbResult {
    id: i32,
    passcode: Option<String>,
}

#[derive(sqlx::FromRow, Clone)]
struct IdResult(i32);

pub(crate) enum LoginResult {
    Success(i32),
    MissingPasscode,
    InvalidCredentials,
    Blocked,
}

pub(crate) struct LoginProvider {
    pool: PgPool,
}

impl LoginProvider {
    pub(crate) fn new(pool: PgPool) -> Self {
        LoginProvider { pool }
    }

    pub async fn try_login(&self, username: &str, password: &str) -> LoginResult {
        let result: Option<LoginDbResult> =
            sqlx::query_as("SELECT id, passcode FROM users WHERE username = $1 and password = $2")
                .bind(username)
                .bind(password)
                .fetch_optional(&self.pool)
                .await
                .unwrap();

        match result {
            Some(result) => {
                if result.passcode.is_some() {
                    LoginResult::MissingPasscode
                } else {
                    LoginResult::Success(result.id)
                }
            },
            None => LoginResult::InvalidCredentials,
        }
    }

    pub async fn try_login_passcode(&self, username: &str, password: &str, passcode: &str) -> LoginResult {
        let result: Option<IdResult> =
            sqlx::query_as("SELECT id FROM users WHERE username = $1 and password = $2 and passcode = $3")
                .bind(username)
                .bind(password)
                .bind(passcode)
                .fetch_optional(&self.pool)
                .await
                .unwrap();

        match result {
            Some(IdResult(id)) => LoginResult::Success(id),
            None => LoginResult::InvalidCredentials,
        }
    }
}
