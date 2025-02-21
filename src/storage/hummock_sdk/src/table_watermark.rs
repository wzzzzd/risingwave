// Copyright 2024 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::{btree_map, BTreeMap, HashMap, HashSet};
use std::mem::size_of;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::sync::Arc;

use bytes::Bytes;
use risingwave_common::buffer::{Bitmap, BitmapBuilder};
use risingwave_common::catalog::TableId;
use risingwave_common::estimate_size::EstimateSize;
use risingwave_common::hash::{VirtualNode, VnodeBitmapExt};
use risingwave_pb::hummock::table_watermarks::PbEpochNewWatermarks;
use risingwave_pb::hummock::{PbTableWatermarks, PbVnodeWatermark};
use tracing::{debug, warn};

use crate::key::{prefix_slice_with_vnode, vnode_range, TableKey, TableKeyRange};
use crate::HummockEpoch;

#[derive(Clone)]
pub struct ReadTableWatermark {
    pub direction: WatermarkDirection,
    pub vnode_watermarks: BTreeMap<VirtualNode, Bytes>,
}

impl ReadTableWatermark {
    pub fn merge_multiple(mut watermarks: Vec<ReadTableWatermark>) -> Option<ReadTableWatermark> {
        fn merge_other(this: &mut ReadTableWatermark, other: ReadTableWatermark) {
            assert_eq!(this.direction, other.direction);
            for (vnode, watermark) in other.vnode_watermarks {
                match this.vnode_watermarks.entry(vnode) {
                    btree_map::Entry::Vacant(entry) => {
                        entry.insert(watermark);
                    }
                    btree_map::Entry::Occupied(mut entry) => {
                        let prev_watermark = entry.get();
                        let overwrite = match this.direction {
                            WatermarkDirection::Ascending => watermark > prev_watermark,
                            WatermarkDirection::Descending => watermark < prev_watermark,
                        };
                        if overwrite {
                            entry.insert(watermark);
                        }
                    }
                }
            }
        }
        let mut ret = watermarks.pop()?;
        while let Some(watermark) = watermarks.pop() {
            merge_other(&mut ret, watermark);
        }
        Some(ret)
    }
}

#[derive(Clone)]
pub struct TableWatermarksIndex {
    watermark_direction: WatermarkDirection,
    index: HashMap<VirtualNode, BTreeMap<HummockEpoch, Bytes>>,
    latest_epoch: HummockEpoch,
    committed_epoch: HummockEpoch,
}

impl TableWatermarksIndex {
    pub fn new(watermark_direction: WatermarkDirection, committed_epoch: HummockEpoch) -> Self {
        Self {
            watermark_direction,
            index: Default::default(),
            latest_epoch: committed_epoch,
            committed_epoch,
        }
    }

    pub fn index(&self) -> &HashMap<VirtualNode, BTreeMap<HummockEpoch, Bytes>> {
        &self.index
    }

    pub fn read_watermark(&self, vnode: VirtualNode, epoch: HummockEpoch) -> Option<Bytes> {
        self.index.get(&vnode).and_then(|epoch_watermarks| {
            epoch_watermarks
                .upper_bound(Included(&epoch))
                .value()
                .cloned()
        })
    }

    pub fn latest_watermark(&self, vnode: VirtualNode) -> Option<Bytes> {
        self.read_watermark(vnode, HummockEpoch::MAX)
    }

    pub fn range_watermarks(
        &self,
        epoch: HummockEpoch,
        key_range: &mut TableKeyRange,
    ) -> Option<ReadTableWatermark> {
        let mut ret = BTreeMap::new();
        let (left, right) = vnode_range(key_range);
        if right - left == 1 {
            // the table key range falls in a single vnode. No table watermark will be returned, and instead the key range
            // will be modified.
            let vnode = VirtualNode::from_index(left);
            if let Some(watermark) = self.read_watermark(vnode, epoch) {
                match self.watermark_direction {
                    WatermarkDirection::Ascending => {
                        let overwrite_start_key = match &key_range.0 {
                            Included(start_key) | Excluded(start_key) => {
                                start_key.key_part() < watermark
                            }
                            Unbounded => true,
                        };
                        if overwrite_start_key {
                            let watermark_key =
                                TableKey(prefix_slice_with_vnode(vnode, &watermark));
                            let fully_filtered = match &key_range.1 {
                                Included(end_key) => end_key < &watermark_key,
                                Excluded(end_key) => end_key <= &watermark_key,
                                Unbounded => false,
                            };
                            if fully_filtered {
                                key_range.1 = Excluded(watermark_key.clone());
                            }
                            key_range.0 = Included(watermark_key);
                        }
                    }
                    WatermarkDirection::Descending => {
                        let overwrite_end_key = match &key_range.1 {
                            Included(end_key) | Excluded(end_key) => end_key.key_part() > watermark,
                            Unbounded => true,
                        };
                        if overwrite_end_key {
                            let watermark_key =
                                TableKey(prefix_slice_with_vnode(vnode, &watermark));
                            let fully_filtered = match &key_range.0 {
                                Included(start_key) => start_key > &watermark_key,
                                Excluded(start_key) => start_key >= &watermark_key,
                                Unbounded => false,
                            };
                            if fully_filtered {
                                *key_range =
                                    (Included(watermark_key.clone()), Excluded(watermark_key));
                            } else {
                                key_range.1 = Included(watermark_key);
                            }
                        }
                    }
                }
            }
            None
        } else {
            for i in left..right {
                let vnode = VirtualNode::from_index(i);
                if let Some(watermark) = self.read_watermark(vnode, epoch) {
                    assert!(ret.insert(vnode, watermark).is_none());
                }
            }
            if ret.is_empty() {
                None
            } else {
                Some(ReadTableWatermark {
                    direction: self.direction(),
                    vnode_watermarks: ret,
                })
            }
        }
    }

