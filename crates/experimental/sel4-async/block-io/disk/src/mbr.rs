//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::mem;

use gpt_disk_types::{MasterBootRecord, MbrPartitionRecord};

use sel4_async_block_io::{
    BlockIOLayout, PartialBlockIO, Partition, access::ReadAccess, read_bytes,
};

use crate::Disk;

pub mod os_indicator {
    pub const FREE: u8 = 0x00;
    pub const FAT32: u8 = 0x0c;
}

impl<T> Disk<T> {
    pub async fn read_mbr<A: ReadAccess>(&self) -> Result<Option<MasterBootRecord>, T::Error>
    where
        T: PartialBlockIO<A>,
    {
        let mut buf = [0; mem::size_of::<MasterBootRecord>()];
        read_bytes(&self.io, 0, &mut buf[..]).await?;
        let mbr = *bytemuck::from_bytes::<MasterBootRecord>(&buf[..]);
        Ok(match mbr.signature {
            [0x55, 0xaa] => Some(mbr),
            _ => None,
        })
    }

    pub fn partition_mbr(self, entry: &MbrPartitionRecord) -> Partition<T>
    where
        T: BlockIOLayout,
    {
        let start: u64 = entry.starting_lba.to_u32().into();
        let size = entry.size_in_lba.to_u32().into();
        let range = start..start.checked_add(size).unwrap();
        Partition::new(self.io, range)
    }
}
