//! MCP server integration for NMS Copilot.
//!
//! When running normally, the MCP server runs on a background thread with its
//! own tokio runtime, sharing the `GalaxyModel` with the REPL via
//! `Arc<RwLock<GalaxyModel>>`. In headless mode (`--headless`), only the MCP
//! server runs (no REPL).

pub mod config;
pub mod resources;
pub mod tools;

#[cfg(feature = "http")]
use std::io::Write;
#[cfg(feature = "http")]
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread;
#[cfg(feature = "http")]
use std::thread::JoinHandle;
use std::time::Duration;

use fabryk_mcp::{
    CompositeRegistry, DiscoverableRegistry, FabrykMcpServer, HealthTools, Notifier,
    ServerGuidance, ServiceHandle, ServiceState, ToolMeta, ToolRegistry,
};
use tokio::sync::RwLock;

use crate::coordinator::AppliedSaveUpdate;
use nms_graph::GalaxyModel;

use resources::{BASES_URI, GALAXY_STATS_URI, NmsResources, PLAYER_LOCATION_URI};
use tools::NmsTools;

/// MCP transport mode.
pub enum Transport {
    Stdio,
    #[cfg(feature = "http")]
    Http(SocketAddr),
}

/// Failure while starting or stopping an owned background MCP HTTP server.
///
/// Startup errors are returned before the readiness call succeeds. Errors
/// observed after readiness are written to stderr; explicit shutdown reports
/// worker panics.
///
/// # Examples
///
/// ```
/// use std::error::Error as _;
/// use nms_copilot::mcp::McpBackgroundError;
///
/// let error = McpBackgroundError::Runtime(std::io::Error::other("example"));
/// assert!(error.source().is_some());
/// ```
#[cfg(feature = "http")]
#[derive(Debug)]
pub enum McpBackgroundError {
    /// The worker operating-system thread could not be created.
    Spawn(std::io::Error),
    /// The worker could not construct its Tokio runtime.
    Runtime(std::io::Error),
    /// The requested listener address could not bind.
    Bind {
        /// Address the caller requested.
        address: SocketAddr,
        /// Error returned by the operating system.
        source: std::io::Error,
    },
    /// The listener bound, but its actual local address could not be read.
    LocalAddress {
        /// Address the caller requested.
        address: SocketAddr,
        /// Error returned while reading the actual bound address.
        source: std::io::Error,
    },
    /// The worker exited before sending its startup result.
    ReadinessClosed,
    /// The worker thread panicked while being joined.
    WorkerPanicked,
}

#[cfg(feature = "http")]
impl std::fmt::Display for McpBackgroundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(error) => write!(f, "could not spawn MCP worker: {error}"),
            Self::Runtime(error) => write!(f, "MCP runtime failed: {error}"),
            Self::Bind { address, source } => write!(f, "bind {address} failed: {source}"),
            Self::LocalAddress { address, source } => {
                write!(f, "could not read bound address for {address}: {source}")
            }
            Self::ReadinessClosed => write!(f, "MCP worker closed readiness before binding"),
            Self::WorkerPanicked => write!(f, "MCP worker panicked"),
        }
    }
}

#[cfg(feature = "http")]
impl std::error::Error for McpBackgroundError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn(error) => Some(error),
            Self::Runtime(error) => Some(error),
            Self::Bind { source, .. } | Self::LocalAddress { source, .. } => Some(source),
            Self::ReadinessClosed | Self::WorkerPanicked => None,
        }
    }
}

/// Owns an HTTP server worker and its stop signal.
///
/// Call [`Self::shutdown`] outside application model guards; it blocks until
/// active requests and SSE connections are cancelled and the worker stops.
/// Dropping the handle also requests stop and joins, but reports join errors
/// to stderr rather than returning them.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use nms_copilot::mcp::{spawn_mcp_background_with_updates, McpBackgroundError};
/// use nms_graph::GalaxyModel;
/// use tokio::sync::RwLock;
///
/// # fn example() -> Result<(), McpBackgroundError> {
/// let model = Arc::new(RwLock::new(GalaxyModel::new()));
/// let address = "127.0.0.1:0".parse().expect("socket address");
/// let (_sender, updates) = std::sync::mpsc::channel();
/// let handle = spawn_mcp_background_with_updates(model, address, Some(updates))?;
/// println!("listening at {}", handle.local_addr());
/// handle.shutdown()?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "http")]
#[must_use = "dropping the handle stops and joins the MCP worker; call shutdown to observe failures"]
pub struct McpBackgroundHandle {
    address: SocketAddr,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

#[cfg(feature = "http")]
impl std::fmt::Debug for McpBackgroundHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpBackgroundHandle")
            .field("address", &self.address)
            .field("running", &self.worker.is_some())
            .finish()
    }
}

