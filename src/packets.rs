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

// todo: find a way to automatically get the ideas for each version
// yes i did all this by hand, took an hour
fn entity_id_to_string(id: i32) -> String{
    match id {
        0 => "allay",
        1 => "Area effect cloud",
        2 => "Armor stand",
        3 => "Arrow",
        4 => "Axolotl",
        5 => "Bat",
        6 => "Bee",
        7 => "Blaze",
        8 => "Block display",
        9 => "Boat",
        10 => "Camel",
        11 => "Cat",
        12 => "Cave Spider",
        13 => "Chest Boat",
        14 => "Chest Minecart",
        15 => "Chicken",
        16 => "Cod",
        17 => "Command Block Minecart",
        18 => "Cow",
        19 => "Creeper",
        20 => "Dolphin",
        21 => "Donkey",
        22 => "Dragon Fireball",
        23 => "Drowned",
        24 => "Egg",
        25 => "Elder Guardian",
        26 => "End Crystal",
        27 => "Ender Dragon",
        28 => "Ender Pearl",
        29 => "Enderman",
        30 => "Endermite",
        31 => "Evoker",
        32 => "Evoker Fangs",
        33 => "Experience Bottle",
        34 => "Experience Orb",
        35 => "Eye of Ender",
        36 => "Falling block",
        37 => "Firework Rocket",
        38 => "Fox",
        39 => "Frog",
        40 => "Furnace Minecart",
        41 => "Ghast",
        42 => "Giant",
        43 => "Glowing Item Frame", // is actually "Glow item frame" but "Glowing item frame" sounds better
        44 => "Glow Squid",
        45 => "Goat",
        46 => "Guardian",
        47 => "Hoglin",
        48 => "Hopper Minecart",
        49 => "Horse",
        50 => "Husk",
        51 => "Illusioner",
        52 => "Interaction", // why is an interaction an entity?
        53 => "Iron Golem",
        54 => "Item",
        55 => "Item Display",
        56 => "Item Frame",
        57 => "Fireball",
        58 => "Leash Knot",
        59 => "Lightning Bolt",
        60 => "Llama",
        61 => "Llama Spit",
        62 => "Magma Cube",
        63 => "Marker",
        64 => "Minecraft",
        65 => "Mooshroom",
        66 => "Mule",
        67 => "Ocelot",
        68 => "Painting",
        69 => "Panda", // Nice
        70 => "Parrot",
        71 => "Phantom",
        72 => "Pig",
        73 => "Piglin",
        74 => "Piglin Brute",
        75 => "Pillager",
        76 => "Polar Bear",
        77 => "Potion", // thrown potion?
        78 => "Pufferfish",
        79 => "Rabbit",
        80 => "Ravager",
        81 => "Salmon",
        82 => "Sheep",
        83 => "Shulker",
        84 => "Shulker Bullet",
        85 => "Silverfish bullet",
        86 => "Skeleton",
        87 => "Skeleton Horse",
        88 => "Slime",
        89 => "Small Fireball",
        90 => "Sniffer",
        91 => "Snow Golem",
        92 => "Snowball",
        93 => "Spawner Minecart",
        94 => "Spectral Arrow",
        95 => "Spider",
        96 => "Squid",
        97 => "Stray",
        98 => "Strider",
        99 => "Tadpole",
        100 => "Text Display",
        101 => "Tnt",
        102 => "Tnt Minecart",
        103 => "Trader Llama",
        104 => "Trident",
        105 => "Tropical Fish",
        106 => "Turtle",
        107 => "Vex",
        108 => "Villager",
        109 => "Vindicator",
        110 => "Wandering Trader",
        111 => "Warden",
        112 => "Witch",
        113 => "Wither",
        114 => "Wither Skeleton",
        115 => "Wither Skull",
        116 => "Wolf",
        117 => "Zoglin",
        118 => "Zombie",
        119 => "Zombie Horse",
        120 => "Zombie Villager",
        121 => "Zombified Piglin",
        122 => "Player",
        123 => "Fishing Bobber",
        _ => "Unknown"
    }.to_string()
}

// packets we send

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

pub fn spawn_entity(packet: &Vec<u8>) -> (i32, String, (f64, f64, f64, i8, i8, i8), i32, (i16, i16, i16)){
    let mut data = packet;

    let (entity_id, size) = varint_read(data);
    data = &data[size..data.len()].to_vec();

    // todo: add uuid support
    data = &data[16..data.len()].to_vec();

    let (entity_type_raw, size) = varint_read(data);
    let entity_type = entity_id_to_string(entity_type_raw);
    data = &data[size..data.len()].to_vec();

    let x = f64::from_be_bytes(data[0..8].try_into().unwrap());
    let y = f64::from_be_bytes(data[8..16].try_into().unwrap());
    let z = f64::from_be_bytes(data[16..24].try_into().unwrap());
    data = &data[24..data.len()].to_vec();


    let pitch = data[0..1][0] as i8;
    data = &data[1..data.len()].to_vec();

    let yaw = data[0..1][0] as i8;
    data = &data[1..data.len()].to_vec();

    let head_yaw = data[0..1][0] as i8;
    data = &data[1..data.len()].to_vec();

    let (entity_data, size) = varint_read(&data);
    data = &data[size..data.len()].to_vec();

    let velocity_x = i16::from_be_bytes(data[0..2].try_into().unwrap());
    let velocity_y = i16::from_be_bytes(data[2..4].try_into().unwrap());
    let velocity_z = i16::from_be_bytes(data[4..6].try_into().unwrap());

    (entity_id, entity_type, (x, y, z, pitch, yaw, head_yaw), entity_data, (velocity_x, velocity_y, velocity_z))
}

// todo: add uuid support
pub fn spawn_player(packet: &Vec<u8>) -> (i32, (f64, f64, f64), (i8, i8)){
    let mut data = packet;

    let (entity_id, size) = varint_read(&data);
    data = &data[size..data.len()].to_vec();

    data = &data[16..data.len()].to_vec();

    let x = f64::from_be_bytes(data[0..8].try_into().unwrap());
    let y = f64::from_be_bytes(data[8..16].try_into().unwrap());
    let z = f64::from_be_bytes(data[16..24].try_into().unwrap());
    data = &data[24..data.len()].to_vec();

    let yaw = data[0..1][0] as i8;
    data = &data[1..data.len()].to_vec();

    let pitch = data[0..1][0] as i8;
    data = &data[1..data.len()].to_vec();

    (entity_id, (x, y, z), (yaw, pitch))
}

pub fn entity_animation(packet: &Vec<u8>) -> (i32, String){
    let (e_ie, size) = varint_read(packet);
    let animation_byte = packet[size..size+1][0] as i8;

    let animation = match animation_byte{
        0 => "Swing main arm",
        1 => "Take Damage",
        2 => "Leave Bed",
        3 => "Swing offhand",
        4 => "Critical effect",
        5 => "Magic Critical effect (?)",
        _ => "Unknown"
    }.to_string();

    (e_ie, animation)
}
