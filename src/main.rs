use std::{
    error::Error,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use thiserror::Error;

#[derive(Parser)]
enum SsgCli {
    Build(BuildArgs),
    Serve(ServeArgs),
    Preview(PreviewArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct BuildArgs {
    #[arg(long)]
    source: PathBuf,
    #[arg(long, default_value = "./public")]
    output: PathBuf,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ServeArgs {
    #[arg(long)]
    root: PathBuf,
    #[arg(long, default_value = "0.0.0.0:8080")]
    address: SocketAddr,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct PreviewArgs {
    #[arg(long)]
    source: PathBuf,
    #[arg(long, default_value = "./public")]
    output: PathBuf,
    #[arg(long, default_value = "0.0.0.0:8080")]
    address: SocketAddr,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("site generation failed")]
    Build(#[from] ssg::BuildError),

    #[error("server failed")]
    Serve(#[from] ServeError),
}

#[derive(Debug, Error)]
enum ServeError {
    #[error("failed to inspect static site root {path}")]
    InspectRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("static site root {0} is not a directory")]
    RootNotDirectory(PathBuf),

    #[error("failed to bind server to {address}")]
    Bind {
        address: SocketAddr,
        #[source]
        source: std::io::Error,
    },

    #[error("HTTP server failed")]
    Server(#[source] std::io::Error),
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");

            let mut source = error.source();
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }

            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), CliError> {
    match SsgCli::parse() {
        SsgCli::Build(BuildArgs { source, output }) => {
            build(source, output)?;
        }
        SsgCli::Serve(ServeArgs { root, address }) => {
            serve(root, address).await?;
        }
        SsgCli::Preview(PreviewArgs {
            source,
            output,
            address,
        }) => {
            preview(source, output, address).await?;
        }
    }

    Ok(())
}

fn build(source: PathBuf, output: PathBuf) -> Result<PathBuf, ssg::BuildError> {
    let report = ssg::build_site(&source, &output)?;

    println!(
        "Built {} pages and copied {} assets into {}",
        report.pages_written,
        report.assets_copied,
        output.display()
    );

    Ok(output)
}

async fn serve(root: PathBuf, address: SocketAddr) -> Result<(), ServeError> {
    validate_root(&root)?;

    let app = axum::Router::new()
        .route(
            "/healthz",
            axum::routing::get(|| async { axum::http::StatusCode::OK }),
        )
        .fallback_service(tower_http::services::ServeDir::new(root));

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|source| ServeError::Bind { address, source })?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServeError::Server)?;

    Ok(())
}

fn validate_root(root: &Path) -> Result<(), ServeError> {
    let metadata = std::fs::metadata(root).map_err(|source| ServeError::InspectRoot {
        path: root.to_path_buf(),
        source,
    })?;

    if !metadata.is_dir() {
        return Err(ServeError::RootNotDirectory(root.to_path_buf()));
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C signal handler");
    };

    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM signal handler");

        tokio::select! {
            () = ctrl_c => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await;
    }
}

async fn preview(source: PathBuf, output: PathBuf, address: SocketAddr) -> Result<(), CliError> {
    serve(build(source, output)?, address).await?;
    Ok(())
}