#[cfg(feature = "http")]
impl McpBackgroundHandle {
    /// The actual local socket address selected by the listener.
    ///
    /// See the complete [`spawn_mcp_background_with_updates`] example for
    /// obtaining and using the actual address after binding.
    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }

    /// Cancel active connections (including SSE) and join the worker thread.
    ///
    /// # Errors
    /// Returns [`McpBackgroundError::WorkerPanicked`] if the worker panicked
    /// while joining. Startup failures are returned by
    /// [`spawn_mcp_background_with_updates`]; post-ready serve errors are
    /// written to stderr by the worker.
    ///
    /// # Examples
    ///
    /// See the complete [`spawn_mcp_background_with_updates`] example for
    /// starting the owner and explicitly shutting it down.
    pub fn shutdown(mut self) -> Result<(), McpBackgroundError> {
        self.stop_worker();
        self.join_worker()
    }

    fn stop_worker(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }

    fn join_worker(&mut self) -> Result<(), McpBackgroundError> {
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| McpBackgroundError::WorkerPanicked),
            None => Ok(()),
        }
    }
}

#[cfg(feature = "http")]
impl Drop for McpBackgroundHandle {
    fn drop(&mut self) {
        self.stop_worker();
        if let Err(error) = self.join_worker() {
            let _ = writeln!(
                std::io::stderr().lock(),
                "MCP background worker failed during drop: {error}"
            );
        }
    }
}

#[derive(Default)]
enum ServerControl {
    #[default]
    Unmanaged,
    #[cfg(feature = "http")]
    Owned {
        ready: Option<std::sync::mpsc::SyncSender<Result<SocketAddr, McpBackgroundError>>>,
        stop: tokio::sync::oneshot::Receiver<()>,
    },
}

#[cfg(feature = "http")]
impl ServerControl {
    fn signal_ready(&mut self, address: SocketAddr) {
        if let Self::Owned { ready, .. } = self
            && let Some(ready) = ready.take()
        {
            let _ = ready.send(Ok(address));
        }
    }

    fn signal_error(&mut self, error: McpBackgroundError) {
        if let Self::Owned { ready, .. } = self
            && let Some(ready) = ready.take()
        {
            let _ = ready.send(Err(error));
        }
    }
}

/// Start an owned HTTP observer for updates already applied by the application.
///
/// Shutdown cancels in-flight requests and SSE streams, drops queued observer
/// events, and joins the worker. The server drains the receiver moved into it;
/// other coordinator subscribers drain their own receivers. The standard
/// observer queue is unbounded, so each subscriber must keep its receiver drained.
/// Parser and filesystem work have no hard
/// deadline. This handle's shutdown is blocking and must run outside model
/// guards.
///
/// # Errors
/// Returns thread-spawn, runtime-construction, bind, local-address, readiness,
/// or worker-panic errors. I/O-backed variants retain their source. Errors
/// after readiness are reported to stderr by the worker.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use nms_copilot::mcp::{spawn_mcp_background_with_updates, McpBackgroundError};
/// use nms_graph::GalaxyModel;
/// use tokio::sync::RwLock;
/// # fn example() -> Result<(), McpBackgroundError> {
/// let model = Arc::new(RwLock::new(GalaxyModel::new()));
/// let address = "127.0.0.1:0".parse().expect("socket address");
/// let (_sender, updates) = std::sync::mpsc::channel();
/// let owner = spawn_mcp_background_with_updates(model, address, Some(updates))?;
/// println!("listening at {}", owner.local_addr());
/// owner.shutdown()?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "http")]
pub fn spawn_mcp_background_with_updates(
    model: Arc<RwLock<GalaxyModel>>,
    address: SocketAddr,
    updates: Option<Receiver<AppliedSaveUpdate>>,
) -> Result<McpBackgroundHandle, McpBackgroundError> {
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
    let service = ServiceHandle::new("mcp-http");
    let worker = thread::Builder::new()
        .name("nms-mcp-http".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = ready_tx.send(Err(McpBackgroundError::Runtime(error)));
                    return;
                }
            };
            runtime.block_on(async move {
                let control = ServerControl::Owned {
                    ready: Some(ready_tx),
                    stop: stop_rx,
                };
                if let Err(error) = run_mcp_server(
                    model,
                    Transport::Http(address),
                    None,
                    updates,
                    None,
                    control,
                    service,
                )
                .await
                {
                    let _ = writeln!(std::io::stderr().lock(), "MCP server error: {error}");
                }
            })
        })
        .map_err(McpBackgroundError::Spawn)?;

    match ready_rx.recv() {
        Ok(Ok(bound_address)) => Ok(McpBackgroundHandle {
            address: bound_address,
            stop: Some(stop_tx),
            worker: Some(worker),
        }),
        Ok(Err(error)) => {
            worker
                .join()
                .map_err(|_| McpBackgroundError::WorkerPanicked)?;
            Err(error)
        }
        Err(_) => {
            worker
                .join()
                .map_err(|_| McpBackgroundError::WorkerPanicked)?;
            Err(McpBackgroundError::ReadinessClosed)
        }
    }
}

