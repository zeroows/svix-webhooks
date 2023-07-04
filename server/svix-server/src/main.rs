// SPDX-FileCopyrightText: © 2022 Svix Authors
// SPDX-License-Identifier: MIT

#![warn(clippy::all)]
#![forbid(unsafe_code)]

use dotenv::dotenv;
use std::process::exit;
use svix_server::core::types::{EndpointSecretInternal, OrganizationId};
use svix_server::db::wipe_org;
use validator::Validate;

use svix_server::core::security::{default_org_id, generate_org_token};

use svix_server::{cfg, db, run, setup_tracing};

use clap::{Parser, Subcommand};

mod wait_for;
use wait_for::wait_for_dsn;

// The names and default ports of services to wait-for
const POSTGRES_NAME: &str = "PostgreSQL";
const POSTGRES_PORT: u16 = 5432;

const REDIS_NAME: &str = "Redis";
const REDIS_PORT: u16 = 6379;

#[derive(Parser)]
#[clap(author, version, about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// Run database migrations before starting
    #[clap(long, value_parser)]
    run_migrations: bool,

    /// The time to wait for a successful connection to the database before timing out in seconds.
    #[clap(long, value_parser)]
    wait_for: Option<u64>,
}

#[derive(Subcommand)]
enum Commands {
    /// JWT utilities
    #[clap()]
    Jwt {
        #[clap(subcommand)]
        command: JwtCommands,
    },
    /// Asymmetric Key utilities
    #[clap()]
    AsymmetricKey {
        #[clap(subcommand)]
        command: AsymmetricKeyCommands,
    },
    /// Run database migrations and exit
    #[clap()]
    Migrate,

    #[clap()]
    Wipe {
        #[clap(value_parser = org_id_parser)]
        org_id: OrganizationId,

        #[clap(long)]
        yes_i_know_what_im_doing: bool,
    },

    /// Generate OpenAPI JSON specification and exit
    #[clap()]
    GenerateOpenapi,
}

#[derive(Subcommand)]
enum JwtCommands {
    /// Generate a new JWT
    #[clap()]
    Generate {
        #[clap(value_parser = org_id_parser)]
        /// Optional org_id to use when generating token (otherwise, default is used).
        org_id: Option<OrganizationId>,
    },
}

#[derive(Subcommand)]
enum AsymmetricKeyCommands {
    /// Generate a new asymmetric key
    #[clap()]
    Generate,
}

fn org_id_parser(s: &str) -> Result<OrganizationId, String> {
    let ret = OrganizationId(s.to_owned());
    ret.validate().map_err(|x| x.to_string())?;
    Ok(ret)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args = Args::parse();
    let cfg = cfg::load().expect("Error loading configuration");

    setup_tracing(&cfg);

    if let Some(wait_for_seconds) = args.wait_for {
        let mut wait_for = Vec::with_capacity(2);
        wait_for.push(wait_for_dsn(
            &cfg.db_dsn,
            POSTGRES_NAME,
            POSTGRES_PORT,
            wait_for_seconds,
        ));

        if let Some(redis_dsn) = &cfg.redis_dsn {
            wait_for.push(wait_for_dsn(
                redis_dsn,
                REDIS_NAME,
                REDIS_PORT,
                wait_for_seconds,
            ));
        }

        futures::future::join_all(wait_for).await;
    }

    if args.run_migrations {
        tracing::debug!("Migrations: Running");
        db::run_migrations(&cfg).await;
        tracing::debug!("Migrations: Success");
    }

    match args.command {
        Some(Commands::Migrate) => {
            db::run_migrations(&cfg).await;
            println!("Migrations: success");
            exit(0);
        }
        Some(Commands::Jwt {
            command: JwtCommands::Generate { org_id },
        }) => {
            let org_id = org_id.unwrap_or_else(default_org_id);
            let token =
                generate_org_token(&cfg.jwt_secret, org_id).expect("Error generating token");
            println!("Token (Bearer): {token}");
            exit(0);
        }
        Some(Commands::AsymmetricKey { command }) => match command {
            AsymmetricKeyCommands::Generate => {
                let secret = EndpointSecretInternal::generate_asymmetric(&cfg.encryption)
                    .unwrap()
                    .into_endpoint_secret(&cfg.encryption)
                    .unwrap();
                println!("Secret key: {}", secret.serialize_secret_key());
                println!("Public key: {}", secret.serialize_public_key());
                exit(0);
            }
        },
        Some(Commands::Wipe {
            org_id,
            yes_i_know_what_im_doing,
        }) => {
            if yes_i_know_what_im_doing {
                wipe_org(&cfg, org_id).await;
            } else {
                println!("Please confirm you wish to wipe this organization with the `--yes-i-know-what-im-doing` flag");
            }

            exit(0);
        }
        Some(Commands::GenerateOpenapi) => {
            let mut openapi = svix_server::openapi::initialize_openapi();

            let router = svix_server::v1::router();
            aide::axum::ApiRouter::new()
                .nest("/api/v1", router)
                .finish_api(&mut openapi);

            let openapi = svix_server::openapi::postprocess_spec(openapi);
            println!(
                "{}",
                serde_json::to_string_pretty(&openapi).expect("Failed to serialize JSON spec")
            );

            exit(0);
        }
        None => {}
    };

    run(cfg, None).await;

    opentelemetry::global::shutdown_tracer_provider();
}
