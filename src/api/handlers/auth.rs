use axum::{Json, extract::State};
use crate::config::AuthConfig;
use crate::types::{ApiResponse, LoginRequest, LoginResponse, UserInfo};
use crate::error::Error;

/// 认证状态
#[derive(Clone)]
pub struct AuthState {
    pub auth_config: AuthConfig,
}

impl AuthState {
    pub fn new(auth_config: AuthConfig) -> Self {
        Self {
            auth_config,
        }
    }
}

/// 登录处理
pub async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, Error> {
    // 验证用户名和密码
    if req.username == state.auth_config.username && req.password == state.auth_config.password {
        let user_info = UserInfo {
            username: state.auth_config.username.clone(),
            role: state.auth_config.role.clone(),
        };

        let response = LoginResponse {
            status: "success".to_string(),
            message: "登录成功".to_string(),
            user: user_info,
        };

        Ok(Json(ApiResponse::success(response)))
    } else {
        Err(Error::Validation("用户名或密码错误".to_string()))
    }
}
