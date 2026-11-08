//! Power-fail-safe storage for one config blob in the `nvs` flash partition.
//!
//! Two 4 KiB sectors at the start of the partition are used as A/B slots. Each
//! slot holds a header (magic, sequence number, length, CRC-32) followed by the
//! JSON data. A write goes to the slot that does not hold the current config,
//! data first and header last, so an interrupted write leaves the previous
//! config intact. On read the valid slot with the highest sequence number wins.

use alloc::vec;
use alloc::vec::Vec;

use esp_bootloader_esp_idf::partitions::{
    self, DataPartitionSubType, PARTITION_TABLE_MAX_LEN, PartitionType,
};
use esp_storage::FlashStorage;

use crate::error::{Error, Result};

const SECTOR: u32 = FlashStorage::SECTOR_SIZE;
const MAGIC: [u8; 4] = *b"GHC1";
const HEADER_LEN: usize = 16;
/// Largest config (serialized JSON) that fits into one slot.
pub const MAX_DATA_LEN: usize = SECTOR as usize - HEADER_LEN;

pub(crate) struct ConfigStore {
    flash: FlashStorage<'static>,
    /// Absolute flash offset of slot A; slot B follows one sector later.
    base: u32,
    /// Slot index and sequence number of the current config, if any.
    current: Option<(u32, u32)>,
}

impl ConfigStore {
    pub(crate) fn new(mut flash: FlashStorage<'static>) -> Result<Self> {
        let mut table = [0u8; PARTITION_TABLE_MAX_LEN];
        let pt = partitions::read_partition_table(&mut flash, &mut table)
            .map_err(|_| Error::MissingConfigPartition)?;
        let nvs = pt
            .find_partition(PartitionType::Data(DataPartitionSubType::Nvs))
            .map_err(|_| Error::MissingConfigPartition)?
            .ok_or(Error::MissingConfigPartition)?;
        if nvs.len() < 2 * SECTOR || nvs.offset() % SECTOR != 0 {
            return Err(Error::MissingConfigPartition);
        }
        Ok(Self {
            flash,
            base: nvs.offset(),
            current: None,
        })
    }

    fn slot_offset(&self, slot: u32) -> u32 {
        self.base + slot * SECTOR
    }

    /// Returns the stored config bytes, or `None` if no valid slot exists.
    pub(crate) fn load(&mut self) -> Result<Option<Vec<u8>>> {
        let mut best: Option<(u32, u32, Vec<u8>)> = None;
        for slot in 0..2 {
            let Some((seq, data)) = self.read_slot(slot)? else {
                continue;
            };
            if best.as_ref().is_none_or(|(_, best_seq, _)| seq > *best_seq) {
                best = Some((slot, seq, data));
            }
        }
        Ok(best.map(|(slot, seq, data)| {
            self.current = Some((slot, seq));
            data
        }))
    }

    fn read_slot(&mut self, slot: u32) -> Result<Option<(u32, Vec<u8>)>> {
        let offset = self.slot_offset(slot);
        let mut header = [0u8; HEADER_LEN];
        self.flash
            .read_nor(offset, &mut header)
            .map_err(|_| Error::Storage)?;
        if header[0..4] != MAGIC {
            return Ok(None);
        }
        let seq = u32::from_le_bytes(header[4..8].try_into().unwrap());
        let len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
        let crc = u32::from_le_bytes(header[12..16].try_into().unwrap());
        if len > MAX_DATA_LEN {
            return Ok(None);
        }
        let mut data = vec![0u8; align4(len)];
        self.flash
            .read_nor(offset + HEADER_LEN as u32, &mut data)
            .map_err(|_| Error::Storage)?;
        data.truncate(len);
        Ok((crc32(&data) == crc).then_some((seq, data)))
    }

    pub(crate) fn store(&mut self, data: &[u8]) -> Result<()> {
        if data.len() > MAX_DATA_LEN {
            return Err(Error::ConfigTooLarge);
        }
        let (slot, seq) = match self.current {
            Some((slot, seq)) => (1 - slot, seq.wrapping_add(1)),
            None => (0, 1),
        };
        let offset = self.slot_offset(slot);

        self.flash
            .erase(offset, offset + SECTOR)
            .map_err(|_| Error::Storage)?;

        let mut padded = vec![0xffu8; align4(data.len())];
        padded[..data.len()].copy_from_slice(data);
        self.flash
            .write_nor(offset + HEADER_LEN as u32, &padded)
            .map_err(|_| Error::Storage)?;

        let mut header = [0u8; HEADER_LEN];
        header[0..4].copy_from_slice(&MAGIC);
        header[4..8].copy_from_slice(&seq.to_le_bytes());
        header[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
        header[12..16].copy_from_slice(&crc32(data).to_le_bytes());
        self.flash
            .write_nor(offset, &header)
            .map_err(|_| Error::Storage)?;

        self.current = Some((slot, seq));
        Ok(())
    }
}

fn align4(n: usize) -> usize {
    n.div_ceil(4) * 4
}

/// CRC-32 (IEEE 802.3), bitwise; the config is small and rarely written.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