/// Start the MCP server on a background thread with its own tokio runtime.
///
/// The server shares the given model with the REPL. When using HTTP transport,
/// the MCP server runs concurrently with the REPL. Stdio transport should only
/// be used in headless mode.
///
/// Starts a default-debounce watcher when `save_path` is `Some`.
///
/// This compatibility wrapper uses a 500ms debounce; use
/// [`spawn_mcp_background_with_watch_config`] to supply explicit settings.
///
/// For HTTP transport, blocks the caller until the server is listening
/// (TCP bind succeeded) so the REPL doesn't start before the MCP endpoint
/// is reachable. Uses fabryk-core's `ServiceHandle` for state tracking.
pub fn spawn_mcp_background(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    save_path: Option<std::path::PathBuf>,
) {
    spawn_mcp_background_with_watch_config(model, transport, default_watch_config(save_path));
}

/// Start the MCP server on a background thread with an explicit watcher setting.
///
/// `Some(config)` is forwarded unchanged to the save watcher and its owned
/// handle remains alive while the server runs. `None` disables file watching
/// while the server continues to use the supplied in-memory model.
pub fn spawn_mcp_background_with_watch_config(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    watch_config: Option<nms_watch::WatchConfig>,
) {
    let mcp_service = ServiceHandle::new("mcp-http");
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<Result<(), String>>(1);

    let service_for_thread = mcp_service.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            let watcher = start_watcher(watch_config);

            if let Err(e) = run_mcp_server(
                model,
                transport,
                watcher,
                None,
                Some(ready_tx),
                ServerControl::Unmanaged,
                service_for_thread,
            )
            .await
            {
                log::error!("MCP server error: {e}");
            }
        });
    });

    // Block until the server signals it is ready (or failed to bind)
    match ready_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            eprintln!("MCP server failed to start: {e}");
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("MCP server thread exited before signalling readiness");
            std::process::exit(1);
        }
    }
}

/// Run the MCP server in headless mode with a default-debounce watcher.
///
/// This compatibility wrapper uses a 500ms debounce when `save_path` is
/// `Some`; use [`run_headless_with_watch_config`] for explicit settings.
pub fn run_headless(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    save_path: Option<std::path::PathBuf>,
) {
    run_headless_with_watch_config(model, transport, default_watch_config(save_path));
}

/// Run the MCP server in headless mode with an explicit watcher setting.
///
/// `Some(config)` is forwarded unchanged to the save watcher and its owned
/// handle remains alive while the server runs. `None` disables file watching
/// while the server continues to use the supplied in-memory model.
pub fn run_headless_with_watch_config(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    watch_config: Option<nms_watch::WatchConfig>,
) {
    let mcp_service = ServiceHandle::new("mcp");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let watcher = start_watcher(watch_config);

        if let Err(e) = run_mcp_server(
            model,
            transport,
            watcher,
            None,
            None,
            ServerControl::Unmanaged,
            mcp_service,
        )
        .await
        {
            eprintln!("MCP server error: {e}");
        }
    });
}

