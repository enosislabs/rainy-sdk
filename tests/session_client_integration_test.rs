use rainy_sdk::{RainySessionClient, SessionConfig};

fn maybe_server() -> Option<mockito::ServerGuard> {
    match std::panic::catch_unwind(mockito::Server::new) {
        Ok(server) => Some(server),
        Err(_) => {
            eprintln!("Skipping session_client_integration_test: mock server unavailable in this environment");
            None
        }
    }
}

#[tokio::test]
async fn session_login_sets_access_token_from_v3_alias_response() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let _mock = server
        .mock("POST", "/api/v1/auth/login")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
              "success": true,
              "data": {
                "accessToken": "acc_123",
                "refreshToken": "ref_123",
                "user": { "id": "u1", "email": "test@example.com", "role": "admin" }
              },
              "accessToken": "acc_123",
              "refreshToken": "ref_123",
              "user": { "id": "u1", "email": "test@example.com", "role": "admin" }
            }"#,
        )
        .create();

    let mut client = RainySessionClient::with_config(
        SessionConfig::new().with_base_url(server.url()).with_timeout(5),
    )
    .expect("client");

    let login = client
        .login("test@example.com", "password123")
        .await
        .expect("login");

    assert_eq!(login.access_token, "acc_123");
    assert_eq!(client.access_token(), Some("acc_123"));
}

#[tokio::test]
async fn session_org_and_usage_calls_send_bearer_token() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let _org = server
        .mock("GET", "/api/v1/orgs/me")
        .match_header("authorization", "Bearer acc_123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
              "success": true,
              "data": {
                "id": "org_1",
                "name": "Rainy Org",
                "planId": "payg",
                "region": "us",
                "createdAt": "2026-02-26T00:00:00.000Z",
                "credits": "10.00000"
              },
              "id": "org_1",
              "name": "Rainy Org",
              "planId": "payg",
              "region": "us",
              "createdAt": "2026-02-26T00:00:00.000Z",
              "credits": "10.00000"
            }"#,
        )
        .create();

    let _credits = server
        .mock("GET", "/api/v1/usage/credits")
        .match_header("authorization", "Bearer acc_123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
              "success": true,
              "data": { "balance": 10, "currency": "credits" },
              "balance": 10,
              "currency": "credits",
              "source": "database"
            }"#,
        )
        .create();

    let mut client = RainySessionClient::with_config(
        SessionConfig::new().with_base_url(server.url()).with_timeout(5),
    )
    .expect("client");
    client.set_access_token("acc_123");

    let org = client.org_me().await.expect("org");
    let credits = client.usage_credits().await.expect("credits");

    assert_eq!(org.plan_id, "payg");
    assert_eq!(credits.currency, "credits");
    assert_eq!(credits.balance, 10.0);
}

#[tokio::test]
async fn session_usage_stats_parses_v3_alias_fields() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let _stats = server
        .mock("GET", "/api/v1/usage/stats?days=7")
        .match_header("authorization", "Bearer acc_123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
              "success": true,
              "data": {
                "byProvider": {},
                "daily": { "used": 1, "limit": 100, "remaining": 99 }
              },
              "periodDays": 7,
              "totalRequests": 12,
              "totalCreditsDeducted": 0.42,
              "statsByProvider": { "openrouter": { "requests": 12, "creditsDeducted": 0.42 } },
              "logs": []
            }"#,
        )
        .create();

    let mut client = RainySessionClient::with_config(
        SessionConfig::new().with_base_url(server.url()).with_timeout(5),
    )
    .expect("client");
    client.set_access_token("acc_123");

    let stats = client.usage_stats(Some(7)).await.expect("stats");
    assert_eq!(stats.period_days, 7);
    assert_eq!(stats.total_requests, 12);
    assert!((stats.total_credits_deducted - 0.42).abs() < f64::EPSILON);
}
