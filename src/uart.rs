/*
    Copyright 2024 Bricky
    https://github.com/bricky149

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

extern crate serialport5;
use self::serialport5::*;

use std::io::{Read, Write};
use std::time::Duration;

const BAUD_RATE: u32 = 115200;

fn checksum(command: &mut [u8]) {
    let last_idx = command.len() - 1;
    let mut sum = 0;
    // Relies on arithmetic overflows
    for byte in command.iter().take(last_idx) {
        sum += byte
    }
    command[last_idx] = sum;
}

fn verify(command: &[u8]) -> bool {
    let last_idx = command.len() - 1;
    let mut calculated_sum = 0;
    // Relies on arithmetic overflows
    for byte in command.iter().take(last_idx) {
        calculated_sum += byte
    }
    command[last_idx] == calculated_sum
}

pub fn command_eraseflash(port: &String) -> Result<bool> {
    let mut command = [0u8; 5];
    command[0] = 0x39;
    command[3] = 0x55;
    checksum(&mut command);

    let mut port = SerialPort::builder()
        .baud_rate(BAUD_RATE)
        .read_timeout(Some(Duration::from_secs(1)))
        .open(port)
        .expect("Failed to open port");
    port.write_all(&command)?;

    let mut buffer = [0u8];
    port.read_exact(&mut buffer)?;
    match buffer {
        [0x06] => Ok(true),
        _ => Ok(false)
    }
}

pub fn command_writeflash(port: &String, offset: usize, firmware: &[u8]) -> Result<bool> {
    let mut command = [0u8; 132];
    command[0] = 0x57;
    command[1] = ((offset >> 8) & 0xFF) as u8;
    command[2] = ((offset) & 0xFF) as u8;
    let mut chunk_length = firmware.len() - offset;
    if chunk_length > 128 {
        chunk_length = 128
    }
    command[3..3+chunk_length].copy_from_slice(&firmware[offset..offset+chunk_length]);
    checksum(&mut command);

    let mut port = SerialPort::builder()
        .baud_rate(BAUD_RATE)
        .read_timeout(Some(Duration::from_secs(1)))
        .open(port)
        .expect("Failed to open port");
    port.write_all(&command)?;

    let mut buffer = [0u8];
    port.read_exact(&mut buffer)?;
    match buffer {
        [0x06] => Ok(true),
        _ => Ok(false)
    }
}

pub fn command_readflash(port: &String, offset: u16) -> Result<Option<Vec<u8>>> {
    let mut command = [0u8; 4];
    command[0] = 0x52;
    command[1] = ((offset >> 8) & 0xFF) as u8;
    command[2] = ((offset) & 0xFF) as u8;
    checksum(&mut command);

    let mut port = SerialPort::builder()
        .baud_rate(BAUD_RATE)
        .read_timeout(Some(Duration::from_secs(1)))
        .open(port)
        .expect("Failed to open port");
    port.write_all(&command)?;

    let mut block = [0u8; 132];
    port.read_exact(&mut block)?;

    if block[1] == 0xFF {
        // No more data
        return Ok(None)
    }

    if verify(&block) {
        let data = block[3..131].to_vec();
        return Ok(Some(data))
    }
    Ok(None)
}

pub fn get_available_ports() -> Vec<SerialPortInfo> {
    serialport5::available_ports().expect("No ports found")
}
