use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Redirect;
use axum_sessions::extractors::WritableSession;
use diesel::prelude::*;

use crate::config::AppState;
use crate::db::schema::*;
use crate::error::FrontendError;
use crate::utils::auth::frontend::Auth;

use super::{ManageFlashMessage, ACCOUNT_MANAGE_FLASH};

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    maybe_author: Option<Auth>,
    mut session: WritableSession,
) -> Result<Redirect, FrontendError> {
    let Some(Auth(author)) = maybe_author else {
        return Ok(Redirect::to("/account/manage"));
    };

    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        let token_author_id = author_tokens::table
            .select(author_tokens::author_id)
            .filter(author_tokens::id.eq(id))
            .first::<i64>(conn)
            .optional()?;

        match token_author_id {
            Some(token_author_id) if token_author_id == author.id => {
                diesel::delete(
                    author_tokens::table
                        .filter(author_tokens::id.eq(id))
                        .filter(author_tokens::author_id.eq(author.id)),
                )
                .execute(conn)?;

                let message = String::from("the token has successfully been revoked.");
                let flash_message = ManageFlashMessage::TokenRevocationSuccess { message };
                session.insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
                Ok(Redirect::to("/account/manage"))
            }
            Some(_) | None => {
                let message = String::from("invalid token to revoke.");
                let flash_message = ManageFlashMessage::TokenRevocationError { message };
                session.insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
                Ok(Redirect::to("/account/manage"))
            }
        }
    });

    transaction.await
}