    pub fn filter_regress_watermarks(&self, watermarks: &mut Vec<VnodeWatermark>) {
        let mut ret = Vec::with_capacity(watermarks.len());
        for watermark in watermarks.drain(..) {
            let mut regress_vnodes = None;
            for vnode in watermark.vnode_bitmap.iter_vnodes() {
                if let Some(prev_watermark) = self.latest_watermark(vnode) {
                    let is_regress = match self.direction() {
                        WatermarkDirection::Ascending => prev_watermark > watermark.watermark,
                        WatermarkDirection::Descending => prev_watermark < watermark.watermark,
                    };
                    if is_regress {
                        warn!(
                            "table watermark regress: {:?} {} {:?} {:?}",
                            self.direction(),
                            vnode,
                            watermark.watermark,
                            prev_watermark
                        );
                        regress_vnodes
                            .get_or_insert_with(|| BitmapBuilder::zeroed(VirtualNode::COUNT))
                            .set(vnode.to_index(), true);
                    }
                }
            }
            if let Some(regress_vnodes) = regress_vnodes {
                let mut bitmap_builder = None;
                for vnode in watermark.vnode_bitmap.iter_vnodes() {
                    let vnode_index = vnode.to_index();
                    if !regress_vnodes.is_set(vnode_index) {
                        bitmap_builder
                            .get_or_insert_with(|| BitmapBuilder::zeroed(VirtualNode::COUNT))
                            .set(vnode_index, true);
                    }
                }
                if let Some(bitmap_builder) = bitmap_builder {
                    ret.push(VnodeWatermark::new(
                        Arc::new(bitmap_builder.finish()),
                        watermark.watermark,
                    ));
                }
            } else {
                // no vnode has regress watermark
                ret.push(watermark);
            }
        }
        *watermarks = ret;
    }

    pub fn direction(&self) -> WatermarkDirection {
        self.watermark_direction
    }

    pub fn add_epoch_watermark(
        &mut self,
        epoch: HummockEpoch,
        vnode_watermark_list: &Vec<VnodeWatermark>,
        direction: WatermarkDirection,
    ) {
        assert!(epoch > self.latest_epoch);
        assert_eq!(self.watermark_direction, direction);
        self.latest_epoch = epoch;
        for vnode_watermark in vnode_watermark_list {
            for vnode in vnode_watermark.vnode_bitmap.iter_vnodes() {
                let epoch_watermarks = self.index.entry(vnode).or_default();
                if let Some((prev_epoch, prev_watermark)) = epoch_watermarks.last_key_value() {
                    assert!(*prev_epoch < epoch);
                    match self.watermark_direction {
                        WatermarkDirection::Ascending => {
                            assert!(vnode_watermark.watermark >= prev_watermark);
                        }
                        WatermarkDirection::Descending => {
                            assert!(vnode_watermark.watermark <= prev_watermark);
                        }
                    };
                };
                assert!(self
                    .index
                    .entry(vnode)
                    .or_default()
                    .insert(epoch, vnode_watermark.watermark.clone())
                    .is_none());
            }
        }
    }

