use anyhow::Result;
use block_explorer::blockexplorerapi::BlockExplorerApi;
use log::debug;

use crate::router::{get_router, Router, RouterSettings};

pub async fn load_dex_routers<T: BlockExplorerApi>(
    block_exlorer: T,
    router_settings: Vec<RouterSettings>,
) -> Result<Vec<Router>> {
    let mut routers: Vec<Router> = Vec::new();
    for router in router_settings {
        debug!("Loading router: {}", router.name);

        let r = get_router(
            &block_exlorer,
            router.addresses.clone(),
            router.factory,
            router.name.clone(),
            router.version,
        )
        .await?;

        debug!("Loaded router: {}", r);

        routers.push(r);
    }

    Ok(routers)
}
