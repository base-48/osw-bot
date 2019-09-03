use std::io::prelude::Write;
use std::{thread, time};
use std::net::TcpStream;
use std::io::{BufReader, BufRead, Read};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::fs::File;

static ADDR: &str = "irc.freenode.org:6667";
static CHAN: &str = "#testbot32";
static NICK: &str = "osw-bot";

static OFILE: 	&str = "/sys/class/gpio/gpio2_pd2/value";
static CFILE:		&str = "/sys/class/gpio/gpio1_pd0/value";
static OLFILE:	&str = "/sys/class/gpio/gpio4_pd5/value";
static CLFILE:	&str = "/sys/class/gpio/gpio5_pd6/value";
static SFILE:		&str = "/sys/class/gpio/gpio3_pd1/value";

// debug
//static OFILE: &str = "ofile.tmp";
//static CFILE: &str = "cfile.tmp";
//static SFILE: &str = "beacon.tmp";
//static OLFILE:	&str = "clfile.tmp";
//static CLFILE:	&str = "olfile.tmp";

fn main() -> std::io::Result<()> {
	loop{
		let mut s = TcpStream::connect(ADDR)?;
		s.set_read_timeout(Some(time::Duration::new(360, 0)))?;
		let mut r = BufReader::new(s.try_clone()?);
		let (send, rec): (Sender<String>, Receiver<String>) = channel();

		s.write(format!("NICK {}\n", NICK).as_ref())?;
		s.write(format!("USER {} 0 * :test bot\n", NICK).as_ref())?;
		s.write(format!("JOIN {}\n", CHAN).as_ref())?;
		
		let sc = s.try_clone()?;
		thread::spawn(move || {
			checksw(rec, sc);
		});

		loop{
			let mut data = String::new();
			match r.read_line(&mut data) {
				Err(_) | Ok(0) => { thread::sleep(time::Duration::new(180, 0)); break; }
				Ok(_) => { eval(data.trim_end().to_string(), &send, s.try_clone()?)?; }
			}
		}
	}
}

fn checksw(rec: Receiver<String>, mut s: TcpStream){
	let mut topic = String::new();
	loop{
		thread::sleep(time::Duration::new(1, 0));
		let mut os = String::new();
		let mut cs = String::new();

		match rec.try_recv() {
			Ok(data)=> { topic = data; }
			Err(_) 	=> {}
		}
	    match File::open(OFILE) {
			Ok(mut file) => { file.read_to_string(&mut os).unwrap(); }
			Err(_) => {}
		}
	    match File::open(CFILE) {
			Ok(mut file) => { file.read_to_string(&mut cs).unwrap(); }
			Err(_) => {}
		}
		if os.trim() == "1" && cs.trim() == "0"
		&& ! topic.starts_with("base open") && ! topic.is_empty(){
			let (_,last) = topic.split_at(topic.find('|').unwrap_or(0));
			let mut top = last.to_string();
			top.remove(0);
			s.write(format!("TOPIC {} :base open \\o/ |{}\n", CHAN, top).as_ref());
		}
		if os.trim() == "0" && cs.trim() == "1"
			&& ! topic.starts_with("base close") && ! topic.is_empty(){
			let (_,last) = topic.split_at(topic.find('|').unwrap_or(0));
			let mut top = last.to_string();
			top.remove(0);
			s.write(format!("TOPIC {} :base close :( |{}\n", CHAN, top).as_ref());
		}
		match File::create(OLFILE) {
			Ok(mut file) => { file.write_all(os.as_bytes()).unwrap(); }
			Err(_) => {}
		}
		match File::create(CLFILE) {
			Ok(mut file) => { file.write_all(cs.as_bytes()).unwrap(); }
			Err(_) => {}
		}
	}
}

fn eval(mut data: String, send: &Sender<String>, mut s: TcpStream)
-> std::io::Result<()> {
	println!("{:#?}", data);

	if data.starts_with("PING :") {
		s.write(format!("PONG :{}\n", data.trim_start_matches("PING :")).as_ref())?;
	}
	else{
		data.remove(0);
		let (_,last) = data.split_at(data.find(' ').unwrap() + 1);
		if last.starts_with("332 ") || last.starts_with("TOPIC ") {
			let (_,last) = data.split_at(data.find(':').unwrap() + 1);
			send.send(last.to_string()).unwrap();
		}
		if last.starts_with("PRIVMSG ") {
			let (_,last) = data.split_at(data.find(':').unwrap() + 1);
			if last == ".beacon on" {
			    match File::create(SFILE) {
					Ok(mut file) => { file.write_all("1".as_bytes()).unwrap(); }
					Err(_) => {}
				}
			}
			if last == ".beacon off" {
				match File::create(SFILE) {
					Ok(mut file) => { file.write_all("0".as_bytes()).unwrap(); }
					Err(_) => {}
				}
			}
		}
	}
	Ok(())
}
