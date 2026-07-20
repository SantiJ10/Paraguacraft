//! Server List Ping minimo (protocolo post-1.7) para inferir versión de un
//! favorito sin depender de metadata guardada. Usado por `game_profiles`
//! para elegir el perfil correcto (PvP 1.8.9 vs PvP Modern 1.21.11).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct PingInfo {
    pub protocol: i32,
    #[allow(dead_code)]
    pub version_name: String,
}

fn write_varint(buf: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(stream: &mut impl Read) -> std::io::Result<i32> {
    let mut result: i32 = 0;
    let mut num_read = 0u32;
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte)?;
        let value = (byte[0] & 0x7F) as i32;
        result |= value << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "VarInt demasiado largo"));
        }
        if (byte[0] & 0x80) == 0 {
            break;
        }
    }
    Ok(result)
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    write_varint(buf, s.len() as i32);
    buf.extend_from_slice(s.as_bytes());
}

/// Hace un Server List Ping y devuelve el protocolo reportado (47 = 1.8.x,
/// 700+ = 1.19+, ~767 = 1.21.x). `None` si el servidor no respondió a tiempo.
pub fn ping(host: &str, port: u16, timeout: Duration) -> Option<PingInfo> {
    let addr = format!("{host}:{port}");
    let socket_addr = addr.to_socket_addrs().ok()?.next()?;
    let mut stream = TcpStream::connect_timeout(&socket_addr, timeout).ok()?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();

    let mut handshake_data = Vec::new();
    write_varint(&mut handshake_data, 0x00);
    write_varint(&mut handshake_data, 767);
    write_string(&mut handshake_data, host);
    handshake_data.extend_from_slice(&port.to_be_bytes());
    write_varint(&mut handshake_data, 1);

    let mut handshake_packet = Vec::new();
    write_varint(&mut handshake_packet, handshake_data.len() as i32);
    handshake_packet.extend_from_slice(&handshake_data);
    stream.write_all(&handshake_packet).ok()?;

    let mut status_packet = Vec::new();
    write_varint(&mut status_packet, 1);
    status_packet.push(0x00);
    stream.write_all(&status_packet).ok()?;

    let _len = read_varint(&mut stream).ok()?;
    let _packet_id = read_varint(&mut stream).ok()?;
    let json_len = read_varint(&mut stream).ok()?;
    if json_len <= 0 || json_len > 1_000_000 {
        return None;
    }
    let mut json_buf = vec![0u8; json_len as usize];
    stream.read_exact(&mut json_buf).ok()?;
    let json_str = String::from_utf8(json_buf).ok()?;
    let value: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    let version = value.get("version")?;
    let protocol = version.get("protocol")?.as_i64()? as i32;
    let version_name = version
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Some(PingInfo { protocol, version_name })
}
