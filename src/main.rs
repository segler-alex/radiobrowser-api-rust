#[macro_use]
extern crate serde_derive;

use std::{env, thread, time};

mod db;
mod api;

fn main() {
    let listen_host : String = env::var("HOST").unwrap_or(String::from("127.0.0.1"));
    let listen_port : i32 = env::var("PORT").unwrap_or(String::from("8080")).parse().expect("listen port is not number");
    let dbhost = env::var("DB_HOST").unwrap_or(String::from("localhost"));
    let dbport : i32 = env::var("DB_PORT").unwrap_or(String::from("3306")).parse().expect("db port is not a number");
    let dbuser = env::var("DB_USER").expect("You have to set DB_USER env var");
    let dbpass = env::var("DB_PASS").expect("You have to set DB_PASS env var");
    let dbname = env::var("DB_NAME").expect("You have to set DB_NAME env var");
    let threads : usize = env::var("THREADS").unwrap_or(String::from("50")).parse().expect("threads is not number");
    
    let mut counter : i32 = 0;
    loop {
        let connection = db::new(&dbhost, dbport, &dbname, &dbuser, &dbpass);
        match connection {
            Ok(v) => {
                api::run(v, listen_host, listen_port, threads);
                break;
            },
            Err(e) => {
                println!("{}", e);
                counter = counter + 1;
                if counter < 10 {
                    thread::sleep(time::Duration::from_millis(1000));
                }else{
                    break;
                }
            }
        }
    }
}
