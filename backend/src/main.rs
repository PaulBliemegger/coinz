use clap::{Parser, Subcommand};
use coinz::{Config, auth, db};
use std::{
    io::{self, Read},
    path::PathBuf,
};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version, about = "A little clarity for your coinz")]
struct Cli {
    #[arg(long, env = "COINZ_DATA_DIR", default_value = "data", global = true)]
    data_dir: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the web app and API.
    Serve {
        #[arg(long, env = "COINZ_BIND", default_value = "127.0.0.1:3000")]
        bind: String,
        #[arg(long, env = "COINZ_ORIGIN", default_value = "http://localhost:3000")]
        origin: String,
        #[arg(long, env = "COINZ_STATIC_DIR", default_value = "frontend/build")]
        static_dir: PathBuf,
        #[arg(long, env = "COINZ_COOKIE_SECURE", default_value = "false")]
        secure_cookie: bool,
    },
    /// Create a local user. Passwords are prompted for, never passed as arguments.
    CreateUser {
        username: String,
        /// Read the password from stdin (useful with docker compose run -T).
        #[arg(long)]
        password_stdin: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "coinz=info,tower_http=info".into()),
        )
        .init();
    let args = Cli::parse();
    let pool = db::connect(&args.data_dir).await?;
    match args.command {
        Command::CreateUser {
            username,
            password_stdin,
        } => {
            let password = if password_stdin {
                let mut password = String::new();
                io::stdin().take(1024).read_to_string(&mut password)?;
                password.trim_end_matches(['\r', '\n']).to_owned()
            } else {
                let password = rpassword::prompt_password("Password (at least 12 characters): ")?;
                if password != rpassword::prompt_password("Confirm password: ")? {
                    return Err("Passwords do not match.".into());
                }
                password
            };
            let user = auth::create_user(&pool, &username, &password).await?;
            println!("Created user {}.", user.username);
        }
        Command::Serve {
            bind,
            origin,
            static_dir,
            secure_cookie,
        } => {
            let origin = origin.trim_end_matches('/').to_owned();
            let parsed: axum::http::Uri = origin.parse()?;
            if !matches!(parsed.scheme_str(), Some("http" | "https"))
                || parsed.authority().is_none()
                || parsed.path() != "/"
            {
                return Err("COINZ_ORIGIN must be an http(s) origin without a path.".into());
            }
            let app = coinz::app(
                pool,
                Config {
                    origin,
                    secure_cookie,
                    static_dir,
                },
            )
            .await?;
            let listener = tokio::net::TcpListener::bind(&bind).await?;
            tracing::info!(%bind, "coinz is ready");
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
        }
    }
    Ok(())
}
