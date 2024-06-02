use crate::scheduler::Scheduler;
use axum::Router;

pub struct KorektoService {
    router: Router,
    scheduler: Scheduler,
}

impl KorektoService {
    pub const fn new(router: Router, scheduler: Scheduler) -> Result<Self, shuttle_runtime::Error> {
        Ok(Self { router, scheduler })
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for KorektoService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let server = async move {
            axum::serve(
                shuttle_runtime::tokio::net::TcpListener::bind(addr).await?,
                self.router,
            )
            .await
        };

        let (_scheduler_hdl, _axum_hdl) = tokio::join!(self.scheduler.start(), server);

        Ok(())
    }
}

pub type KorektoServiceResult = Result<KorektoService, shuttle_runtime::Error>;
