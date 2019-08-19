use std::io::prelude::*;
use std::net::TcpStream;
use std::str::from_utf8;

fn main() -> std::io::Result<()> {
	let addr = "irc.freenode.org:6667";
	let chan = "#testbot32";
	let nick = "osw-switch";

	let mut s = TcpStream::connect(addr)?;
	s.write(format!("NICK {}\n", nick).as_ref())?;
	s.write(format!("USER {} 0 * :test bot\n", nick).as_ref())?;
	s.write(format!("JOIN {}\n", chan).as_ref())?;
	let mut buff = [0; 128];
	loop{
		s.read(&mut buff)?;
		println!("{:#?}", from_utf8(&buff).unwrap());
	}
	Ok(())
}

