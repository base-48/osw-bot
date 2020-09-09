use std::{thread, time};
use std::net::TcpStream;
use std::io::{BufReader, BufRead, Write, Result};
use std::sync::mpsc::{Sender, Receiver, channel};
use gpio::{GpioIn, GpioOut};

static ADDR: &str = "irc.freenode.org:6667";
static CHAN: &str = "#base48";
static NICK: &str = "osw-bot";

fn main() {
	loop{
		if let Err(err) = start() {
			println!("{:#?}", err);
			thread::sleep(time::Duration::new(180, 0));
		}
	}
}

fn start() -> Result<()> {
	let mut s = TcpStream::connect(ADDR)?;
	s.set_read_timeout(Some(time::Duration::new(360, 0)))?;
	let mut r = BufReader::new(s.try_clone()?);
	let (send, rec): (Sender<String>, Receiver<String>) = channel();
	
	// gpio inint
	let mut l_b     = gpio::sysfs::SysFsGpioOutput::open(27).unwrap();
	let mut sw_c    = gpio::sysfs::SysFsGpioInput::open(4).unwrap();
	let mut sw_o    = gpio::sysfs::SysFsGpioInput::open(17).unwrap();
	let mut l_o     = gpio::sysfs::SysFsGpioOutput::open(22).unwrap();
	let mut l_c     = gpio::sysfs::SysFsGpioOutput::open(23).unwrap();

	let sc = s.try_clone()?;        // start check switch thread
	thread::spawn(move || checksw(rec, sc, &mut sw_c, &mut sw_o, &mut l_o, &mut l_c),);

	s.write(format!("NICK {}\n", NICK).as_ref())?; // irc join 
	s.write(format!("USER {} 0 * :open switch bot\n", NICK).as_ref())?;
	s.write(format!("JOIN {}\n", CHAN).as_ref())?;

	loop{
		let mut data = String::new();
		r.read_line(&mut data)?;
		eval(data.trim_end().to_string(), &send, s.try_clone()?, &mut l_b)?;
	}
}

fn eval(mut data: String, send: &Sender<String>, mut s: TcpStream, 
		l_b: &mut gpio::sysfs::SysFsGpioOutput) -> Result<()> {
	
	println!("{:#?}", data);    // print data

	if data.starts_with("PING :") {
		s.write(format!("PONG :{}\n", data.trim_start_matches("PING :")).as_ref())?;
	}

	else{
		data.remove(0);
		let (_,l1) = data.split_at(data.find(' ').unwrap() + 1);
		if l1.starts_with("332 ") || l1.starts_with("TOPIC ") {
			let (_,l2) = l1.split_at(l1.find(':').unwrap() + 1);
			send.send(l2.to_string()).unwrap();
		}
		if l1.starts_with("PRIVMSG ") {
			let (_,l2) = l1.split_at(l1.find(':').unwrap() + 1);
			if l2 == ".beacon on" { 
				l_b.set_value(true).expect("Led beacon error");
            }
			if l2 == ".beacon off" {
				l_b.set_value(false).expect("Led beacon error");
			}
		}
	}
	Ok(())
}

fn checksw(rec: Receiver<String>, mut s: TcpStream, 
		sw_c: &mut gpio::sysfs::SysFsGpioInput,
		sw_o: &mut gpio::sysfs::SysFsGpioInput,
		l_o: &mut gpio::sysfs::SysFsGpioOutput,
		l_c: &mut gpio::sysfs::SysFsGpioOutput){

	let mut topic = String::new();
	let mut to: u32 = 0;

	loop{
		thread::sleep(time::Duration::new(1, 0));

		let os = sw_o.read_value().unwrap();
		let cs = sw_c.read_value().unwrap();

		if let Ok(data) = rec.try_recv() { topic = data; }

		if os == gpio::GpioValue::High && cs == gpio::GpioValue::Low  && to == 0
			&& ! topic.starts_with("base open") && ! topic.is_empty(){
			let (_,last) = topic.split_at(topic.find('|').unwrap_or(0));
			s.write(format!("TOPIC {} :base open \\o/ {}\n", CHAN, last).as_ref());
			to = 5;
		}
		if os == gpio::GpioValue::Low && cs == gpio::GpioValue::High && to == 0
			&& ! topic.starts_with("base closed") && ! topic.is_empty(){
			let (_,last) = topic.split_at(topic.find('|').unwrap_or(0));
			s.write(format!("TOPIC {} :base closed :( {}\n", CHAN, last).as_ref());
			to = 5;
		}
		if to != 0 { to=to-1; }

		if topic.starts_with("base open") {
			l_o.set_value(os).expect("Led open error");
		}

		if topic.starts_with("base closed") {
			l_c.set_value(cs).expect("Led close error");
		}
	}
}
