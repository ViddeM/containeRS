use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AccountsRsUserResponse {
    pub success: AccountsRsUserInfo,
}

#[derive(Deserialize, Clone)]
pub struct AccountsRsUserInfo {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}
