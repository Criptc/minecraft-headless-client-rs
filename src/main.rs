mod packets;
use std::{
    net::{
        TcpStream
    },
    io::{
        Write,
        Read
    },
    process::exit
};
use flate2;

fn compress(data: Vec<u8>) -> Vec<u8>{
    let mut buf: Vec<u8> = Vec::new();
    let mut compresser = flate2::Compress::new(flate2::Compression::fast(), true);
    compresser.compress_vec(data.as_slice(), &mut buf, flate2::FlushCompress::Finish).unwrap();
    buf
}

fn decompress(data: Vec<u8>) -> Vec<u8>{
    let mut buf: Vec<u8> = Vec::new();
    let mut decompresser = flate2::Decompress::new(true);
    decompresser.decompress_vec(data.as_slice(), &mut buf, flate2::FlushDecompress::Finish).unwrap();
    buf
}

fn send(sock: &mut TcpStream, databuff: &[u8]){
    let ogsize: usize = databuff.len();
    match sock.write(databuff){
        Ok(size) => {
            if size != ogsize{
                println!("failed sending, og size: {}, actual sent size: {}", ogsize, size);
                exit(1);
            }
        }
        Err(err) => {
            println!("failed sending, err: {}", err);
            exit(1);
        }
    }
}

fn varint_from_stream(stream: &mut TcpStream) -> i32{
    let mut data: i32 = 0;
    for i in 0..5{
        let mut buff  = vec![0u8; 1];

        stream.read(buff.as_mut_slice()).unwrap();

        if buff.len() == 0{
            break
        }

        let byte = buff[0] as i64;
        data = data | ((byte & 0x7F) << (7 * i)) as i32;

        if buff[0] & 0x80 == 0{
            break
        }
    }

    return data
}

fn read_packet(sock: &mut TcpStream) -> (Vec<u8>, i32){
    let length: usize = varint_from_stream(sock) as usize;
    let mut buff: Vec<u8> = vec![0u8; length];
    match sock.read(buff.as_mut_slice()){
        Ok(_) => {}
        Err(err) => {
            println!("failed reading, err: {}", err);
            exit(1);
        }
    };
    let (id, size) = packets::varint_read(&buff);
    buff = buff[size..buff.len()].to_vec();
    (buff, id)
}

fn connect(ip: &String, port: &u32) -> TcpStream{
    let mut sock = match TcpStream::connect(format!("{}:{}", ip, port)){
        Ok(k) => {k}
        Err(err) => {
            println!("could not connect to server, error: {}", err);
            exit(1);
        }
    };
    return sock
}

// todo: make it so that the ping is added to the json
pub fn get_status(ip: String, port: u16){
    let mut sock = connect(&ip, &(port as u32));

    send(&mut sock, packets::handshake(763, ip, port, "status".to_string()).as_slice());
    send(&mut sock, packets::status_request().as_slice());

    let (packet, _) = read_packet(&mut sock);
    let json = packets::status_response(&packet);

    send(&mut sock, packets::ping_request().as_slice());

    let(packet, _) = read_packet(&mut sock);
    let ping = packets::ping_response(&packet);
    println!("{}\n{}\n", json, ping);
}

pub fn offline_login(ip: String, port: u16){
    let mut compression = false;
    let mut compression_size = 0;
    //let mut encryption = false; // todo: check if encryption is only active on online mode servers

    let mut sock = connect(&ip, &(port as u32));

    send(&mut sock, packets::handshake(763, ip, port, "login".to_string()).as_slice());
    send(&mut sock, packets::login_start("rust_bot".to_string()).as_slice());


    loop{
        let (mut packet, id) = read_packet(&mut sock);

        if compression{
            packet = decompress(packet);
        }
        if id == 0x0{
            println!("disconnected, reason: {}", packets::login_disconnect(&packet));
            exit(0);
        } else if id == 0x1{ // encryption request
            // todo: add encryption to login
            println!("Error, no support for encryption at this time");
            exit(1);
        } else if id == 0x3 { // set compression
            compression = true;
            compression_size = packets::compression_request(&packet);
        } else if id == 0x4 { // login success
            println!("username: {}", packets::login_success(&packet));
            break;
        }
    }
}

fn main() {
    get_status("127.0.0.1".to_string(), 25565);
    offline_login("127.0.0.1".to_string(), 25565);
}
