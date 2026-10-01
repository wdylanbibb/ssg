use std::{error::Error, net::SocketAddr, path::PathBuf, process::ExitCode};

use clap::Parser;
use thiserror::Error;

#[derive(Parser)]
enum SsgCli {
    Build(BuildArgs),
    Serve(ServeArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct BuildArgs {
    #[arg(long)]
    source: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ServeArgs {
    #[arg(long)]
    source: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    address: Option<SocketAddr>,
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
    #[error("failed to build the site")]
    Build(#[from] ssg::BuildError),

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
        SsgCli::Serve(ServeArgs {
            source,
            output,
            address,
        }) => {
            serve(source, output, address).await?;
        }
    }

    Ok(())
}

fn build(source: PathBuf, output: Option<PathBuf>) -> Result<PathBuf, ssg::BuildError> {
    let output = output.unwrap_or_else(|| PathBuf::from("./public"));
    let report = ssg::build_site(&source, &output)?;

    println!(
        "Built {} pages and copied {} assets into {}",
        report.pages_written,
        report.assets_copied,
        output.display()
    );

    Ok(output)
}

async fn serve(
    source: PathBuf,
    output: Option<PathBuf>,
    address: Option<SocketAddr>,
) -> Result<(), ServeError> {
    let output = build(source, output)?;
    let address = address.unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 8080)));

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|source| ServeError::Bind { address, source })?;

    let app = axum::Router::new().fallback_service(tower_http::services::ServeDir::new(output));

    axum::serve(listener, app).await.map_err(ServeError::Server)
}
