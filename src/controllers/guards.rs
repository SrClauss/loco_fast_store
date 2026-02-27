use crate::models::users::Model as UserModel;
use loco_rs::{Error, Result};

/// Validate that user has admin access.
pub async fn ensure_admin(user: &UserModel) -> Result<()> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(Error::string("admin access required"))
    }
}

/// Validate warehouse access (admin or warehouse user).
pub async fn ensure_warehouse(user: &UserModel) -> Result<()> {
    if user.is_admin() || user.is_warehouse_user() {
        Ok(())
    } else {
        Err(Error::string("warehouse access required"))
    }
}
