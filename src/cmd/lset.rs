use std::sync::Arc;

use crate::cmd::{Parse};
use crate::tikv::errors::AsyncResult;
use crate::tikv::list::ListCommandCtx;
use crate::{Connection, Frame};
use crate::config::{is_use_txn_api};
use crate::utils::{resp_err, resp_invalid_arguments};

use bytes::Bytes;
use tikv_client::Transaction;
use tokio::sync::Mutex;
use crate::config::LOGGER;
use slog::debug;

#[derive(Debug)]
pub struct Lset {
    key: String,
    idx: i64,
    element: Bytes,
    valid: bool,
}

impl Lset {
    pub fn new(key: &str, idx: i64, ele: Bytes) -> Lset {
        Lset {
            key: key.to_owned(),
            idx: idx,
            element: ele.to_owned(),
            valid: true
        }
    }

    pub fn new_invalid() -> Lset {
        Lset {
            key: "".to_owned(),
            idx: 0,
            element: Bytes::new(),
            valid: false
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Lset> {
        let key = parse.next_string()?;
        let idx = parse.next_int()?;
        let element = parse.next_bytes()?;

        Ok(Lset { key, idx, element, valid: true})
    }

    pub(crate) fn parse_argv(argv: &Vec<String>) -> crate::Result<Lset> {
        if argv.len() != 3 {
            return Ok(Lset::new_invalid());
        }
        let key = &argv[0];
        let idx;
        match argv[1].parse::<i64>() {
            Ok(v) => idx = v,
            Err(_) => return Ok(Lset::new_invalid()),
        }
        let ele = Bytes::from(argv[2].clone());
        Ok(Lset::new(key, idx, ele))
    }

    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let response = self.lset(None).await?;
        debug!(LOGGER, "res, {} -> {}, {:?}", dst.local_addr(), dst.peer_addr(), response);
        dst.write_frame(&response).await?;

        Ok(())
    }

    pub async fn lset(&self, txn: Option<Arc<Mutex<Transaction>>>) -> AsyncResult<Frame> {
        if !self.valid {
            return Ok(resp_invalid_arguments());
        }
        if is_use_txn_api() {
            ListCommandCtx::new(txn).do_async_txnkv_lset(&self.key, self.idx, &self.element).await
        } else {
            Ok(resp_err("not supported yet"))
        }
    }
}
