use crate::core::consts;
use crate::core::receiver::spawn_receiver_actor;
use crate::core::sender::spawn_sender_actor;
use crate::core::CusResult;
use crate::core::models::{MessageTransAccept, MessageTransHandle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use log::{error, info};

#[derive(Debug)]
pub struct TransHandle {
    save_dir: Arc<Mutex<PathBuf>>,
    ts_chan :Option<Sender<MessageTransAccept>>,
    ts_handle_chan: Option<JoinHandle<()>>,
    tr_handle_chan: Option<JoinHandle<()>>,
    ts_cancel_token: Option<CancellationToken>,
    tr_cancel_token: Option<CancellationToken>,
}

impl TransHandle{

    pub fn init() -> Self {
        Self{
            save_dir: Arc::new(Default::default()),
            ts_chan: None,
            ts_handle_chan: None,
            tr_handle_chan: None,
            ts_cancel_token: None,
            tr_cancel_token: None,
        }
    }

    pub fn start_sender<F>(&mut self, on_service_resolved: F) -> CusResult<()>
    where
        F: Fn(MessageTransHandle) + Send +'static
    {
        self.close_send_handle();

        let (accept_sender, mut handle_receiver, cancellation_token) = spawn_sender_actor(consts::CHANNEL_BUFFER_SIZE);

        let c_cancellation_token = cancellation_token.clone();
        let channel_task = tokio::spawn(async move{
            loop {
                tokio::select! {
                    _ = c_cancellation_token.cancelled() => {
                        info!("sender task stop");
                        break;
                    }
                    rece_res = handle_receiver.recv() => {
                        match rece_res{
                            Ok(message)  => {
                                on_service_resolved(message)
                            }
                            Err(e) => {
                                error!("sender message error {}",e)
                            }
                        }
                    }
                }
            }
            info!("sender task stop");
        });

        self.ts_chan = Some(accept_sender);
        self.ts_cancel_token = Some(cancellation_token);
        self.ts_handle_chan = Some(channel_task);

        Ok(())
    }

    pub fn start_receiver<F>(&mut self, addr: String, on_service_resolved: F) -> CusResult<()>
    where
        F: Fn(MessageTransHandle) +Send +'static
    {
        self.close_receive_handle();

        let (mut receiver, cancellation_token) = spawn_receiver_actor(addr,consts::CHANNEL_BUFFER_SIZE,self.save_dir.clone());

        let c_cancellation_token = cancellation_token.clone();
        let channel_task = tokio::spawn(async move {
           loop {
               tokio::select! {
                   _ = c_cancellation_token.cancelled()=> {
                       info!("receive task stop");
                       break;
                   }
                   rece_res = receiver.recv() => {
                       match rece_res{
                           Ok(message) => {
                               on_service_resolved(message)
                           }
                           Err(e) => {
                               error!("receive message error {}",e)
                           }
                       }
                   }
               }
           }
            info!("receive task stop")
        });

        self.tr_cancel_token = Some(cancellation_token);
        self.tr_handle_chan = Some(channel_task);

        Ok(())
    }

    pub fn change_save_dir(&mut self, new_path: PathBuf) {
        let c_save_dir = self.save_dir.clone();
        tokio::spawn(async move{
            *c_save_dir.lock().await = new_path;
        });
    }

    pub fn send_files(&mut self, message: MessageTransAccept) -> CusResult<()> {

        if let Some(ref sender) = self.ts_chan {
            info!("send message {:?}",message);
            sender.send(message)?;
        }

        Ok(())
    }

    pub fn close_send_handle(&mut self){
        if let Some(cancellation_token) = self.ts_cancel_token.take() {
            cancellation_token.cancel();
        }

        // 将 if let 语句替换为更简洁的写法，利用 let else 或直接使用 if let 的紧凑形式来减少嵌套层级并提升可读性。
        if let Some(ref sender) = self.ts_chan
            &&
            let Err(e) = sender.send(MessageTransAccept::Shutdown){
                error!("send message error {}",e)
            }

        self.ts_chan = None;
        self.ts_cancel_token = None;
    }

    pub fn close_receive_handle(&mut self){
        if let Some(cancellation_token) = self.tr_cancel_token.take(){
            cancellation_token.cancel();
        }
        self.tr_handle_chan = None;
    }

    #[allow(unused)]
    pub async fn block(&mut self){
       let sender_task = self.ts_handle_chan.take();
        let receiver_task = self.tr_handle_chan.take();

        tokio::join!(
            async{
                if let Some(task) = sender_task{
                    if let Err(e) = task.await{
                        error!("sender task error {}",e)
                    }
                }
            },
            async{
                if let Some(task) = receiver_task {
                    if let Err(e) = task.await {
                        error!("receive task error {}",e)
                    }
                }
            }
        );

        info!("all task bave been stopped.");
    }

}