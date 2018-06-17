// vim:set sw=4 sts=4:
use std::env;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
	println!("rust-fwd <listen> <dest>");
	return;
    }

    let listen = &args[1];
    let dest = &args[2];
    let listener = TcpListener::bind(listen).unwrap();

    for stream in listener.incoming() {
	match stream {
	    Err(e) => { println!("failed: {}", e) }
	    Ok(stream) => {
		let passdest = dest.clone();
		thread::spawn(move || {
		    fwd(stream, passdest);
		    println!("handle thread done");
		});
	    }
	}
    }
}

fn fwd(ls: TcpStream, dest: String) {
    println!("fwd to {}", dest);
    let rs = TcpStream::connect(dest).unwrap();

    // start copy local -> remote
    let (lerr, lntf) = iocopy(&ls, &rs);
    // start copy remote -> local
    let (rerr, rntf) = iocopy(&rs, &ls);

    loop {
	// check local error
	match lerr.try_recv() {
	    Err(_) => {},
	    Ok(_) => { break; }
	}
	match rerr.try_recv() {
	    Err(_) => {},
	    Ok(_) => { break; }
	}
	// sleep a sec
	thread::sleep(Duration::from_secs(1));
    }

    // send notification to iocopy threads
    lntf.send(()).unwrap();
    rntf.send(()).unwrap();
}

fn iocopy(src: &TcpStream, dst: &TcpStream) -> (Receiver<()>, Sender<()>) {
    let (esch, erch) = channel(); // error
    let (nsch, nrch) = channel(); // notify for terminate
    // launch thread
    let s = src.try_clone().unwrap();
    let mut d = dst.try_clone().unwrap();
    let e = esch.clone();
    thread::spawn(move || {
	let mut buf = [0u8; 1024];
	loop {
	    let n = read(&s, &mut buf, &e);
	    if n > 0 {
		d.write(&mut buf[..n]).unwrap();
	    }
	    match nrch.try_recv() {
		Err(_) => {},
		Ok(_) => { break; }
	    }
	}
	thread::sleep(Duration::from_secs(1));
    });
    return (erch, nsch);
}

fn read(mut s: &TcpStream, buf: &mut [u8], ech: &Sender<()>) -> usize {
    s.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

    match s.read(buf) {
	Ok(0) => { ech.send(()).unwrap(); 0 },
	Ok(n) => n,
	Err(e) => {
	    if e.kind() != ErrorKind::WouldBlock {
		ech.send(()).unwrap();
	    }
	    0
	}
    }
}
