use itertools::Itertools;
use std::io;
use std::io::BufRead;

#[derive(Debug)]
enum PacketContents {
    Value(usize),
    Subpackets(Vec<Packet>),
}

#[derive(Debug)]
struct Packet {
    version: usize,
    contents: PacketContents,
}

impl Packet {
    fn from_line(line: &str) -> (Self, usize) {
        let version = usize::from_str_radix(&line[0..3], 2).unwrap();
        let packet_type = usize::from_str_radix(&line[3..6], 2).unwrap();

        if packet_type == 4 {
            let mut count = 0;
            let chunks = line[6..].chars().chunks(5);
            let nibbles = chunks
                .into_iter()
                .map(|mut chunk| {
                    let last = chunk.next().unwrap() == '0';
                    (last, usize::from_str_radix(&String::from_iter(chunk), 2).unwrap())
                })
                .inspect(|_| {
                    count += 1;
                });

            let mut value = 0;
            for (last, nibble) in nibbles {
                value = (value << 4) + nibble;
                if last {
                    break;
                }
            }

            (Packet { version, contents: PacketContents::Value(value) }, count * 5 + 6)
        } else {
            let &length_type = &line[6..].chars().next().unwrap();
            if length_type == '0' {
                let length = usize::from_str_radix(&line[7..22], 2).unwrap();
                let mut processed = 0;
                let mut subpackets = vec![];
                while processed < length {
                    let (subpacket, subpacket_length) = Self::from_line(&line[22 + processed..]);
                    subpackets.push(subpacket);
                    processed += subpacket_length;
                }
                (Packet { version, contents: PacketContents::Subpackets(subpackets) }, 22 + length)
            } else {
                let occurrences = usize::from_str_radix(&line[7..18], 2).unwrap();
                let mut processed = 0;
                let mut length_processed = 0;
                let mut subpackets = vec![];

                while processed < occurrences {
                    let (subpacket, subpacket_length) =
                        Self::from_line(&line[18 + length_processed..]);
                    subpackets.push(subpacket);
                    processed += 1;
                    length_processed += subpacket_length;
                }
                (
                    Packet { version, contents: PacketContents::Subpackets(subpackets) },
                    18 + length_processed,
                )
            }
        }
    }
}

fn sum_versions(packets: Vec<Packet>) -> usize {
    let mut sum = 0;
    for packet in packets {
        sum += packet.version;
        match packet.contents {
            PacketContents::Value(_) => {}
            PacketContents::Subpackets(packets) => sum += sum_versions(packets),
        }
    }
    sum
}

fn main() {
    let stdin = io::stdin();
    let lines = stdin
        .lock()
        .lines()
        .flatten()
        .map(|line| {
            let line = String::from_iter(
                line.chars()
                    .map(|c| format!("{:04b}", u8::from_str_radix(&c.to_string(), 16).unwrap())),
            );
            let (packet, _) = Packet::from_line(&line);
            packet
        })
        .collect_vec();

    println!("{}", sum_versions(lines));
}
