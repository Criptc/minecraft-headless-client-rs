use {
    std::{
        process::exit,
        time::SystemTime,
    }
};

pub fn varint_write(number: i32) -> Vec<u8>{
    let mut data: Vec<u8> = Vec::new();
    let mut num = number;

    if num < 0{
        num = (1 << 31) + num
    }

    while num >= 0x80{
        let byte = num & 0x7F;
        num = num >> 7;

        let tmp = (0x80 | byte) as u8;
        data.extend_from_slice(&[tmp]);
    }
    let byte = (num & 0x7F) as u8;
    data.extend_from_slice(&[byte]);
    data
}

pub fn varint_read(data: &Vec<u8>) -> (i32, usize){
    if data.len() == 0{
        println!("error, data can't be empty [pub varint_read]");
        exit(1)
    }

    let mut out: i32 = 0;
    let mut shift = 0;
    let mut val: i64 = 0x80;
    let mut n = 0;

    while val & 0x80 != 0{
        val = data[n..n+1][0] as i64;
        out = out | ((val & 0x7F) << shift) as i32;
        shift += 7;
        n += 1
    }

    if out & (1 << 31) == 1{
        out = out - (1 << 31)
    }

    (out, n)
}

fn packer(packet: Vec<u8>) -> Vec<u8>{
    let mut buff = varint_write(packet.len() as i32);
    buff.extend_from_slice(packet.as_slice());
    buff
}

pub fn read_string(data: &Vec<u8>) -> (String, i32){
    if data.len() == 0{
        panic!("error, data can't be empty [pub read_string]");
    }

    let (mut size, byte_size) = varint_read(data);
    let string = String::from_utf8_lossy(&data[byte_size..(size as usize + byte_size)]).replace("\0", "");
    size = byte_size as i32 + size;
    (string, size)
}

fn write_string(data: String) -> Vec<u8>{
    let mut buff: Vec<u8> = Vec::new();
    let size = data.as_bytes().len() as i32;
    buff.extend_from_slice(varint_write(size).as_slice());
    buff.extend_from_slice(data.as_bytes());
    buff
}

pub fn handshake(protocol: i32, server_addr: String, server_port: u16, next_state: String) -> Vec<u8>{
    if next_state.to_lowercase() != "status" && next_state.to_lowercase() != "login"{
        panic!("error: next_state on handshake packet must be 'status' or 'login', not: {}", next_state.to_lowercase());
    }

    let protocol_ver: Vec<u8> = varint_write(protocol);
    let server = server_addr.as_bytes();
    let server_size: Vec<u8> = varint_write(server.len() as i32);
    let port = server_port.to_be_bytes();

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(varint_write(0x0).as_slice()); // packet ID
    buf.extend_from_slice(protocol_ver.as_slice()); // protocol version
    buf.extend_from_slice(server_size.as_slice()); // server ip length
    buf.extend_from_slice(server); // server ip used to connect
    buf.extend_from_slice(port.as_slice()); // server port used to connect

    if next_state.to_lowercase() == "status"{
        buf.extend_from_slice(&[1u8]);
    } else if next_state.to_lowercase() == "login"{
        buf.extend_from_slice(&[2u8]);
    }
    packer(buf)
}

pub fn status_request() -> Vec<u8>{
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(varint_write(0x0).as_slice());
    packer(buf)
}

pub fn ping_request() -> Vec<u8>{
    let mut buff: Vec<u8> = Vec::new();
    let epoch_new: u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    buff.extend_from_slice(varint_write(0x1).as_slice());
    buff.extend_from_slice(epoch_new.to_be_bytes().as_slice());
    packer(buff)
}

//Todo: add uuid support by asking for a bool and Optional uuid (?) and then encode them properly

pub fn login_start(username: String) -> Vec<u8>{
    if username.len() > 16{
        panic!("error: username must be less then 16 characters, size: {}", username.len());
    }

    let mut buff: Vec<u8> = Vec::new();
    let user = write_string(username);
    let has_uuid = [0u8];

    buff.extend_from_slice(varint_write(0x0).as_slice());
    buff.extend_from_slice(user.as_slice());
    buff.extend_from_slice(&has_uuid);
    packer(buff)
}

// stuff received from a server

// Todo: add uuid support
pub fn login_success(packet: &Vec<u8>) -> String{
    let mut data = packet[16..packet.len()].to_vec(); // skips uuid

    let (username, size) = read_string(&data);
    data = data[size as usize..data.len()].to_vec();

    let (props, size) = varint_read(&data);
    data = data[size..data.len() as usize].to_vec();

    for _ in 0..props{
        let (name, size) = read_string(&data);
        data = data[size as usize..data.len()].to_vec();

        let (value, size) = read_string(&data);
        data = data[size as usize..data.len()].to_vec();

        if data[0] == 0u8{
            data = data[1..data.len()].to_vec();
            println!("name: {}\nvalue: {}\n", name, value);
        } else {
            data = data[1..data.len()].to_vec();

            let (sig, size) = read_string(&data);
            data = data[size as usize..data.len()].to_vec();

            println!("name: {}\nvalue: {}\nsignature: {}\n", name, value, sig);
        }
    }
    username
}

pub fn compression_request(packet: &Vec<u8>) -> i32{
    let (i, _) = varint_read(packet);
    i
}

pub fn login_disconnect(packet: &Vec<u8>) -> String{
    if packet.len() == 0{
        panic!("error, data can't be empty [pub login_disconnect]");
    }

    let (json_data, _) = read_string(packet);
    json_data
}

pub fn status_response(packet: &Vec<u8>) -> String{
    if packet.len() == 0{
        panic!("error, data can't be empty [pub status_response]");
    }

    let (json_data, _) = read_string(packet);
    json_data
}

pub fn ping_response(packet: &Vec<u8>) -> String{
    if packet.len() != 8{
        panic!("error, invalid length packet\nexpected: 8\nreceived: {}", packet.len());
    }

    let now: u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let epoch = u64::from_be_bytes(packet.as_slice().try_into().unwrap());

    if now == epoch {
        format!("ping <1 second")
    } else {
        format!("ping: {}", now-epoch)
    }
}
