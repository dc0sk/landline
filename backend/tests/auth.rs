//! HTTP-level integration tests for auth & RBAC (NFR-MAINT-02).
//!
//! Traces: TC-AUTH-01 (unauthenticated rejected), TC-AUTH-04 (Observer denied a
//! control action; 403), TC-AUTH-05 (logout invalidates session).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use landline_backend::app;
use landline_backend::auth::{hash_password, Role};
use landline_backend::config::{AuthConfig, Config, UserConfig};
use serde_json::{json, Value};
use tower::ServiceExt;

fn config_with(users: Vec<UserConfig>) -> Config {
    Config {
        auth: AuthConfig {
            access_ttl_secs: 900,
            refresh_ttl_secs: 3600,
            users,
        },
        ..Config::default()
    }
}

fn user(name: &str, role: Role, password: &str) -> UserConfig {
    UserConfig {
        name: name.to_owned(),
        role,
        password_hash: hash_password(password).unwrap(),
    }
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn get(uri: &str, bearer: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri(uri);
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    builder.body(Body::empty()).unwrap()
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn unauthenticated_protected_route_is_rejected() {
    // TC-AUTH-01
    let response = app(&config_with(vec![]))
        .oneshot(get("/api/me", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_bad_password_is_unauthorized() {
    let app = app(&config_with(vec![user("op", Role::Operator, "s3cret")]));
    let response = app
        .oneshot(post_json(
            "/auth/login",
            &json!({"name": "op", "password": "wrong"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_then_access_protected_route() {
    let app = app(&config_with(vec![user("op", Role::Operator, "s3cret")]));
    let login = app
        .clone()
        .oneshot(post_json(
            "/auth/login",
            &json!({"name": "op", "password": "s3cret"}),
        ))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let tokens = body_json(login).await;
    let access = tokens["access_token"].as_str().unwrap();

    let me = app.oneshot(get("/api/me", Some(access))).await.unwrap();
    assert_eq!(me.status(), StatusCode::OK);
    let who = body_json(me).await;
    assert_eq!(who["sub"], "op");
    assert_eq!(who["role"], "operator");
}

#[tokio::test]
async fn observer_denied_operator_action() {
    // TC-AUTH-04: Observer attempts a control action -> 403.
    let app = app(&config_with(vec![user("obs", Role::Observer, "pw")]));
    let login = app
        .clone()
        .oneshot(post_json(
            "/auth/login",
            &json!({"name": "obs", "password": "pw"}),
        ))
        .await
        .unwrap();
    let access = body_json(login).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    let response = app
        .oneshot(get("/api/operator-ping", Some(&access)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn operator_allowed_operator_action() {
    let app = app(&config_with(vec![user("op", Role::Operator, "pw")]));
    let login = app
        .clone()
        .oneshot(post_json(
            "/auth/login",
            &json!({"name": "op", "password": "pw"}),
        ))
        .await
        .unwrap();
    let access = body_json(login).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    let response = app
        .oneshot(get("/api/operator-ping", Some(&access)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn logout_invalidates_session() {
    // TC-AUTH-05: after logout, the access token is rejected.
    let app = app(&config_with(vec![user("op", Role::Operator, "pw")]));
    let login = app
        .clone()
        .oneshot(post_json(
            "/auth/login",
            &json!({"name": "op", "password": "pw"}),
        ))
        .await
        .unwrap();
    let tokens = body_json(login).await;
    let access = tokens["access_token"].as_str().unwrap().to_owned();
    let refresh = tokens["refresh_token"].as_str().unwrap().to_owned();

    let logout = app
        .clone()
        .oneshot(post_json_auth(
            "/auth/logout",
            &access,
            &json!({"refresh_token": refresh}),
        ))
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);

    let after = app.oneshot(get("/api/me", Some(&access))).await.unwrap();
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
}

fn post_json_auth(uri: &str, bearer: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {bearer}"))
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}
