// Copyright 2022 The Turbo Cache Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;

use buf_channel::{DropCloserReadHalf, DropCloserWriteHalf};
use bytes::{Bytes, BytesMut};
use common::DigestInfo;
use error::{Code, Error, ResultExt};
use evicting_map::{EvictingMap, LenEntry};
use prometheus_utils::{Collector, CollectorState, MetricsComponent, Registry};
use traits::{StoreTrait, UploadSizeInfo};

#[derive(Clone)]
pub struct BytesWrapper(Bytes);

impl Debug for BytesWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BytesWrapper { -- Binary data -- }")
    }
}

impl LenEntry for BytesWrapper {
    #[inline]
    fn len(&self) -> usize {
        Bytes::len(&self.0)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        Bytes::is_empty(&self.0)
    }
}

pub struct MemoryStore {
    evicting_map: EvictingMap<BytesWrapper, SystemTime>,
}

impl MemoryStore {
    pub fn new(config: &config::stores::MemoryStore) -> Self {
        let empty_policy = config::stores::EvictionPolicy::default();
        let eviction_policy = config.eviction_policy.as_ref().unwrap_or(&empty_policy);
        MemoryStore {
            evicting_map: EvictingMap::new(eviction_policy, SystemTime::now()),
        }
    }

    pub async fn remove_entry(&self, digest: &DigestInfo) -> bool {
        self.evicting_map.remove(digest).await
    }
}

#[async_trait]
impl StoreTrait for MemoryStore {
    async fn has_with_results(
        self: Pin<&Self>,
        digests: &[DigestInfo],
        results: &mut [Option<usize>],
    ) -> Result<(), Error> {
        self.evicting_map.sizes_for_keys(digests, results).await;
        Ok(())
    }

    async fn update(
        self: Pin<&Self>,
        digest: DigestInfo,
        reader: DropCloserReadHalf,
        size_info: UploadSizeInfo,
    ) -> Result<(), Error> {
        let max_size = match size_info {
            UploadSizeInfo::ExactSize(sz) => sz,
            UploadSizeInfo::MaxSize(sz) => sz,
        };
        let buffer = reader
            .collect_all_with_size_hint(max_size)
            .await
            .err_tip(|| "Failed to collect all bytes from reader in memory_store::update")?;

        // Resize our buffer if our max_size was not accurate.
        // The buffer might have reserved much more than the amount of data transferred.
        // This will ensure we use less memory for the long term stored data.
        let buffer = if buffer.len() != max_size {
            let mut new_buffer = BytesMut::with_capacity(buffer.len());
            new_buffer.extend_from_slice(&buffer[..]);
            new_buffer.freeze()
        } else {
            buffer
        };
        self.evicting_map.insert(digest, BytesWrapper(buffer)).await;
        Ok(())
    }

    async fn get_part(
        self: Pin<&Self>,
        digest: DigestInfo,
        mut writer: DropCloserWriteHalf,
        offset: usize,
        length: Option<usize>,
    ) -> Result<(), Error> {
        let value = self
            .evicting_map
            .get(&digest)
            .await
            .err_tip_with_code(|_| (Code::NotFound, format!("Hash {} not found", digest.hash_str())))?;

        let default_len = value.len() - offset;
        let length = length.unwrap_or(default_len).min(default_len);
        if length > 0 {
            writer
                .send(value.0.slice(offset..(offset + length)))
                .await
                .err_tip(|| "Failed to write data in memory store")?;
        }
        writer
            .send_eof()
            .await
            .err_tip(|| "Failed to write EOF in memory store get_part")?;
        Ok(())
    }

    fn as_any(self: Arc<Self>) -> Box<dyn std::any::Any + Send> {
        Box::new(self)
    }

    fn register_metrics(self: Arc<Self>, registry: &mut Registry) {
        registry.register_collector(Box::new(Collector::new(&self)));
    }
}

impl MetricsComponent for MemoryStore {
    fn gather_metrics(&self, c: &mut CollectorState) {
        c.publish("evicting_map", &self.evicting_map, "");
    }
}
