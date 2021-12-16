use itertools::Itertools;
use std::io;
use std::io::BufRead;

#[derive(Debug)]
enum PacketContents {
    Value(usize),
    Sum(Vec<Packet>),
    Product(Vec<Packet>),
    Minimum(Vec<Packet>),
    Maximum(Vec<Packet>),
    GreaterThan(Vec<Packet>),
    LessThan(Vec<Packet>),
    EqualTo(Vec<Packet>),
}

#[derive(Debug)]
struct Packet {
    #[allow(dead_code)]
    version: usize,
    contents: PacketContents,
}

impl Packet {
    fn from_sub_packets(version: usize, type_id: usize, sub_packets: Vec<Packet>) -> Self {
        let contents = match type_id {
            0 => PacketContents::Sum(sub_packets),
            1 => PacketContents::Product(sub_packets),
            2 => PacketContents::Minimum(sub_packets),
            3 => PacketContents::Maximum(sub_packets),
            5 => PacketContents::GreaterThan(sub_packets),
            6 => PacketContents::LessThan(sub_packets),
            7 => PacketContents::EqualTo(sub_packets),
            _ => unreachable!(),
        };

        Self { version, contents }
    }

    fn from_line(line: &str) -> (Self, usize) {
        let version = usize::from_str_radix(&line[0..3], 2).unwrap();
        let packet_type = usize::from_str_radix(&line[3..6], 2).unwrap();

        match packet_type {
            4 => {
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
            }
            _ => {
                let &length_type = &line[6..].chars().next().unwrap();
                if length_type == '0' {
                    let length = usize::from_str_radix(&line[7..22], 2).unwrap();
                    let mut processed = 0;
                    let mut subpackets = vec![];
                    while processed < length {
                        let (subpacket, subpacket_length) =
                            Self::from_line(&line[22 + processed..]);
                        subpackets.push(subpacket);
                        processed += subpacket_length;
                    }
                    (Packet::from_sub_packets(version, packet_type, subpackets), 22 + length)
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
                        Packet::from_sub_packets(version, packet_type, subpackets),
                        18 + length_processed,
                    )
                }
            }
        }
    }
}

fn get_value(packet: &Packet) -> usize {
    match &packet.contents {
        &PacketContents::Value(value) => value,
        PacketContents::Sum(packets) => packets.iter().map(get_value).sum(),
        PacketContents::Product(packets) => packets.iter().map(get_value).product(),
        PacketContents::Minimum(packets) => packets.iter().map(get_value).min().unwrap(),
        PacketContents::Maximum(packets) => packets.iter().map(get_value).max().unwrap(),
        PacketContents::GreaterThan(packets) => {
            if get_value(packets.get(0).unwrap()) > get_value(packets.get(1).unwrap()) {
                1
            } else {
                0
            }
        }
        PacketContents::LessThan(packets) => {
            if get_value(packets.get(0).unwrap()) < get_value(packets.get(1).unwrap()) {
                1
            } else {
                0
            }
        }
        PacketContents::EqualTo(packets) => {
            if get_value(packets.get(0).unwrap()) == get_value(packets.get(1).unwrap()) {
                1
            } else {
                0
            }
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines().flatten().map(|line| {
        let line = String::from_iter(
            line.chars()
                .map(|c| format!("{:04b}", u8::from_str_radix(&c.to_string(), 16).unwrap())),
        );
        let (packet, _) = Packet::from_line(&line);
        packet
    });

    println!("{}", get_value(&lines.next().unwrap()));
}
