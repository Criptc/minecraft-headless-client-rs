mod packets;

use {
    flate2,
    std::{
        net::{
            TcpStream
        },
        io::{
            Write,
            Read,
        },
        process::exit,
    },
    packets::{
        varint_read,
        varint_write
    }
};


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

    let mut received_size = match sock.read(buff.as_mut_slice()){
        Ok(t) => {t}
        Err(err) => {
            panic!("failed reading, err: {}", err);
        }
    };
    while received_size != length{
        received_size += match sock.read(buff.as_mut_slice()){
            Ok(t) => {t}
            Err(err) => {
                panic!("failed reading, err: {}", err);
            }
        };
    }

    let (id, size) = varint_read(&buff);
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
    let mut sock = connect(&ip, &(port as u32));

    send(&mut sock, packets::handshake(763, ip, port, "login".to_string()).as_slice());
    send(&mut sock, packets::login_start("rust_bot".to_string()).as_slice());

    loop{
        let (mut packet, id) = read_packet(&mut sock);
        println!("id: {}", id);

        if compression{
            packet = decompress(packet);
        }
        if id == 0x0{
            panic!("disconnected, reason: {}", packets::login_disconnect(&packet));
        } else if id == 0x1{ // encryption request
            panic!("server must be using online mode or has chat reporting enabled due to requesting for encryption");
        } else if id == 0x2 { // login success
            println!("username: {}", packets::login_success(&packet));
            break;
        } else if id == 0x3 { // set compression
            compression = true;
            compression_size = packets::compression_request(&packet);
        } else if id == 0x4 { // login plugin request (custom login flow)
            println!("Warning: server tried sending a custom login request");
            send(&mut sock, packets::status_request().as_slice()); // re using the empty packet from status_request, if you want you can add support for different custom login flows
        }
    }

    let play_send = |sock: &mut TcpStream, packet: Vec<u8>| { // a simple little function to reduce the amount of code needed to send a packet
        let mut pack: Vec<u8>;
        if packet.len() >= compression_size as usize && compression {
            let mut buf: Vec<u8> = Vec::new();

            let uncompressed_size = varint_write(packet.len() as i32);
            let compressed = compress(packet);
            let size = varint_write((uncompressed_size.len() + compressed.len()) as i32);

            buf.extend_from_slice(size.as_slice());
            buf.extend_from_slice(uncompressed_size.as_slice());
            buf.extend_from_slice(compressed.as_slice());

            pack = buf
        } else {
            let mut buf: Vec<u8> = Vec::new();
            let data_length = varint_write(0);
            let size = varint_write((packet.len() + data_length.len()) as i32);

            buf.extend_from_slice(size.as_slice());
            buf.extend_from_slice(data_length.as_slice());
            buf.extend_from_slice(packet.as_slice());

            pack = buf;
        }

        send(sock, pack.as_slice());
    };

    let play_receive = |sock: &mut TcpStream| -> (Vec<u8>, i32) { // same perpous as play_send, to reduce amount of code needed to be writen
        let (mut packet, id) = read_packet(sock);
        if compression {
            let (uncompressed_size, size) = varint_read(&packet);
            packet = packet[size..packet.len()].to_vec();
            packet = decompress(packet);

            if packet.len() != uncompressed_size as usize{
                panic!("uncompressed packet size ({}) is not what it should be ({}) [play_receive]", packet.len(), uncompressed_size)
            }
        }
        (packet, id)
    };

    // data we will want to track
    // todo:
    //  add uuid support and turn uuids into player names
    //  figure out how to handle blocks and block states
    let mut entities: Vec<(i32, String, (f64, f64, f64, i8, i8, i8), i32, (i16, i16, i16))> = Vec::new(); // entity ID, {uuid will be here}, Type, (position x/y/x, pitch, yaw, head yaw), object id, (velocity x/y/z)
    let mut players: Vec<(i32, (f64, f64, f64), (i8, i8))> = Vec::new();

    // todo: work on play mode packets
    loop{ // main play mode loop
        let (packet, id) = play_receive(&mut sock);

        // todo:
        //  add pack bundles 0x0
        //  add spawn exp orb 0x2
        //  Award Statistics 0x5
        //  Acknowledge block state 0x6
        //  Set Block destroy stage 0x7
        match id{
            0x1 => {
                // so messy but dammit it works
                let (e_id, e_name, (x, y, z, pitch, yaw, h_yaw), data, (vel_x, vel_y, vel_z)) = packets::spawn_entity(&packet);
                println!("new entity, name: {}\nid: {}\nposition: {}, {}, {}", e_name, e_id, x, y, z);
                entities.append(&mut vec!((e_id, e_name, (x, y, z, pitch, yaw, h_yaw), data, (vel_x, vel_y, vel_z))));
            },
            0x3 => {
                let (e_id, (x, y, z), (yaw, pitch)) = packets::spawn_player(&packet);
                println!("new player, ID: {}\nposition: {}, {}, {}", e_id, x, y, z);
                players.append(&mut vec!((e_id, (x, y, z), (yaw, pitch))));
            },
            0x4 => {
                let (e_id, animation) = packets::entity_animation(&packet);
                println!("play with id {} played the {} animation", e_id, animation);
            },
            _ => {
                println!("unknown packet, id: {}", id);
            }
        }
    }
}

fn main() {
    get_status("127.0.0.1".to_string(), 25565);
    offline_login("127.0.0.1".to_string(), 25565);
}
