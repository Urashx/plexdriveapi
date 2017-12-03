extern crate getopts;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;
use getopts::Options;
use std::env;
fn main() {

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("p", "port", "set listening port", "32500");
    opts.optopt("l", "log", "set file path", "/var/log/plexdrive.log");
    opts.optopt("c", "command", "set restart command", "service plexdrive restart");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let port = matches.opt_str("p").expect("Specify listening port");
    let log = matches.opt_str("l").expect("Specify log file");
    let command = matches.opt_str("c").expect("Specify command");


    let bindadress  = format!("0.0.0.0:{}", port);
    println!("Start listening on {}", bindadress);
    let listener = TcpListener::bind(bindadress).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &log[..], &command[..]);
    }
}


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}



fn handle_connection(mut stream: TcpStream, logfile : &str, restartcommand : &str) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let getrestart = b"GET /restart HTTP/1.1\r\n";
    let getlog = b"GET /log HTTP/1.1\r\n";

    let (status_line, content) =  if buffer.starts_with(getrestart) {


        restart(restartcommand);
        let mut res = String::new();
        res.push_str("Reboot started !");
        ("HTTP/1.1 200 OK\r\n\r\n", res)


    }else if buffer.starts_with(getlog) {


        let contents = Command::new("sh").arg("-c").arg(format!("tail -n 300 {}", logfile)).output().expect("Tail log file fail");
        let contents = String::from_utf8_lossy(&contents.stdout).into_owned();

        ("HTTP/1.1 200 OK\r\n\r\n", contents)

    } else {

        let mut res = String::new();
        res.push_str("Commande inconnue");
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", res)

    };



    let response = format!("{}{}", status_line, content);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn restart(command : &str){
    Command::new("sh").arg("-c").arg(command).spawn().expect("Restart failed");
}