    pub fn apply_committed_watermarks(&mut self, committed_index: &TableWatermarksIndex) {
        self.committed_epoch = committed_index.committed_epoch;
        for (vnode, committed_epoch_watermark) in &committed_index.index {
            let epoch_watermark = self.index.entry(*vnode).or_default();
            // keep only watermark higher than committed epoch
            *epoch_watermark = epoch_watermark.split_off(&committed_index.committed_epoch);
            for (epoch, watermark) in committed_epoch_watermark {
                epoch_watermark.insert(*epoch, watermark.clone());
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WatermarkDirection {
    Ascending,
    Descending,
}

impl WatermarkDirection {
    pub fn filter_by_watermark(&self, key: impl AsRef<[u8]>, watermark: impl AsRef<[u8]>) -> bool {
        let key = key.as_ref();
        let watermark = watermark.as_ref();
        match self {
            WatermarkDirection::Ascending => key < watermark,
            WatermarkDirection::Descending => key > watermark,
        }
    }

    pub fn is_ascending(&self) -> bool {
        match self {
            WatermarkDirection::Ascending => true,
            WatermarkDirection::Descending => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, EstimateSize)]
pub struct VnodeWatermark {
    vnode_bitmap: Arc<Bitmap>,
    watermark: Bytes,
}

impl VnodeWatermark {
    pub fn new(vnode_bitmap: Arc<Bitmap>, watermark: Bytes) -> Self {
        Self {
            vnode_bitmap,
            watermark,
        }
    }

    pub fn to_protobuf(&self) -> PbVnodeWatermark {
        PbVnodeWatermark {
            watermark: self.watermark.to_vec(),
            vnode_bitmap: Some(self.vnode_bitmap.to_protobuf()),
        }
    }

    pub fn from_protobuf(pb: &PbVnodeWatermark) -> Self {
        Self {
            vnode_bitmap: Arc::new(Bitmap::from(pb.vnode_bitmap.as_ref().unwrap())),
            watermark: Bytes::from(pb.watermark.clone()),
        }
    }

    pub fn vnode_bitmap(&self) -> &Bitmap {
        &self.vnode_bitmap
    }

    pub fn watermark(&self) -> &Bytes {
        &self.watermark
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableWatermarks {
    // later epoch at the back
    pub(crate) watermarks: Vec<(HummockEpoch, Vec<VnodeWatermark>)>,
    pub(crate) direction: WatermarkDirection,
}

impl TableWatermarks {
    pub fn single_epoch(
        epoch: HummockEpoch,
        watermarks: Vec<VnodeWatermark>,
        direction: WatermarkDirection,
    ) -> Self {
        Self {
            direction,
            watermarks: vec![(epoch, watermarks)],
        }
    }

    pub fn to_protobuf(&self) -> PbTableWatermarks {
        PbTableWatermarks {
            epoch_watermarks: self
                .watermarks
                .iter()
                .map(|(epoch, watermarks)| PbEpochNewWatermarks {
                    watermarks: watermarks.iter().map(VnodeWatermark::to_protobuf).collect(),
                    epoch: *epoch,
                })
                .collect(),
            is_ascending: match self.direction {
                WatermarkDirection::Ascending => true,
                WatermarkDirection::Descending => false,
            },
        }
    }

    pub fn estimated_encode_len(&self) -> usize {
        self.watermarks.len() * size_of::<HummockEpoch>()
            + self
                .watermarks
                .iter()
                .map(|(_, watermarks)| {
                    watermarks
                        .iter()
                        .map(|watermark| watermark.estimated_size())
                        .sum::<usize>()
                })
                .sum::<usize>()
            + size_of::<bool>() // for direction
    }

    pub fn add_new_epoch_watermarks(
        &mut self,
        epoch: HummockEpoch,
        watermarks: Vec<VnodeWatermark>,
        direction: WatermarkDirection,
    ) {
        assert_eq!(self.direction, direction);
        if let Some((prev_epoch, _)) = self.watermarks.last() {
            assert!(*prev_epoch < epoch);
        }
        self.watermarks.push((epoch, watermarks));
    }

    pub fn from_protobuf(pb: &PbTableWatermarks) -> Self {
        Self {
            watermarks: pb
                .epoch_watermarks
                .iter()
                .map(|epoch_watermark| {
                    let epoch = epoch_watermark.epoch;
                    let watermarks = epoch_watermark
                        .watermarks
                        .iter()
                        .map(VnodeWatermark::from_protobuf)
                        .collect();
                    (epoch, watermarks)
                })
                .collect(),
            direction: if pb.is_ascending {
                WatermarkDirection::Ascending
            } else {
                WatermarkDirection::Descending
            },
        }
    }

    pub fn build_index(&self, committed_epoch: HummockEpoch) -> TableWatermarksIndex {
        let mut ret = TableWatermarksIndex {
            index: HashMap::new(),
            watermark_direction: self.direction,
            latest_epoch: HummockEpoch::MIN,
            committed_epoch: HummockEpoch::MIN,
        };
        for (epoch, vnode_watermark_list) in &self.watermarks {
            ret.add_epoch_watermark(*epoch, vnode_watermark_list, self.direction);
        }
        ret.committed_epoch = committed_epoch;
        ret
    }
}

pub fn merge_multiple_new_table_watermarks(
    table_watermarks_list: impl IntoIterator<Item = HashMap<TableId, TableWatermarks>>,
) -> HashMap<TableId, TableWatermarks> {
    let mut ret: HashMap<TableId, (WatermarkDirection, BTreeMap<u64, Vec<VnodeWatermark>>)> =
        HashMap::new();
    for table_watermarks in table_watermarks_list {
        for (table_id, new_table_watermarks) in table_watermarks {
            let epoch_watermarks = match ret.entry(table_id) {
                Entry::Occupied(entry) => {
                    let (direction, epoch_watermarks) = entry.into_mut();
                    assert_eq!(&new_table_watermarks.direction, direction);
                    epoch_watermarks
                }
                Entry::Vacant(entry) => {
                    let (_, epoch_watermarks) =
                        entry.insert((new_table_watermarks.direction, BTreeMap::new()));
                    epoch_watermarks
                }
            };
            for (new_epoch, new_epoch_watermarks) in new_table_watermarks.watermarks {
                epoch_watermarks
                    .entry(new_epoch)
                    .or_insert_with(Vec::new)
                    .extend(new_epoch_watermarks);
            }
        }
    }
    ret.into_iter()
        .map(|(table_id, (direction, epoch_watermarks))| {
            (
                table_id,
                TableWatermarks {
                    direction,
                    // ordered from earlier epoch to later epoch
                    watermarks: epoch_watermarks.into_iter().collect(),
                },
            )
        })
        .collect()
}

impl TableWatermarks {
    pub fn apply_new_table_watermarks(&mut self, newly_added_watermarks: &TableWatermarks) {
        assert_eq!(self.direction, newly_added_watermarks.direction);
        assert!(self.watermarks.iter().map(|(epoch, _)| epoch).is_sorted());
        assert!(newly_added_watermarks
            .watermarks
            .iter()
            .map(|(epoch, _)| epoch)
            .is_sorted());
        // ensure that the newly added watermarks have a later epoch than the previous latest epoch.
        if let Some((prev_last_epoch, _)) = self.watermarks.last()
            && let Some((new_first_epoch, _)) = newly_added_watermarks.watermarks.first()
        {
            assert!(prev_last_epoch < new_first_epoch);
        }
        self.watermarks.extend(
            newly_added_watermarks
                .watermarks
                .iter()
                .map(|(epoch, new_watermarks)| (*epoch, new_watermarks.clone())),
        );
    }

    pub fn clear_stale_epoch_watermark(&mut self, safe_epoch: u64) {
        match self.watermarks.first() {
            None => {
                // return on empty watermark
                return;
            }
            Some((earliest_epoch, _)) => {
                if *earliest_epoch >= safe_epoch {
                    // No stale epoch watermark needs to be cleared.
                    return;
                }
            }
        }
        debug!("clear stale table watermark below epoch {}", safe_epoch);
        let mut result_epoch_watermark = Vec::with_capacity(self.watermarks.len());
        let mut unset_vnode: HashSet<VirtualNode> = (0..VirtualNode::COUNT)
            .map(VirtualNode::from_index)
            .collect();
        while let Some((epoch, _)) = self.watermarks.last() {
            if *epoch >= safe_epoch {
                let (epoch, watermarks) = self.watermarks.pop().expect("have check Some");
                for watermark in &watermarks {
                    for vnode in watermark.vnode_bitmap.iter_vnodes() {
                        unset_vnode.remove(&vnode);
                    }
                }
                result_epoch_watermark.push((epoch, watermarks));
            } else {
                break;
            }
        }
        while !unset_vnode.is_empty()
            && let Some((_, watermarks)) = self.watermarks.pop()
        {
            let mut new_vnode_watermarks = Vec::new();
            for vnode_watermark in watermarks {
                let mut set_vnode = Vec::new();
                for vnode in vnode_watermark.vnode_bitmap.iter_vnodes() {
                    if unset_vnode.remove(&vnode) {
                        set_vnode.push(vnode);
                    }
                }
                if !set_vnode.is_empty() {
                    let mut builder = BitmapBuilder::zeroed(VirtualNode::COUNT);
                    for vnode in set_vnode {
                        builder.set(vnode.to_index(), true);
                    }
                    let bitmap = Arc::new(builder.finish());
                    new_vnode_watermarks.push(VnodeWatermark {
                        vnode_bitmap: bitmap,
                        watermark: vnode_watermark.watermark,
                    })
                }
            }
            if !new_vnode_watermarks.is_empty() {
                if let Some((last_epoch, last_watermarks)) = result_epoch_watermark.last_mut()
                    && *last_epoch == safe_epoch
                {
                    last_watermarks.extend(new_vnode_watermarks);
                } else {
                    result_epoch_watermark.push((safe_epoch, new_vnode_watermarks));
                }
            }
        }
        // epoch watermark are added from later epoch to earlier epoch.
        // reverse to ensure that earlier epochs are at the front
        result_epoch_watermark.reverse();
        assert!(
            result_epoch_watermark.is_sorted_by(|(first_epoch, _), (second_epoch, _)| {
                let ret = first_epoch.cmp(second_epoch);
                assert_ne!(ret, Ordering::Equal);
                Some(ret)
            })
        );
        *self = TableWatermarks {
            watermarks: result_epoch_watermark,
            direction: self.direction,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::Bound::Included;
    use std::collections::{Bound, HashMap};
    use std::ops::Bound::Excluded;
    use std::sync::Arc;
    use std::vec;

    use bytes::Bytes;
    use risingwave_common::buffer::{Bitmap, BitmapBuilder};
    use risingwave_common::catalog::TableId;
    use risingwave_common::hash::VirtualNode;

    use crate::key::{
        is_empty_key_range, map_table_key_range, prefix_slice_with_vnode,
        prefixed_range_with_vnode, TableKeyRange,
    };
    use crate::table_watermark::{
        merge_multiple_new_table_watermarks, TableWatermarks, TableWatermarksIndex, VnodeWatermark,
        WatermarkDirection,
    };
    use crate::version::HummockVersion;

    fn build_bitmap(vnodes: impl IntoIterator<Item = usize>) -> Arc<Bitmap> {
        let mut builder = BitmapBuilder::zeroed(VirtualNode::COUNT);
        for vnode in vnodes {
            builder.set(vnode, true);
        }
        Arc::new(builder.finish())
    }

    #[test]
    fn test_apply_new_table_watermark() {
        let epoch1 = 233;
        let direction = WatermarkDirection::Ascending;
        let watermark1 = Bytes::from("watermark1");
        let watermark2 = Bytes::from("watermark2");
        let watermark3 = Bytes::from("watermark3");
        let watermark4 = Bytes::from("watermark4");
        let mut table_watermarks = TableWatermarks::single_epoch(
            epoch1,
            vec![VnodeWatermark::new(
                build_bitmap(vec![0, 1, 2]),
                watermark1.clone(),
            )],
            direction,
        );
        let epoch2 = epoch1 + 1;
        table_watermarks.add_new_epoch_watermarks(
            epoch2,
            vec![VnodeWatermark::new(
                build_bitmap(vec![0, 1, 2, 3]),
                watermark2.clone(),
            )],
            direction,
        );

        let mut table_watermark_checkpoint = table_watermarks.clone();

        let epoch3 = epoch2 + 1;
        let mut second_table_watermark = TableWatermarks::single_epoch(
            epoch3,
            vec![VnodeWatermark::new(
                build_bitmap(0..VirtualNode::COUNT),
                watermark3.clone(),
            )],
            direction,
        );
        table_watermarks.add_new_epoch_watermarks(
            epoch3,
            vec![VnodeWatermark::new(
                build_bitmap(0..VirtualNode::COUNT),
                watermark3.clone(),
            )],
            direction,
        );
        let epoch4 = epoch3 + 1;
        let epoch5 = epoch4 + 1;
        table_watermarks.add_new_epoch_watermarks(
            epoch5,
            vec![VnodeWatermark::new(
                build_bitmap(vec![0, 3, 4]),
                watermark4.clone(),
            )],
            direction,
        );
        second_table_watermark.add_new_epoch_watermarks(
            epoch5,
            vec![VnodeWatermark::new(
                build_bitmap(vec![0, 3, 4]),
                watermark4.clone(),
            )],
            direction,
        );

        table_watermark_checkpoint.apply_new_table_watermarks(&second_table_watermark);
        assert_eq!(table_watermarks, table_watermark_checkpoint);
    }

    #[test]
    fn test_clear_stale_epoch_watmermark() {
        let epoch1 = 233;
        let direction = WatermarkDirection::Ascending;
        let watermark1 = Bytes::from("watermark1");
        let watermark2 = Bytes::from("watermark2");
        let watermark3 = Bytes::from("watermark3");
        let watermark4 = Bytes::from("watermark4");
        let mut table_watermarks = TableWatermarks::single_epoch(
            epoch1,
            vec![VnodeWatermark::new(
                build_bitmap(vec![0, 1, 2]),
                watermark1.clone(),
            )],
            direction,
        );
        let epoch2 = epoch1 + 1;
        table_watermarks.add_new_epoch_watermarks(
            epoch2,
            vec![VnodeWatermark::new(
                build_bitmap(vec![0, 1, 2, 3]),
                watermark2.clone(),
            )],
            direction,
        );
        let epoch3 = epoch2 + 1;
        table_watermarks.add_new_epoch_watermarks(
            epoch3,
            vec![VnodeWatermark::new(
                build_bitmap(0..VirtualNode::COUNT),
                watermark3.clone(),
            )],
            direction,
        );
        let epoch4 = epoch3 + 1;
        let epoch5 = epoch4 + 1;
        table_watermarks.add_new_epoch_watermarks(
            epoch5,
            vec![VnodeWatermark::new(
                build_bitmap(vec![0, 3, 4]),
                watermark4.clone(),
            )],
            direction,
        );

        let mut table_watermarks_checkpoint = table_watermarks.clone();
        table_watermarks_checkpoint.clear_stale_epoch_watermark(epoch1);
        assert_eq!(table_watermarks_checkpoint, table_watermarks);

        table_watermarks_checkpoint.clear_stale_epoch_watermark(epoch2);
        assert_eq!(
            table_watermarks_checkpoint,
            TableWatermarks {
                watermarks: vec![
                    (
                        epoch2,
                        vec![VnodeWatermark::new(
                            build_bitmap(vec![0, 1, 2, 3]),
                            watermark2.clone(),
                        )]
                    ),
                    (
                        epoch3,
                        vec![VnodeWatermark::new(
                            build_bitmap(0..VirtualNode::COUNT),
                            watermark3.clone(),
                        )]
                    ),
                    (
                        epoch5,
                        vec![VnodeWatermark::new(
                            build_bitmap(vec![0, 3, 4]),
                            watermark4.clone(),
                        )]
                    )
                ],
                direction,
            }
        );

        table_watermarks_checkpoint.clear_stale_epoch_watermark(epoch3);
        assert_eq!(
            table_watermarks_checkpoint,
            TableWatermarks {
                watermarks: vec![
                    (
                        epoch3,
                        vec![VnodeWatermark::new(
                            build_bitmap(0..VirtualNode::COUNT),
                            watermark3.clone(),
                        )]
                    ),
                    (
                        epoch5,
                        vec![VnodeWatermark::new(
                            build_bitmap(vec![0, 3, 4]),
                            watermark4.clone(),
                        )]
                    )
                ],
                direction,
            }
        );

        table_watermarks_checkpoint.clear_stale_epoch_watermark(epoch4);
        assert_eq!(
            table_watermarks_checkpoint,
            TableWatermarks {
                watermarks: vec![
                    (
                        epoch4,
                        vec![VnodeWatermark::new(
                            build_bitmap((1..3).chain(5..VirtualNode::COUNT)),
                            watermark3.clone()
                        )]
                    ),
                    (
                        epoch5,
                        vec![VnodeWatermark::new(
                            build_bitmap(vec![0, 3, 4]),
                            watermark4.clone(),
                        )]
                    )
                ],
                direction,
            }
        );

        table_watermarks_checkpoint.clear_stale_epoch_watermark(epoch5);
        assert_eq!(
            table_watermarks_checkpoint,
            TableWatermarks {
                watermarks: vec![(
                    epoch5,
                    vec![
                        VnodeWatermark::new(build_bitmap(vec![0, 3, 4]), watermark4.clone()),
                        VnodeWatermark::new(
                            build_bitmap((1..3).chain(5..VirtualNode::COUNT)),
                            watermark3.clone()
                        )
                    ]
                )],
                direction,
            }
        );
    }

    #[test]
    fn test_merge_multiple_new_table_watermarks() {
        fn epoch_new_watermark(epoch: u64, bitmaps: Vec<&Bitmap>) -> (u64, Vec<VnodeWatermark>) {
            (
                epoch,
                bitmaps
                    .into_iter()
                    .map(|bitmap| VnodeWatermark {
                        watermark: Bytes::from(vec![1, 2, epoch as _]),
                        vnode_bitmap: Arc::new(bitmap.clone()),
                    })
                    .collect(),
            )
        }
        fn build_table_watermark(
            vnodes: impl IntoIterator<Item = usize>,
            epochs: impl IntoIterator<Item = u64>,
        ) -> TableWatermarks {
            let bitmap = build_bitmap(vnodes);
            TableWatermarks {
                watermarks: epochs
                    .into_iter()
                    .map(|epoch: u64| epoch_new_watermark(epoch, vec![&bitmap]))
                    .collect(),
                direction: WatermarkDirection::Ascending,
            }
        }
        let table1_watermark1 = build_table_watermark(0..3, vec![1, 2, 4]);
        let table1_watermark2 = build_table_watermark(4..6, vec![1, 2, 5]);
        let table2_watermark = build_table_watermark(0..4, 1..3);
        let table3_watermark = build_table_watermark(0..4, 3..5);
        let mut first = HashMap::new();
        first.insert(TableId::new(1), table1_watermark1);
        first.insert(TableId::new(2), table2_watermark.clone());
        let mut second = HashMap::new();
        second.insert(TableId::new(1), table1_watermark2);
        second.insert(TableId::new(3), table3_watermark.clone());
        let result = merge_multiple_new_table_watermarks(vec![first, second]);
        let mut expected = HashMap::new();
        expected.insert(
            TableId::new(1),
            TableWatermarks {
                watermarks: vec![
                    epoch_new_watermark(1, vec![&build_bitmap(0..3), &build_bitmap(4..6)]),
                    epoch_new_watermark(2, vec![&build_bitmap(0..3), &build_bitmap(4..6)]),
                    epoch_new_watermark(4, vec![&build_bitmap(0..3)]),
                    epoch_new_watermark(5, vec![&build_bitmap(4..6)]),
                ],
                direction: WatermarkDirection::Ascending,
            },
        );
        expected.insert(TableId::new(2), table2_watermark);
        expected.insert(TableId::new(3), table3_watermark);
        assert_eq!(result, expected);
    }

    const COMMITTED_EPOCH: u64 = 233;
    const EPOCH1: u64 = COMMITTED_EPOCH + 1;
    const EPOCH2: u64 = EPOCH1 + 1;
    const TEST_SINGLE_VNODE: VirtualNode = VirtualNode::from_index(1);

    fn build_watermark_range(
        direction: WatermarkDirection,
        (low, high): (Bound<Bytes>, Bound<Bytes>),
    ) -> TableKeyRange {
        let range = match direction {
            WatermarkDirection::Ascending => (low, high),
            WatermarkDirection::Descending => (high, low),
        };
        prefixed_range_with_vnode(range, TEST_SINGLE_VNODE)
    }

    /// Build and return a watermark index with the following watermarks
    /// EPOCH1 bitmap(0, 1, 2, 3) watermark1
    /// EPOCH2 bitmap(1, 2, 3, 4) watermark2
    fn build_and_test_watermark_index(
        direction: WatermarkDirection,
        watermark1: Bytes,
        watermark2: Bytes,
        watermark3: Bytes,
    ) -> TableWatermarksIndex {
        let mut index = TableWatermarksIndex::new(direction, COMMITTED_EPOCH);
        index.add_epoch_watermark(
            EPOCH1,
            &vec![VnodeWatermark::new(build_bitmap(0..4), watermark1.clone())],
            direction,
        );
        index.add_epoch_watermark(
            EPOCH2,
            &vec![VnodeWatermark::new(build_bitmap(1..5), watermark2.clone())],
            direction,
        );

        assert_eq!(
            index.read_watermark(VirtualNode::from_index(0), EPOCH1),
            Some(watermark1.clone())
        );
        assert_eq!(
            index.read_watermark(VirtualNode::from_index(1), EPOCH1),
            Some(watermark1.clone())
        );
        assert_eq!(
            index.read_watermark(VirtualNode::from_index(4), EPOCH1),
            None
        );
        assert_eq!(
            index.read_watermark(VirtualNode::from_index(0), EPOCH2),
            Some(watermark1.clone())
        );
        assert_eq!(
            index.read_watermark(VirtualNode::from_index(1), EPOCH2),
            Some(watermark2.clone())
        );
        assert_eq!(
            index.read_watermark(VirtualNode::from_index(4), EPOCH2),
            Some(watermark2.clone())
        );
        assert_eq!(
            index.latest_watermark(VirtualNode::from_index(0)),
            Some(watermark1.clone())
        );
        assert_eq!(
            index.latest_watermark(VirtualNode::from_index(1)),
            Some(watermark2.clone())
        );
        assert_eq!(
            index.latest_watermark(VirtualNode::from_index(4)),
            Some(watermark2.clone())
        );

        // test read from multiple vnodes
        {
            let range = map_table_key_range((
                Included(prefix_slice_with_vnode(
                    VirtualNode::from_index(1),
                    b"begin",
                )),
                Excluded(prefix_slice_with_vnode(VirtualNode::from_index(2), b"end")),
            ));
            let mut range_mut = range.clone();
            let read_watermarks = index.range_watermarks(EPOCH2, &mut range_mut).unwrap();
            assert_eq!(range_mut, range);
            assert_eq!(direction, read_watermarks.direction);
            assert_eq!(2, read_watermarks.vnode_watermarks.len());
            assert_eq!(
                &watermark2,
                read_watermarks
                    .vnode_watermarks
                    .get(&VirtualNode::from_index(1))
                    .unwrap()
            );
            assert_eq!(
                &watermark2,
                read_watermarks
                    .vnode_watermarks
                    .get(&VirtualNode::from_index(2))
                    .unwrap()
            );
        }

        // watermark is watermark2
        let check_watermark_range =
            |query_range: (Bound<Bytes>, Bound<Bytes>),
             output_range: Option<(Bound<Bytes>, Bound<Bytes>)>| {
                let mut range = build_watermark_range(direction, query_range);
                assert!(index.range_watermarks(EPOCH2, &mut range).is_none());
                if let Some(output_range) = output_range {
                    assert_eq!(range, build_watermark_range(direction, output_range));
                } else {
                    assert!(is_empty_key_range(&range));
                }
            };

        // test read from single vnode and truncate begin key range
        check_watermark_range(
            (Included(watermark1.clone()), Excluded(watermark3.clone())),
            Some((Included(watermark2.clone()), Excluded(watermark3.clone()))),
        );

        // test read from single vnode and begin key right at watermark
        check_watermark_range(
            (Included(watermark2.clone()), Excluded(watermark3.clone())),
            Some((Included(watermark2.clone()), Excluded(watermark3.clone()))),
        );
        check_watermark_range(
            (Excluded(watermark2.clone()), Excluded(watermark3.clone())),
            Some((Excluded(watermark2.clone()), Excluded(watermark3.clone()))),
        );

        // test read from single vnode and end key right at watermark
        check_watermark_range(
            (Excluded(watermark1.clone()), Excluded(watermark2.clone())),
            None,
        );
        check_watermark_range(
            (Excluded(watermark1.clone()), Included(watermark2.clone())),
            Some((Included(watermark2.clone()), Included(watermark2.clone()))),
        );

        index
    }

    #[test]
    fn test_watermark_index_ascending() {
        let watermark1 = Bytes::from_static(b"watermark1");
        let watermark2 = Bytes::from_static(b"watermark2");
        let watermark3 = Bytes::from_static(b"watermark3");
        build_and_test_watermark_index(
            WatermarkDirection::Ascending,
            watermark1.clone(),
            watermark2.clone(),
            watermark3.clone(),
        );
    }

    #[test]
    fn test_watermark_index_descending() {
        let watermark1 = Bytes::from_static(b"watermark254");
        let watermark2 = Bytes::from_static(b"watermark253");
        let watermark3 = Bytes::from_static(b"watermark252");
        build_and_test_watermark_index(
            WatermarkDirection::Descending,
            watermark1.clone(),
            watermark2.clone(),
            watermark3.clone(),
        );
    }

    #[test]
    fn test_apply_committed_index() {
        let watermark1 = Bytes::from_static(b"watermark1");
        let watermark2 = Bytes::from_static(b"watermark2");
        let watermark3 = Bytes::from_static(b"watermark3");
        let mut index = build_and_test_watermark_index(
            WatermarkDirection::Ascending,
            watermark1.clone(),
            watermark2.clone(),
            watermark3.clone(),
        );

        let mut version = HummockVersion {
            max_committed_epoch: EPOCH1,
            safe_epoch: EPOCH1,
            ..Default::default()
        };
        let test_table_id = TableId::from(233);
        version.table_watermarks.insert(
            test_table_id,
            TableWatermarks {
                watermarks: vec![(
                    EPOCH1,
                    vec![VnodeWatermark {
                        watermark: watermark1.clone(),
                        vnode_bitmap: build_bitmap(0..VirtualNode::COUNT),
                    }],
                )],
                direction: WatermarkDirection::Ascending,
            },
        );
        let committed_index = version
            .build_table_watermarks_index()
            .remove(&test_table_id)
            .unwrap();
        index.apply_committed_watermarks(&committed_index);
        assert_eq!(EPOCH1, index.committed_epoch);
        assert_eq!(EPOCH2, index.latest_epoch);
        for vnode in 0..VirtualNode::COUNT {
            let vnode = VirtualNode::from_index(vnode);
            let epoch_watermark = index.index.get(&vnode).unwrap();
            if (1..5).contains(&vnode.to_index()) {
                assert_eq!(2, epoch_watermark.len());
                assert_eq!(&watermark1, epoch_watermark.get(&EPOCH1).unwrap());
                assert_eq!(&watermark2, epoch_watermark.get(&EPOCH2).unwrap());
            } else {
                assert_eq!(1, epoch_watermark.len());
                assert_eq!(&watermark1, epoch_watermark.get(&EPOCH1).unwrap());
            }
        }
    }

    #[test]
    fn test_filter_regress_watermark() {
        let watermark1 = Bytes::from_static(b"watermark1");
        let watermark2 = Bytes::from_static(b"watermark2");
        let watermark3 = Bytes::from_static(b"watermark3");
        let index = build_and_test_watermark_index(
            WatermarkDirection::Ascending,
            watermark1.clone(),
            watermark2.clone(),
            watermark3.clone(),
        );

        let mut new_watermarks = vec![
            // Partial regress
            VnodeWatermark {
                vnode_bitmap: build_bitmap(0..2),
                watermark: watermark1.clone(),
            },
            // All not regress
            VnodeWatermark {
                vnode_bitmap: build_bitmap(2..4),
                watermark: watermark3.clone(),
            },
            // All regress
            VnodeWatermark {
                vnode_bitmap: build_bitmap(4..5),
                watermark: watermark1.clone(),
            },
            // All newly set vnode
            VnodeWatermark {
                vnode_bitmap: build_bitmap(5..6),
                watermark: watermark3.clone(),
            },
        ];

        index.filter_regress_watermarks(&mut new_watermarks);

        assert_eq!(
            new_watermarks,
            vec![
                VnodeWatermark {
                    vnode_bitmap: build_bitmap(0..1),
                    watermark: watermark1,
                },
                VnodeWatermark {
                    vnode_bitmap: build_bitmap(2..4),
                    watermark: watermark3.clone(),
                },
                VnodeWatermark {
                    vnode_bitmap: build_bitmap(5..6),
                    watermark: watermark3,
                },
            ]
        );
    }
}
