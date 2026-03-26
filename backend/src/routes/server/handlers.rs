use axum::{extract::State, response::IntoResponse, Json,extract::Path, response::sse::{Event, Sse}};
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, NetworkingConfig, StopContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::models::{HostConfig, PortBinding, EndpointSettings}; // Added EndpointSettings
use bollard::network::{CreateNetworkOptions, InspectNetworkOptions};
use futures_util::stream::TryStreamExt;
use std::collections::HashMap;
use std::{convert::Infallible, sync::Arc};
use std::pin::Pin;
use axum::response::sse::KeepAlive;
use bollard::container::LogsOptions;
use futures_util::{Stream, StreamExt};
use bollard::container::StatsOptions;

use crate::dbmodels::server::{startServerRequest, CreateServerRequest, Server};
use crate::state::AppState;

pub async fn handle_create_server(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateServerRequest>,
) -> impl IntoResponse {
    match create_server_instance(&state.docker, &state.pool, payload).await {
        Ok(_) => (axum::http::StatusCode::CREATED, "Server created successfully").into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Orchestrator Error: {}", e),
        )
            .into_response(),
    }
}


pub async fn handle_start_server(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<startServerRequest>
) -> impl IntoResponse {
    match start_server_instance(&state.docker, &payload.name).await {
        Ok(_) => (axum::http::StatusCode::CREATED, "Server started").into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Orchestrator Error: {}", e),
            )
        .into_response(),
    }
}


pub async fn handle_stop_server(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<startServerRequest>
) -> impl IntoResponse {
    match stop_server_instance(&state.docker, &payload.name).await {
        Ok(_) => (axum::http::StatusCode::CREATED, "Server stopped").into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Orchestrator Error: {}", e),
        )
            .into_response(),
    }
}

pub async fn create_server_instance(
    docker: &bollard::Docker,
    db_pool: &sqlx::PgPool,
    req: CreateServerRequest,
) -> anyhow::Result<()> {
    let network_name = "Servers";
    let image_name = "itzg/minecraft-server";

    // 1. NETWORK: Ensure the custom bridge exists
    // We use a specific type for InspectNetworkOptions to avoid inference errors
    if let Err(e) = docker.inspect_network(network_name, None::<InspectNetworkOptions<String>>).await {
        println!("Network not found ({}), attempting to create...", e);

        let create_network_options = CreateNetworkOptions {
            name: network_name,
            driver: "bridge",
            check_duplicate: true,
            ..Default::default()
        };

        docker.create_network(create_network_options).await?;
        println!("Network '{}' created successfully", network_name);
    }

    // 2. IMAGE: Pull logic (keeps the same)
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: image_name,
                tag: "latest",
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    // 3. PREPARE: Env Vars
    let mc_type = req.server_type.to_uppercase();
    let mc_version = &req.version;
    let online_mode = if req.online_mode.unwrap_or(true) { "TRUE" } else { "FALSE" };
    let memory = req.memory.unwrap_or_else(|| "1G".to_string());

    let env_vars = vec![
        "EULA=TRUE".to_string(),
        format!("VERSION={}", mc_version),
        format!("TYPE={}", mc_type),
        format!("ONLINE_MODE={}", online_mode),
        format!("MEMORY={}", memory),
    ];
    let env_refs: Vec<&str> = env_vars.iter().map(|s| s.as_str()).collect();

    // 4. NETWORKING CONFIG: Explicitly link to the "Servers" network
    let mut endpoints_config = HashMap::new();
    endpoints_config.insert(
        network_name.to_string(),
        EndpointSettings {
            ..Default::default()
        },
    );

    // 5. CONFIG: Defining the container with NetworkingConfig
    let mut endpoints_config = HashMap::new();
    endpoints_config.insert(
        network_name, // No .to_string() here!
        EndpointSettings {
            ..Default::default()
        },
    );
    let project_name = "Beacon-live";

    // 2. Prepare the labels
    let mut labels = HashMap::new();
    labels.insert("com.docker.compose.project", project_name);
    labels.insert("com.docker.compose.service", &req.name); // Optional: names the "service" in the UI

    // 3. Update your Config
    // 4. CONFIG: Define the container
    let config = Config {
        image: Some("itzg/minecraft-server:latest"),
        env: Some(env_refs),
        labels: Some(labels),
        host_config: Some(HostConfig {
            network_mode: Some(network_name.to_string()),
            // ADD THIS: Explicitly set DNS servers
            dns: Some(vec![
                "8.8.8.8".to_string(), // Google
                "1.1.1.1".to_string()  // Cloudflare
            ]),
            port_bindings: Some(HashMap::from([(
                "25565/tcp".to_string(),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some("25565".to_string()),
                }]),
            )])),
            ..Default::default()
        }),
        networking_config: Some(NetworkingConfig {
            endpoints_config,
        }),
        ..Default::default()
    };

    // 6. EXECUTE: Create and Start
    let container = docker
        .create_container(
            Some(CreateContainerOptions {
                name: req.name.clone(),
                ..Default::default()
            }),
            config,
        )
        .await?;

    docker
        .start_container(&req.name, None::<StartContainerOptions<String>>)
        .await?;

    // 7. PERSIST: Save to Postgres
    sqlx::query(
        "INSERT INTO servers (name, container_id, version, status, server_type) VALUES ($1, $2, $3, $4, $5)",
    )
        .bind(&req.name)
        .bind(&container.id)
        .bind(mc_version)
        .bind("running")
        .bind(&req.server_type)
        .execute(db_pool)
        .await?;

    Ok(())
}


