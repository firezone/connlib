use clap::Parser;
use firezone_client_connlib::{
    Callbacks, Error, ErrorType, ResourceList, Session, TunnelAddresses,
};
use url::Url;

enum CallbackHandler {}

impl Callbacks for CallbackHandler {
    fn on_update_resources(_resource_list: ResourceList) {
        todo!()
    }

    fn on_set_tunnel_adresses(_tunnel_addresses: TunnelAddresses) {
        todo!()
    }

    fn on_error(error: &Error, error_type: ErrorType) {
        match error_type {
            ErrorType::Recoverable => tracing::warn!("Encountered error: {error}"),
            ErrorType::Fatal => panic!("Encountered fatal error: {error}"),
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    // TODO: read args from env instead
    let args = Args::parse();
    // TODO: This is disgusting
    let mut session =
        Session::<CallbackHandler>::connect::<CallbackHandler>(args.url, args.secret).unwrap();
    tracing::info!("Started new session");
    session.wait_for_ctrl_c().unwrap();
    session.disconnect();
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    #[arg(long)]
    pub secret: String,
    #[arg(long)]
    pub url: Url,
}
