use log::{error, info};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::sync::Arc;
use std::{
    collections::HashMap,
    time::{self, SystemTime},
};
use tokio_util::sync::CancellationToken;
use crate::consts;

type CallbackFn = Box<dyn Fn(ServiceEvent) + Send + Sync>;

pub struct MdnsActorState {
    daemon: ServiceDaemon,
    current_service: Option<ServiceInfo>,
    service_type: String,
    cancellation_token: CancellationToken,
    cb: Option<Arc<CallbackFn>>,
}

impl MdnsActorState {
    pub fn new() -> anyhow::Result<Self> {
        anyhow::Ok(Self {
            daemon: ServiceDaemon::new()?,
            current_service: None,
            service_type: consts::SERVICE_TYPE.to_string(),
            cancellation_token: CancellationToken::new(),
            cb: None,
        })
    }

    pub fn set_call_fn(&mut self, cb:CallbackFn) {
        self.cb = Some(Arc::new(cb));
    }

    fn build_service_info(&self,service_type: &str, name: &str,ip: &str, port: u16) -> anyhow::Result<ServiceInfo>{
        let mut props: HashMap<String, String> = HashMap::new();

        let now = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        props.insert("update_at".into(), format!("{}", now));

        anyhow::Ok(ServiceInfo::new(
            service_type,
            name,
            format!("{}.local.", name).as_str(),
            ip,
            port,
            props,
        )?)
    }

    pub fn register(&mut self, name: String, ip: String,port: u16, ) -> anyhow::Result<()>{

        // 停止监听
        if let Some(ref old) = self.current_service {
            self.daemon.unregister(old.get_fullname())?;
        }

        let service_info_build = self.build_service_info(&self.service_type, &name, &ip,port)?;
        self.daemon.register(service_info_build.clone())?;

        self.current_service = Some(service_info_build);

        Ok(())
    }

    pub fn start_listen(&mut self) -> anyhow::Result<()> {
        // CancellationToken 是一次性使用的信号量。一旦调用了 cancel() 方法，它的状态就会永久变为已取消（canceled）。
        // 每次注册 重新创建
        if self.cancellation_token.is_cancelled() {
            self.cancellation_token  = CancellationToken::new();
        }

        let c_cancellation_token = self.cancellation_token.clone();

        let receiver = self.daemon.browse(&self.service_type)?;

        match self.cb {
            None => {
                info!("start listen bu can not find callback")
            }
            Some(ref cb) => {
                let f = cb.clone();
                info!("start listen");
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                    _ = c_cancellation_token.cancelled() => {
                        break
                    }
                    recv_res = receiver.recv_async() => {
                        match recv_res {
                            Ok(event) => {
                                f(event);
                            }
                            Err(e) => {
                                error!("Browse error: {}", e)
                            }
                        }
                    }
                }
                    }
                });
            }
        }




        Ok(())
    }

    pub fn close_listen(&mut self) -> anyhow::Result<()> {
        // if let Some(ref old) = self.current_service {
        //     self.daemon.unregister(old.get_fullname())?;
        // }
        info!("close_listen");
        self.cancellation_token.cancel();
        Ok(())
    }
}