/// Run headless MCP using already-applied updates from the application owner.
/// This observer never starts a watcher, mutates the model, or writes cache.
pub fn run_headless_with_updates(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    updates: Option<Receiver<AppliedSaveUpdate>>,
) {
    let service = ServiceHandle::new("mcp");
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("MCP runtime initialization error: {error}");
            return;
        }
    };
    runtime.block_on(async {
        if let Err(error) = run_mcp_server(
            model,
            transport,
            None,
            updates,
            None,
            ServerControl::Unmanaged,
            service,
        )
        .await
        {
            eprintln!("MCP server error: {error}");
        }
    });
}

fn default_watch_config(save_path: Option<std::path::PathBuf>) -> Option<nms_watch::WatchConfig> {
    save_path.map(|save_path| nms_watch::WatchConfig {
        save_path,
        ..Default::default()
    })
}

/// Start the watcher with the caller's exact configuration, if enabled.
///
/// Returns the owned watcher handle if watching started successfully.
fn start_watcher(watch_config: Option<nms_watch::WatchConfig>) -> Option<nms_watch::WatchHandle> {
    let watch_config = watch_config?;
    let path = watch_config.save_path.clone();
    match nms_watch::start_watching(watch_config) {
        Ok(handle) => {
            log::info!("File watcher started for {}", path.display());
            Some(handle)
        }
        Err(e) => {
            log::warn!("File watcher failed to start: {e}");
            None
        }
    }
}

/// Run the MCP server (async, expects to be called from a tokio runtime).
///
/// If a watcher receiver is provided, starts a background task that applies
/// deltas to the model and pushes notifications to connected MCP clients.
///
/// When `ready_tx` is provided (REPL mode), the sender is signalled once
/// the HTTP listener is bound so the caller can unblock. The `service_handle`
/// tracks startup state via fabryk-core's service lifecycle.
async fn run_mcp_server(
    model: Arc<RwLock<GalaxyModel>>,
    transport: Transport,
    mut watcher: Option<nms_watch::WatchHandle>,
    updates: Option<Receiver<AppliedSaveUpdate>>,
    ready_tx: Option<std::sync::mpsc::SyncSender<Result<(), String>>>,
    control: ServerControl,
    service_handle: ServiceHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "http")]
    let mut control = control;
    #[cfg(not(feature = "http"))]
    let _ = control;
    service_handle.set_state(ServiceState::Starting);

    let nms_tools = NmsTools::new(Arc::clone(&model));
    let tool_count = nms_tools.tool_count() + 1; // +1 for health
    let health = HealthTools::new("nms-copilot", env!("CARGO_PKG_VERSION"), tool_count);

    let composite = CompositeRegistry::new().add(nms_tools).add(health);
    let guidance = build_guidance();
    let nms_resources = NmsResources::new(Arc::clone(&model));
    let discoverable = DiscoverableRegistry::from_guidance(composite, &guidance);

    let server = FabrykMcpServer::new(discoverable)
        .with_name("nms-copilot")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_guidance(&guidance)
        .with_resources(nms_resources)
        .with_service(service_handle.clone());

    // Extract notifier before serving (serve consumes the server)
    let notifier = server.notifier();

    // Move only the receiver into the existing delta bridge. Keep the
    // WatchHandle here so its debouncer remains alive for the serving scope.
    let watcher_rx = watcher
        .as_mut()
        .map(|handle| std::mem::replace(&mut handle.receiver, std::sync::mpsc::channel().1));
    let mut delta_task = DeltaTaskGuard::default();
    if let Some(receiver) = watcher_rx {
        let model_for_watcher = Arc::clone(&model);
        let notifier_for_watcher = notifier.clone();
        delta_task.0 = Some(tokio::spawn(async move {
            apply_deltas_loop(receiver, model_for_watcher, notifier_for_watcher).await;
        }));
    }
    if let Some(receiver) = updates {
        delta_task.1 = Some(tokio::spawn(async move {
            observe_applied_updates(receiver, notifier).await;
        }));
    }

    let serve_result: Result<(), Box<dyn std::error::Error + Send + Sync>> = match transport {
        Transport::Stdio => {
            service_handle.set_state(ServiceState::Ready);
            if let Some(tx) = ready_tx {
                let _ = tx.send(Ok(()));
            }
            server.serve_stdio().await.map_err(Into::into)
        }
        #[cfg(feature = "http")]
        Transport::Http(addr) => {
            // Build the HTTP service and bind the listener ourselves so we
            // can signal readiness before entering the serve loop.
            let service = server.into_http_service();
            let router = fabryk_mcp::axum::Router::new()
                .route("/mcp-info", fabryk_mcp::axum::routing::get(mcp_info))
                .merge(fabryk_mcp::health_router(vec![service_handle.clone()]))
                .nest_service("/mcp", service);

            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    let msg = format!("bind {addr} failed: {e}");
                    service_handle.set_state(ServiceState::Failed(msg.clone()));
                    if let Some(tx) = ready_tx {
                        let _ = tx.send(Err(msg.clone()));
                    }
                    control.signal_error(McpBackgroundError::Bind {
                        address: addr,
                        source: e,
                    });
                    drop(watcher);
                    delta_task.abort_and_join().await;
                    return Err(msg.into());
                }
            };

            let bound_address = match listener.local_addr() {
                Ok(address) => address,
                Err(error) => {
                    let message = format!("local_addr for {addr} failed: {error}");
                    service_handle.set_state(ServiceState::Failed(message.clone()));
                    if let Some(tx) = ready_tx {
                        let _ = tx.send(Err(message.clone()));
                    }
                    control.signal_error(McpBackgroundError::LocalAddress {
                        address: addr,
                        source: error,
                    });
                    drop(watcher);
                    delta_task.abort_and_join().await;
                    return Err(message.into());
                }
            };

            // Bind succeeded — mark ready and unblock the caller
            service_handle.set_state(ServiceState::Ready);
            if let Some(tx) = ready_tx {
                let _ = tx.send(Ok(()));
            }
            control.signal_ready(bound_address);

            log::info!("NMS Copilot MCP server listening on http://{bound_address}");
            let serve =
                std::future::IntoFuture::into_future(fabryk_mcp::axum::serve(listener, router));
            tokio::pin!(serve);
            match &mut control {
                ServerControl::Owned { stop, .. } => {
                    tokio::select! {
                        result = &mut serve => result.map_err(Into::into),
                        _ = stop => Ok(()),
                    }
                }
                ServerControl::Unmanaged => serve.await.map_err(Into::into),
            }
        }
    };

    // Stop the producer before cancelling its async consumer. Early returns
    // above are covered by WatchHandle and DeltaTaskGuard drop semantics.
    drop(watcher);
    delta_task.abort_and_join().await;
    serve_result?;
    service_handle.set_state(ServiceState::Stopped);
    Ok(())
}

