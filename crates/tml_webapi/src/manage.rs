use crate::app_state::AppState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Inquire error: {0}")]
    InquireError(#[from] inquire::InquireError),
    #[error("Init admin error: {0}")]
    InitAdminError(#[from] tml_application::console_usecase::init_admin::Error),
    #[error("Reset password error: {0}")]
    ResetPasswordError(#[from] tml_application::console_usecase::reset_password::Error),
}

pub async fn init(username: &str, app_state: AppState) -> Result<(), Error> {
    let secret_password = inquire::Password::new("Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()
        .unwrap();
    Ok(())
}

pub async fn reset_password(username: &str, app_state: AppState) -> Result<(), Error> {
    let secret_password = inquire::Password::new("Password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .prompt()
        .unwrap();
    Ok(())
}
