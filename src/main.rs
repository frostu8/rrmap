use std::fs::File;
use std::io::Write;

use rrmap::map::Map;
use rrmap::wad::Wad;

fn main() {
    let file = std::env::args()
        .nth(1)
        .expect("Pass wad file as first argument!");

    let lump = std::env::args().nth(2);

    let mut file = File::open(file).expect("Valid file");
    let wad = Wad::from_reader(&mut file).unwrap();

    if let Some(lump_name) = lump {
        let lump = wad.lump(lump_name).expect("No lump");

        // put to stdout
        let mut stdout = std::io::stdout();
        stdout.write_all(lump.data()).expect("write");
    } else {
        println!("{:?}", wad.header());
        print!("Lumps:");

        for lump in wad.lumps() {
            print!(" {}", lump.name());
        }

        println!();

        if let Some(text) = wad.lump("TEXTMAP") {
            // read as text
            let text = std::str::from_utf8(text.data()).unwrap();

            let map = Map::from_str(text);

            println!("Map: {:?}", map);
        }
    }
}