/// Own the delta task so every early return cancels it instead of detaching it.
#[derive(Default)]
struct DeltaTaskGuard(
    Option<tokio::task::JoinHandle<()>>,
    Option<tokio::task::JoinHandle<()>>,
);

impl DeltaTaskGuard {
    async fn abort_and_join(mut self) {
        for task in [self.0.take(), self.1.take()].into_iter().flatten() {
            task.abort();
            let _ = task.await;
        }
    }
}

impl Drop for DeltaTaskGuard {
    fn drop(&mut self) {
        for task in [&self.0, &self.1].into_iter().flatten() {
            task.abort();
        }
    }
}

#[cfg(feature = "http")]
async fn mcp_info() -> fabryk_mcp::axum::Json<serde_json::Value> {
    fabryk_mcp::axum::Json(mcp_info_json())
}

#[cfg(feature = "http")]
pub fn mcp_info_json() -> serde_json::Value {
    serde_json::json!({
        "name": "nms-copilot",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": "streamable-http",
        "stateful": true,
        "endpoints": {
            "mcp": "/mcp",
            "health": "/health",
            "info": "/mcp-info"
        },
        "session_header": "Mcp-Session-Id",
        "required_headers": {
            "post": {
                "accept": "application/json, text/event-stream",
                "content_type": "application/json"
            },
            "get": {
                "accept": "text/event-stream"
            }
        },
        "lifecycle": [
            "POST initialize without Mcp-Session-Id",
            "Capture Mcp-Session-Id from the initialize response header",
            "POST notifications/initialized with Mcp-Session-Id",
            "POST tools/list or tools/call with Mcp-Session-Id"
        ],
        "curl_example": "curl -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' --data '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-03-26\",\"capabilities\":{},\"clientInfo\":{\"name\":\"probe\",\"version\":\"1.0\"}}}' http://127.0.0.1:5055/mcp"
    })
}

