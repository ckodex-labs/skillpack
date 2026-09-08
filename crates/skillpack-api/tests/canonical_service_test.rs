use skillpack_proto::proto::{
    CheckBoundaryRequest, GetStatusRequest, MigrateAllRequest, SyncAgentsRequest,
    canonical_store_service_client::CanonicalStoreServiceClient,
};

#[tokio::test]
#[ignore = "requires running skillpack-server on 127.0.0.1:50052"]
async fn test_canonical_service_get_status() {
    let mut client = CanonicalStoreServiceClient::connect("http://127.0.0.1:50052")
        .await
        .expect("connect to server; ensure skillpack-server is running on port 50052");

    let request = tonic::Request::new(GetStatusRequest {});
    let response = client.get_status(request).await.expect("get_status call");
    let resp = response.into_inner();
    assert!(resp.is_healthy);
    println!(
        "Status: root={} skills={} healthy={}",
        resp.canonical_root, resp.total_skills, resp.is_healthy
    );
}

#[tokio::test]
#[ignore = "requires running skillpack-server on 127.0.0.1:50052"]
async fn test_canonical_service_check_boundary_safe() {
    let mut client = CanonicalStoreServiceClient::connect("http://127.0.0.1:50052")
        .await
        .expect("connect to server");

    let request = tonic::Request::new(CheckBoundaryRequest {
        target_path: "~/Skills/shared/safe-skill".to_string(),
        skill_name: Some("safe-skill".to_string()),
    });
    let response = client
        .check_boundary(request)
        .await
        .expect("check_boundary call");
    let resp = response.into_inner();
    assert!(resp.is_safe);
}

#[tokio::test]
#[ignore = "requires running skillpack-server on 127.0.0.1:50052"]
async fn test_canonical_service_check_boundary_violation() {
    let mut client = CanonicalStoreServiceClient::connect("http://127.0.0.1:50052")
        .await
        .expect("connect to server");

    let request = tonic::Request::new(CheckBoundaryRequest {
        target_path: "~/Skills/shared/Thales-Internal".to_string(),
        skill_name: Some("Thales-Internal".to_string()),
    });
    let response = client
        .check_boundary(request)
        .await
        .expect("check_boundary call");
    let resp = response.into_inner();
    assert!(!resp.is_safe);
}

#[tokio::test]
#[ignore = "requires running skillpack-server on 127.0.0.1:50052"]
async fn test_canonical_service_migrate_all() {
    let mut client = CanonicalStoreServiceClient::connect("http://127.0.0.1:50052")
        .await
        .expect("connect to server");

    let canonical_root = std::env::var("SKILLS_ROOT").unwrap_or_else(|_| {
        dirs::home_dir()
            .unwrap()
            .join("Skills/shared")
            .to_string_lossy()
            .to_string()
    });
    let request = tonic::Request::new(MigrateAllRequest { canonical_root });
    let response = client.migrate_all(request).await.expect("migrate_all call");
    let resp = response.into_inner();
    println!(
        "Migrate result: success={} msg={}",
        resp.success, resp.message
    );
}

#[tokio::test]
#[ignore = "requires running skillpack-server on 127.0.0.1:50052"]
async fn test_canonical_service_sync_agents() {
    let mut client = CanonicalStoreServiceClient::connect("http://127.0.0.1:50052")
        .await
        .expect("connect to server");

    let request = tonic::Request::new(SyncAgentsRequest {
        dry_run: true,
        no_index: false,
        only_agent: Some("".to_string()),
        namespace: None,
    });
    let response = client.sync_agents(request).await.expect("sync_agents call");
    let resp = response.into_inner();
    println!(
        "Sync result: success={} skills={} msg={}",
        resp.success, resp.skills_processed, resp.message
    );
}
