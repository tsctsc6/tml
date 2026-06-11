use crate::app_state::AppState;
use tml_application::console_usecase::init_admin;
use tml_application::console_usecase::reset_password;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Inquire error: {0}")]
    InquireError(#[from] inquire::InquireError),
    #[error("Init admin error: {0}")]
    InitAdminError(#[from] tml_application::console_usecase::init_admin::Error),
    #[error("Reset password error: {0}")]
    ResetPasswordError(#[from] tml_application::console_usecase::reset_password::Error),
}

pub async fn init_admin(username: &str, app_state: AppState) -> Result<(), Error> {
    let new_password = inquire::Password::new("Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;
    init_admin::handle(
        init_admin::Request {
            username,
            password: &new_password,
        },
        &app_state.password_hasher,
        &tml_infrastructure::console_usecase::init_admin::Repository::new(app_state.db),
    )
    .await?;
    Ok(())
}

pub async fn reset_password(username: &str, app_state: AppState) -> Result<(), Error> {
    let new_password = inquire::Password::new("Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()?;
    reset_password::handle(
        reset_password::Request {
            username,
            password: &new_password,
        },
        &app_state.password_hasher,
        &tml_infrastructure::console_usecase::reset_password::Repository::new(
            app_state.db,
            app_state.user_id_security_stamp_cache,
        ),
    )
    .await?;
    Ok(())
}
