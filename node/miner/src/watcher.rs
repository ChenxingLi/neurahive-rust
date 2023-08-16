#![allow(unused)]

use contract_interface::{nrhv_flow::MineContext, NrhvFlow, PoraMine};
use ethereum_types::{Address, H256, U256};
use ethers::{
    contract::Contract,
    providers::{JsonRpcClient, Middleware, Provider, StreamExt},
    types::BlockId,
};
use task_executor::TaskExecutor;
use tokio::{
    sync::{broadcast, mpsc},
    time::{sleep, Instant, Sleep},
    try_join,
};

use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::{ops::DerefMut, str::FromStr};

use crate::{config::MineServiceMiddleware, MinerConfig, MinerMessage};

pub type MineContextMessage = Option<(MineContext, U256)>;

lazy_static! {
    pub static ref EMPTY_HASH: H256 =
        H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap();
}

pub struct MineContextWatcher {
    provider: Arc<MineServiceMiddleware>,
    flow_contract: NrhvFlow<MineServiceMiddleware>,
    mine_contract: PoraMine<MineServiceMiddleware>,

    mine_context_sender: mpsc::UnboundedSender<MineContextMessage>,
    last_report: MineContextMessage,

    msg_recv: broadcast::Receiver<MinerMessage>,
}

impl MineContextWatcher {
    pub fn spawn(
        executor: TaskExecutor,
        msg_recv: broadcast::Receiver<MinerMessage>,
        provider: Arc<MineServiceMiddleware>,
        config: &MinerConfig,
    ) -> mpsc::UnboundedReceiver<MineContextMessage> {
        let provider = provider;

        let mine_contract = PoraMine::new(config.mine_address, provider.clone());
        let flow_contract = NrhvFlow::new(config.flow_address, provider.clone());

        let (mine_context_sender, mine_context_receiver) =
            mpsc::unbounded_channel::<MineContextMessage>();
        let watcher = MineContextWatcher {
            provider,
            flow_contract,
            mine_contract,
            mine_context_sender,
            msg_recv,
            last_report: None,
        };
        executor.spawn(
            async move { Box::pin(watcher.start()).await },
            "mine_context_watcher",
        );
        mine_context_receiver
    }

    async fn start(mut self) {
        let mut mining_enabled = true;
        let mut channel_opened = true;

        let mut mining_throttle = sleep(Duration::from_secs(0));
        tokio::pin!(mining_throttle);

        loop {
            tokio::select! {
                biased;

                v = self.msg_recv.recv(), if channel_opened => {
                    match v {
                        Ok(MinerMessage::ToggleMining(enable)) => {
                            mining_enabled = enable;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            channel_opened = false;
                        }
                        _ => {}
                    }
                }

                () = &mut mining_throttle, if !mining_throttle.is_elapsed() => {
                }

                _ = async {}, if mining_enabled && mining_throttle.is_elapsed() => {
                    mining_throttle.as_mut().reset(Instant::now() + Duration::from_secs(1));
                    if let Err(err) = self.query_recent_context().await {
                        warn!(err);
                    }
                }
            }
        }
    }

    async fn query_recent_context(&mut self) -> Result<(), String> {
        // let mut watcher = self
        //     .provider
        //     .watch_blocks()
        //     .await
        //     .expect("should success")
        //     .stream();
        // watcher.next().await
        let context_call = self.flow_contract.make_context_with_result();
        let epoch_call = self.mine_contract.last_mined_epoch();
        let quality_call = self.mine_contract.target_quality();

        let (context, epoch, quality) =
            try_join!(context_call.call(), epoch_call.call(), quality_call.call())
                .map_err(|e| format!("Failed to query mining context: {:?}", e))?;
        let report = if context.epoch > epoch && context.digest != EMPTY_HASH.0 {
            if context.block_digest == [0; 32] {
                warn!("Mine Context is not updated on time.");
                None
            } else {
                Some((context, quality))
            }
        } else {
            None
        };

        if report != self.last_report {
            self.mine_context_sender
                .send(report.clone())
                .map_err(|e| format!("Failed to send out the most recent mine context: {:?}", e))?;
        }
        self.last_report = report;

        Ok(())
    }
}
