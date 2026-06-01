#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub seq_num: u32,
    pub total_packets: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolSlice {
    pub id: String,
    pub header: Header,
    pub data: Vec<u8>,
    pub is_parity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketBatch {
    pub total_packets: u32,
    pub packets: Vec<ProtocolSlice>,
}

pub const MTU_PAYLOAD: usize = 1460;
const MAX_LEAF_PAYLOAD: usize = MTU_PAYLOAD / 2;

pub fn calculate_parity(left: &[u8], right: &[u8]) -> Vec<u8> {
    let max_len = left.len().max(right.len());
    let mut parity = vec![0; max_len];
    for i in 0..max_len {
        let l = if i < left.len() { left[i] } else { 0 };
        let r = if i < right.len() { right[i] } else { 0 };
        parity[i] = l ^ r;
    }
    parity
}

pub fn process_slice(id: String, data: Vec<u8>, seq: &mut u32) -> Vec<ProtocolSlice> {
    if data.len() <= MAX_LEAF_PAYLOAD {
        let slice = ProtocolSlice {
            id,
            header: Header {
                seq_num: *seq,
                total_packets: 0,
            },
            data,
            is_parity: false,
        };
        *seq += 1;
        return vec![slice];
    }

    let half = data.len() / 2;
    let left_data = data[..half].to_vec();
    let right_data = data[half..].to_vec();

    let mut slices = process_slice(format!("{}.0", id), left_data.clone(), seq);
    slices.append(&mut process_slice(format!("{}.1", id), right_data.clone(), seq));

    let parity = calculate_parity(&left_data, &right_data);
    slices.push(ProtocolSlice {
        id: format!("{}.P", id),
        header: Header {
            seq_num: *seq,
            total_packets: 0,
        },
        data: parity,
        is_parity: true,
    });
    *seq += 1;

    slices
}

pub fn packetize(id: String, data: Vec<u8>) -> PacketBatch {
    let mut seq = 0;
    let mut packets = process_slice(id, data, &mut seq);
    let total_packets = packets.len() as u32;
    for packet in &mut packets {
        packet.header.total_packets = total_packets;
    }
    PacketBatch {
        total_packets,
        packets,
    }
}

pub fn summarize_packets(packets: &[ProtocolSlice]) -> Vec<String> {
    packets
        .iter()
        .map(|packet| {
            format!(
                "ID: {:<8} | Seq: {:>2}/{} | Type: {:<6} | Size: {}B",
                packet.id,
                packet.header.seq_num,
                packet.header.total_packets,
                if packet.is_parity { "PARITY" } else { "DATA" },
                packet.data.len()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_slice_emits_parity_and_data() {
        let raw_data = vec![0xAB; 2000];
        let batch = packetize("ROOT".to_string(), raw_data);
        let packets = batch.packets;

        assert!(!packets.is_empty());
        assert!(packets.iter().any(|packet| packet.is_parity));
        assert!(packets.iter().any(|packet| packet.id == "ROOT.P"));
        assert!(packets.iter().any(|packet| packet.id.starts_with("ROOT.0.")));
        assert!(packets.iter().any(|packet| packet.id.starts_with("ROOT.1.")));
        assert!(packets.iter().all(|packet| packet.data.len() <= MTU_PAYLOAD));
        assert!(packets.iter().all(|packet| packet.header.total_packets == batch.total_packets));
    }

    #[test]
    fn calculate_parity_xors_equal_length_blocks() {
        let left = [0xDE, 0xAD, 0xBE];
        let right = [0x01, 0x02, 0x03];
        assert_eq!(calculate_parity(&left, &right), vec![0xDF, 0xAF, 0xBD]);
    }
}