#[cfg(all(test, feature = "http"))]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_info_json_documents_streamable_http() {
        let info = mcp_info_json();

        assert_eq!(info["name"], "nms-copilot");
        assert_eq!(info["transport"], "streamable-http");
        assert_eq!(info["stateful"], true);
        assert_eq!(info["endpoints"]["mcp"], "/mcp");
        assert_eq!(info["endpoints"]["health"], "/health");
        assert_eq!(info["endpoints"]["info"], "/mcp-info");
        assert_eq!(info["session_header"], "Mcp-Session-Id");
        assert_eq!(
            info["required_headers"]["post"]["accept"],
            "application/json, text/event-stream"
        );
    }
}

/// Background loop: receive deltas from watcher, apply to model, notify clients.
async fn apply_deltas_loop(
    receiver: std::sync::mpsc::Receiver<nms_core::SaveDelta>,
    model: Arc<RwLock<GalaxyModel>>,
    notifier: Notifier,
) {
    // Bridge std::sync::mpsc to tokio::sync::mpsc
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    thread::spawn(move || {
        while let Ok(delta) = receiver.recv() {
            if tx.send(delta).is_err() {
                break;
            }
        }
    });

    while let Some(delta) = rx.recv().await {
        {
            let mut model = model.write().await;
            model.apply_delta(&delta);
        }

        // Notify connected MCP clients about changes
        if let Some(ref moved) = delta.player_moved {
            log::info!("Live update: player moved");
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "player_moved",
                        "from": format!("{}", moved.from),
                        "to": format!("{}", moved.to),
                    }),
                )
                .await;
            notifier.resource_updated(PLAYER_LOCATION_URI).await;
        }

        if !delta.new_systems.is_empty() {
            let names: Vec<&str> = delta
                .new_systems
                .iter()
                .filter_map(|s| s.name.as_deref())
                .collect();
            log::info!("Live update: {} new system(s)", delta.new_systems.len());
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "new_systems",
                        "count": delta.new_systems.len(),
                        "names": names,
                    }),
                )
                .await;
            notifier.resource_updated(GALAXY_STATS_URI).await;
        }

        if !delta.new_planets.is_empty() {
            log::info!("Live update: {} new planet(s)", delta.new_planets.len());
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "new_planets",
                        "count": delta.new_planets.len(),
                    }),
                )
                .await;
            notifier.resource_updated(GALAXY_STATS_URI).await;
        }

        if !delta.new_bases.is_empty() {
            let names: Vec<&str> = delta.new_bases.iter().map(|b| b.name.as_str()).collect();
            log::info!("Live update: {} new base(s)", delta.new_bases.len());
            notifier
                .log(
                    fabryk_mcp::model::LoggingLevel::Info,
                    "nms-copilot",
                    serde_json::json!({
                        "event": "new_bases",
                        "count": delta.new_bases.len(),
                        "names": names,
                    }),
                )
                .await;
            notifier.resource_updated(BASES_URI).await;
        }

        if !delta.modified_bases.is_empty() {
            notifier.resource_updated(BASES_URI).await;
        }
    }
}

async fn observe_applied_updates(receiver: Receiver<AppliedSaveUpdate>, notifier: Notifier) {
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        loop {
            match receiver.try_recv() {
                Ok(event) => {
                    // Complete player state and source version can change even
                    // when the atlas delta is empty, so location is invalidated
                    // for every applied update.
                    notifier.resource_updated(PLAYER_LOCATION_URI).await;
                    notify_delta_resources(&notifier, &event.update.delta).await;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
            }
        }
    }
}

