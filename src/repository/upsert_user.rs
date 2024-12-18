use anyhow::Context;

use crate::entities;

use super::Repository;

impl Repository {
    pub async fn upsert_user(&self, user: &entities::NewUser) -> anyhow::Result<entities::User> {
        const QUERY: &str = "INSERT INTO \"user\"
        (provider_name, provider_login, provider_email, school_email, avatar_url, github_user_tokens, first_name, last_name, school_group)
        VALUES ($1, $2, $3, $3, $4, $5, $6, $7, '')
        ON CONFLICT (provider_login) DO UPDATE
        SET (provider_name, provider_email, avatar_url, github_user_tokens)
        = ($1, $3, $4, $5)
        RETURNING *, uuid::varchar as uuid";

        let names = Names::split_name(&user.provider_name);

        sqlx::query_as::<_, entities::User>(QUERY)
            .bind(&user.provider_name)
            .bind(&user.provider_login)
            .bind(&user.provider_email)
            .bind(&user.avatar_url)
            .bind(&user.github_user_tokens)
            .bind(names.first_name)
            .bind(names.last_name)
            .fetch_one(&self.pool)
            .await
            .context(format!("[sql] upsert_user(user={user:?})"))
    }

    pub async fn update_user_profile(
        &self,
        user_id: &i32,
        user: &entities::UserProfileUpdate,
    ) -> anyhow::Result<entities::User> {
        const QUERY: &str = "UPDATE \"user\"
        SET first_name = $2, last_name = $3, school_group = $4, school_email = $5
        WHERE id = $1
        RETURNING *, uuid::varchar as uuid";

        sqlx::query_as::<_, entities::User>(QUERY)
            .bind(user_id)
            .bind(&user.firstname)
            .bind(&user.lastname)
            .bind(&user.school_group)
            .bind(&user.school_email)
            .fetch_one(&self.pool)
            .await
            .context(format!(
                "[sql] update_user_profile(user_id={user_id:?}, user={user:?})"
            ))
    }
}

#[derive(PartialEq, Debug)]
struct Names<'a> {
    first_name: &'a str,
    last_name: &'a str,
}

impl<'a> Names<'a> {
    const fn new(first_name: &'a str, last_name: &'a str) -> Self {
        Self {
            first_name,
            last_name,
        }
    }

    fn split_name(provider_name: &'a str) -> Self {
        provider_name
            .split_once(' ')
            .map_or_else(|| Names::new(provider_name, ""), |s| Names::new(s.0, s.1))
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::upsert_user::Names;
    use pretty_assertions::assert_eq;

    #[test]
    fn names_from_empty() {
        let result = Names::split_name("");
        assert_eq!(result, Names::new("", ""));
    }

    #[test]
    fn names_from_one_word() {
        let result = Names::split_name("Tom");
        assert_eq!(result, Names::new("Tom", ""));
    }

    #[test]
    fn names_from_two_words() {
        let result = Names::split_name("Tom Riddle");
        assert_eq!(result, Names::new("Tom", "Riddle"));
    }

    #[test]
    fn names_from_three_words() {
        let result = Names::split_name("Tom Marvolo Riddle");
        assert_eq!(result, Names::new("Tom", "Marvolo Riddle"));
    }
}
