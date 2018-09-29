use failure::Error;
use std::net::TcpStream;

struct EpochClient {
};

impl EpochClient {
    pub fn run(address: &str) -> Result<(), Error> {
    let mut stream = TcpStream::connect(address)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let handle = thread::spawn(move || {
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok() {
            print!("{}", line.replace(";", "\n"));
            line.clear()
        }
    });

    let mut reader = BufReader::new(stdin());
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok() {
        match Command::from_line(&line) {
            Ok(cmd) => {
                let s = serde_json::to_string(&cmd)?;
                println!("{}", s);
                writeln!(&mut stream, "{}", s)?;
            }
            Err(err) => {
                for e in err.iter_chain() {
                    eprintln!("{}", e);
                }
            }
        }
        line.clear();
    }

    handle
        .join()
        .map_err(|_| format_err!("Error while joining thread."))?;
    Ok(())
    }
}