async fn notify_delta_resources(notifier: &Notifier, delta: &nms_core::SaveDelta) {
    if let Some(moved) = &delta.player_moved {
        log::info!("Live update: player moved");
        notifier.log(fabryk_mcp::model::LoggingLevel::Info, "nms-copilot", serde_json::json!({
            "event": "player_moved", "from": format!("{}", moved.from), "to": format!("{}", moved.to)
        })).await;
    }
    if !delta.new_systems.is_empty() {
        let names: Vec<&str> = delta
            .new_systems
            .iter()
            .filter_map(|system| system.name.as_deref())
            .collect();
        log::info!("Live update: {} new system(s)", delta.new_systems.len());
        notifier
            .log(
                fabryk_mcp::model::LoggingLevel::Info,
                "nms-copilot",
                serde_json::json!({
                    "event": "new_systems", "count": delta.new_systems.len(), "names": names
                }),
            )
            .await;
    }
    if !delta.new_planets.is_empty() {
        log::info!("Live update: {} new planet(s)", delta.new_planets.len());
        notifier
            .log(
                fabryk_mcp::model::LoggingLevel::Info,
                "nms-copilot",
                serde_json::json!({
                    "event": "new_planets", "count": delta.new_planets.len()
                }),
            )
            .await;
    }
    if !delta.new_bases.is_empty() {
        let names: Vec<&str> = delta
            .new_bases
            .iter()
            .map(|base| base.name.as_str())
            .collect();
        log::info!("Live update: {} new base(s)", delta.new_bases.len());
        notifier
            .log(
                fabryk_mcp::model::LoggingLevel::Info,
                "nms-copilot",
                serde_json::json!({
                    "event": "new_bases", "count": delta.new_bases.len(), "names": names
                }),
            )
            .await;
    }
    if !delta.new_systems.is_empty() || !delta.new_planets.is_empty() {
        notifier.resource_updated(GALAXY_STATS_URI).await;
    }
    if !delta.new_bases.is_empty() || !delta.modified_bases.is_empty() {
        notifier.resource_updated(BASES_URI).await;
    }
}

fn build_guidance() -> ServerGuidance {
    ServerGuidance::for_domain("nms")
        .context(
            "Galactic copilot for No Man's Sky. Search planets, plan routes, \
             convert portal glyphs, and explore your galaxy with an AI.",
        )
        .subscribe(
            PLAYER_LOCATION_URI,
            "Live warp tracking \u{2014} know when the player moves",
        )
        .subscribe(
            GALAXY_STATS_URI,
            "Updated when new systems/planets are discovered",
        )
        .subscribe(BASES_URI, "Know when new bases are built or modified")
        .workflow("Subscribe to recommended resources for live updates")
        .workflow("Call where_am_i to establish the player's current location")
        .workflow("Use search_planets or whats_nearby to find destinations")
        .workflow("Use plan_route to plan navigation between targets")
        .workflow("Use convert_coordinates to provide portal glyphs for in-game use")
        .convention("Distances are in light-years")
        .convention("Portal glyphs are 12 hex digits rendered as emoji")
        .convention("System and planet names may be unnamed (shown as \"-\")")
        .tool_metas(vec![
            (
                "search_planets",
                ToolMeta {
                    summary: "Search planets by biome, distance, discoverer, or name.".into(),
                    when_to_use: "Looking for planets with a specific biome or property".into(),
                    returns: "Ranked list of matching planets with coordinates and \
                              portal glyphs"
                        .into(),
                    next: Some("Call plan_route to navigate to results".into()),
                    category: Some("search".into()),
                },
            ),
            (
                "plan_route",
                ToolMeta {
                    summary: "Plan an optimal route through target systems.".into(),
                    when_to_use: "Need to visit multiple systems efficiently with warp range \
                         constraints"
                        .into(),
                    returns: "Step-by-step itinerary with distances and portal glyphs".into(),
                    next: Some(
                        "Use convert_coordinates to get portal addresses for waypoints".into(),
                    ),
                    category: Some("navigation".into()),
                },
            ),
            (
                "where_am_i",
                ToolMeta {
                    summary: "Get the player's current location.".into(),
                    when_to_use: "Need to know the player's current system and coordinates".into(),
                    returns: "System name, coordinates, portal glyphs, galaxy".into(),
                    next: Some("Call whats_nearby for situational awareness".into()),
                    category: Some("location".into()),
                },
            ),
            (
                "whats_nearby",
                ToolMeta {
                    summary: "Find systems and planets near the player's current \
                              position."
                        .into(),
                    when_to_use: "Need situational awareness or looking for nearby options".into(),
                    returns: "Nearby systems with distances, biomes, and portal glyphs".into(),
                    next: Some(
                        "Call search_planets for filtered results or plan_route to \
                         navigate"
                            .into(),
                    ),
                    category: Some("search".into()),
                },
            ),
            (
                "show_system",
                ToolMeta {
                    summary: "Get detailed information about a star system.".into(),
                    when_to_use: "Need details about a specific system (planets, \
                                  discoverer, coordinates)"
                        .into(),
                    returns: "System details with all discovered planets and their \
                              biomes"
                        .into(),
                    next: None,
                    category: Some("detail".into()),
                },
            ),
            (
                "show_base",
                ToolMeta {
                    summary: "Get detailed information about a player base.".into(),
                    when_to_use: "Need details about a specific base (location, type, system)"
                        .into(),
                    returns: "Base details with portal glyphs and system context".into(),
                    next: None,
                    category: Some("detail".into()),
                },
            ),
            (
                "convert_coordinates",
                ToolMeta {
                    summary: "Convert between portal glyphs, signal booster, and \
                              galactic addresses."
                        .into(),
                    when_to_use: "Need to convert coordinates between formats for in-game use"
                        .into(),
                    returns: "All coordinate formats for the given address".into(),
                    next: None,
                    category: Some("utility".into()),
                },
            ),
            (
                "galaxy_stats",
                ToolMeta {
                    summary: "Get aggregate statistics about the explored galaxy.".into(),
                    when_to_use: "Want an overview of discoveries, biome distribution, \
                                  or progress"
                        .into(),
                    returns: "System/planet/base counts with biome breakdown".into(),
                    next: Some("Call search_planets to explore specific biomes".into()),
                    category: Some("overview".into()),
                },
            ),
        ])
}

