use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!(
            "Wrong number of arguments: {}. Only a directory path containing one or more torrents should be passed",
            (args.len() - 1)
        );
    }

    //bt_client::init(args[1])
}