// Starts Existing Server/Containers
pub async fn start_server_instance(
    docker: &bollard::Docker,
    container_name: &str
) -> anyhow::Result<()> {


    docker
        .start_container(
            container_name,
            None::<StartContainerOptions<String>>
        )
        .await?;

    println!("Container '{}' started successfully.", container_name);
    Ok(())
}


pub async fn stop_server_instance(
    docker: &bollard::Docker,
    container_name: &str
) -> anyhow::Result<()> {


    docker
        .stop_container(
            container_name,
            None::<StopContainerOptions>
        )
        .await?;

    println!("Container '{}' started successfully.", container_name);
    Ok(())
}


pub async fn handle_get_servers(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 1. Fetch metadata from Postgres
    let mut servers = match sqlx::query_as::<_, Server>("SELECT * FROM servers")
        .fetch_all(&state.pool)
        .await
    {
        Ok(s) => s,
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 2. Hydrate with Docker Stats
    for server in servers.iter_mut() {
        // Fetch container info to get the real status and address
        if let Ok(inspect) = state.docker.inspect_container(&server.container_id, None).await {
            server.status = inspect.state.and_then(|s| s.status).map(|s| format!("{:?}", s)).unwrap_or("unknown".into());

            // Map the internal 25565 to the host address (hub.beacon.local)
            server.address = Some(format!("{}.beacon.local", server.name.to_lowercase()));

            // Get a single snapshot of stats for CPU usage
            let mut stats_stream = state.docker.stats(
                &server.container_id,
                Some(StatsOptions {
                    stream: false,
                    one_shot: true, // Add this field
                })
            );
            if let Some(Ok(stats)) = stats_stream.next().await {
                // Simplified CPU calculation
                let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64 - stats.precpu_stats.cpu_usage.total_usage as f64;
                let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64 - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

                if system_delta > 0.0 {
                    let cpu_pct = (cpu_delta / system_delta) * stats.cpu_stats.online_cpus.unwrap_or(1) as f64 * 100.0;
                    server.cpu_usage = Some(cpu_pct.round());
                }
            }
        }
    }

    (axum::http::StatusCode::OK, Json(servers)).into_response()
}

pub async fn handle_server_logs(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Sse<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>> { // Use Pin<Box<...>>

    let options = LogsOptions::<String> {
        follow: true,
        stdout: true,
        stderr: true,
        tail: "100".into(),
        ..Default::default()
    };

    let log_stream = state.docker.logs(&name, Some(options));

    // 1. Create the event stream as before
    let event_stream = log_stream.map(|result| {
        match result {
            Ok(output) => Ok(Event::default().data(format!("{}", output))),
            Err(e) => Ok(Event::default().data(format!("Error: {}", e))),
        }
    });

    // 2. Explicitly define the type for the boxed stream to force the coercion
    let stream: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> = Box::pin(event_stream);

    // 3. Return the Sse
    Sse::new(stream).keep_alive(KeepAlive::default())
}