#[cfg(test)]
mod watcher_lifecycle_tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn watch_config(save_path: std::path::PathBuf) -> nms_watch::WatchConfig {
        nms_watch::WatchConfig {
            save_path,
            debounce: Duration::from_millis(500),
        }
    }

    fn save_bytes(reality_index: u8, voxel_x: i16) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "LiveWatch", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {
                        "RealityIndex": reality_index,
                        "GalacticAddress": {
                            "VoxelX": voxel_x,
                            "VoxelY": 0,
                            "VoxelZ": 0,
                            "SolarSystemIndex": 1,
                            "PlanetIndex": 0
                        }
                    },
                    "Units": 7,
                    "Nanites": 8,
                    "Specials": 9,
                    "PersistentPlayerBases": []
                }
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {
                    "UniverseAddress": {
                        "RealityIndex": 0,
                        "GalacticAddress": {
                            "VoxelX": 0,
                            "VoxelY": 0,
                            "VoxelZ": 0,
                            "SolarSystemIndex": 0,
                            "PlanetIndex": 0
                        }
                    },
                    "Units": 0,
                    "Nanites": 0,
                    "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "DiscoveryManagerData": {
                "DiscoveryData-v1": {
                    "ReserveStore": 0,
                    "ReserveManaged": 0,
                    "Store": {"Record": []}
                }
            }
        }))
        .expect("JSON fixture serializes")
    }

    #[test]
    fn test_start_watcher_none_missing_and_malformed_falls_back_without_handle() {
        assert!(start_watcher(None).is_none());

        let dir = tempfile::tempdir().expect("temporary directory");
        let missing = dir.path().join("missing.json");
        assert!(start_watcher(Some(watch_config(missing))).is_none());

        let malformed = dir.path().join("malformed.json");
        std::fs::write(&malformed, b"not a save").expect("write malformed save");
        assert!(start_watcher(Some(watch_config(malformed))).is_none());
    }

    #[test]
    fn test_legacy_watch_config_preserves_path_and_default_debounce() {
        assert!(default_watch_config(None).is_none());
        let path = std::path::PathBuf::from("relative/../save.hg");

        let config = default_watch_config(Some(path.clone())).expect("path enables default watch");

        assert_eq!(config.save_path, path);
        assert_eq!(config.debounce, Duration::from_millis(500));
    }

    #[test]
    fn test_retained_watcher_receives_later_write_and_owner_drop_disconnects() {
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = dir.path().join("save.json");
        std::fs::write(&path, save_bytes(0, 0)).expect("write initial save");

        let mut owner = start_watcher(Some(watch_config(path.clone()))).expect("watcher starts");
        let receiver = std::mem::replace(&mut owner.receiver, std::sync::mpsc::channel().1);
        std::fs::write(&path, save_bytes(9, 0)).expect("write later save contents");

        let delta = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("watcher observes the actual file replacement");
        let moved = delta.player_moved.expect("player moved to reality 9");
        assert_eq!(moved.from.reality_index, 0);
        assert_eq!(moved.to.reality_index, 9);
        assert_eq!(moved.to.voxel_x(), 0);

        drop(owner);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) if Instant::now() < deadline => {}
                result => panic!("watch producer did not disconnect after owner drop: {result:?}"),
            }
        }
    }
}
