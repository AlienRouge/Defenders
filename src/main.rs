extern crate rand;
use rand::Rng;
use std::char;
use std::env;
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

fn run_client(ip: String) {
    match TcpStream::connect(ip) {
        Ok(mut stream) => {
            println!("Сonnection established.");

            let mut data = [0 as u8; 50];
            let mut rep = [0 as u8; 50];

            loop {
                let hash_str = get_hash_str();
                let session_key = get_session_key();

                let next_key = next_session_key(&hash_str, &session_key);

                println!("Message: ");
                let mut message = String::new();
                io::stdin().read_line(&mut message);

                stream.write(&hash_str.into_bytes()).unwrap();
                stream.write(&session_key.into_bytes()).unwrap();
                stream.write(&message.into_bytes()).unwrap();

                match stream.read(&mut data) {
                    Ok(size) => {
                        stream.read(&mut rep);
                        let received_key = from_utf8(&data[0..size]).unwrap();
                        let response = from_utf8(&rep).unwrap();

                        if received_key == next_key {
                            println!("Client key: {}, Server key: {}", next_key, received_key);
                        } else {
                            break;
                        }
                        println!("Response: {}", response);
                    }
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Сonnection closed");
}

fn run_server(port: i32, limit: i32) {
    let mut client_limit = limit;
    let address = format!("localhost:{}", port);
    let listener = TcpListener::bind(address.to_string()).unwrap();
    println!("Server started. Listenin on {}...", port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if client_limit > 0 {
                    client_limit -= 1;
                    println!("Connected new client: {}", stream.peer_addr().unwrap());
                    thread::spawn(move || handle_request(stream));
                } else {
                    println!("The server is stopped(overflow).");
                    break;
                }
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
    drop(listener);
}

fn handle_request(mut stream: TcpStream) {
    let mut hash = [0 as u8; 5];
    let mut key = [0 as u8; 10];
    let mut message = [0 as u8; 50];
    while match stream.read(&mut hash) {
        Ok(_) => {
            stream.read(&mut key);
            stream.read(&mut message);
            let received_hash = from_utf8(&hash).unwrap();
            let received_key = from_utf8(&key).unwrap();
            let new_key = next_session_key(&received_hash, &received_key);
            let result = new_key.clone().into_bytes();
            stream.write(&result).unwrap();
            stream.write(&message).unwrap();
            true
        }
        Err(_) => {
            println!("Connection error with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn get_session_key() -> String {
    let mut key = String::new();
    let mut rng = rand::thread_rng();

    for _i in 0..10 {
        let num = rng.gen_range(1, 10);
        let ch = char::from_digit(num, 10).unwrap();
        key.push(ch);
    }

    return key;
}

fn get_hash_str() -> String {
    let mut hash_str = String::new();
    let mut rng = rand::thread_rng();

    for _i in 0..5 {
        let num = rng.gen_range(1, 7);
        let ch = char::from_digit(num, 10).unwrap();
        hash_str.push(ch);
    }

    return hash_str;
}

fn next_session_key(hash_str: &str, session_key: &str) -> String {
    if hash_str.is_empty() {
        return "Empty HASH.".to_string();
    }

    for ch in hash_str.chars() {
        if !ch.is_ascii_digit() {
            return "Hash code contains non-digit letter".to_string();
        }
    }

    let mut result = 0;

    for ch in hash_str.chars() {
        let l = ch.to_string();
        result += calc_hash(session_key.to_string(), l.parse::<u64>().unwrap())
            .parse::<u64>()
            .unwrap();
    }

    return result.to_string();
}

fn calc_hash(key: String, value: u64) -> String {
    match value {
        1 => {
            let chp = "00".to_string() + &(key[0..5].parse::<u64>().unwrap() % 97).to_string(); // MATCH CONSTRUCTION
            return chp[chp.len() - 2..chp.len()].to_string();
        }
        2 => {
            let reverse_key = key.chars().rev().collect::<String>();
            return reverse_key + &key.chars().nth(0).unwrap().to_string();
        }
        3 => {
            return key[key.len() - 5..key.len()].to_string() + &key[0..5].to_string();
        }
        4 => {
            let mut num = 0;
            for _i in 1..9 {
                num += key.chars().nth(_i).unwrap().to_digit(10).unwrap() as u64 + 41;
            }
            return num.to_string();
        }
        5 => {
            let mut ch: char;
            let mut num = 0;
            for _i in 0..key.len() {
                ch = ((key.chars().nth(_i).unwrap() as u8) ^ 43) as char;
                if !ch.is_ascii_digit() {
                    ch = (ch as u8) as char;
                }
                num += ch as u64;
            }
            return num.to_string();
        }
        _ => return (key.parse::<u64>().unwrap() + value).to_string(),
    }
}

fn main() {
    // Запуск сервера: <порт> -n <ограничение_пользователей>
    // Запуск клиента: <ip:port> (localhost:port)
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        run_client(args[1].clone())
    } else if (args.len() >= 4) && (args[2] == "-n") {
        if (args[1].parse::<i32>().unwrap() >= 1) && (args[1].parse::<i32>().unwrap() <= 65535) {
            run_server(
                args[1].parse::<i32>().unwrap(),
                args[3].parse::<i32>().unwrap(),
            )
        } else {
            println!("Error. Wrong port value to start the server!")
        }
    } else {
        println!("Error. Wrong format of data.")
    }
}
