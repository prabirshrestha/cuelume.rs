use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderValue, header},
    response::Html,
    routing::get,
};

const DEFAULT_ADDRESS: &str = "127.0.0.1:8080";
const WASM_ARTIFACT: &str = "wasm_app";
const WASM_PACKAGE: &str = "cuelume_wasm_example";

#[derive(Clone)]
struct Assets {
    javascript: Bytes,
    wasm: Bytes,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_ADDRESS.to_owned())
        .parse::<SocketAddr>()?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets = build_wasm(&root)?;
    let app = Router::new()
        .route("/", get(index))
        .route(&format!("/{WASM_PACKAGE}.js"), get(javascript))
        .route(&format!("/{WASM_PACKAGE}_bg.wasm"), get(wasm))
        .with_state(assets);
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!("Cuelume WebAssembly example: http://{address}");
    println!("Press Ctrl-C to stop.");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn build_wasm(root: &Path) -> Result<Assets, Box<dyn std::error::Error>> {
    let manifest = root.join("Cargo.toml");
    let build = root.join("target/wasm-example");
    let cargo_target = build.join("cargo");
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .arg("build")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--example")
        .arg("wasm-app")
        .arg("--features")
        .arg("wasm-example")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--target-dir")
        .arg(&cargo_target)
        .status()?;
    if !status.success() {
        return Err("the WebAssembly build failed".into());
    }

    let target = cargo_target.join("wasm32-unknown-unknown/debug/examples");
    let generated = build.join("pkg");
    fs::create_dir_all(&generated)?;
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(target.join(format!("{WASM_ARTIFACT}.wasm")))
        .out_name(WASM_PACKAGE)
        .web(true)?
        .omit_default_module_path(false)
        .typescript(false)
        .generate(&generated)?;

    Ok(Assets {
        javascript: Bytes::from(fs::read(generated.join(format!("{WASM_PACKAGE}.js")))?),
        wasm: Bytes::from(fs::read(generated.join(format!("{WASM_PACKAGE}_bg.wasm")))?),
    })
}

async fn index() -> Html<&'static str> {
    Html(include_str!("wasm/index.html"))
}

async fn javascript(
    State(assets): State<Assets>,
) -> ([(header::HeaderName, HeaderValue); 1], Bytes) {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/javascript"),
        )],
        assets.javascript,
    )
}

async fn wasm(State(assets): State<Assets>) -> ([(header::HeaderName, HeaderValue); 1], Bytes) {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/wasm"),
        )],
        assets.wasm,
    )
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("could not listen for Ctrl-C: {error}");
    }
}
