use robius_location::{Error, Location, Manager};

struct Handler;

impl robius_location::Handler for Handler {
    fn handle(&self, _: Location<'_>) {
        println!("received location");
    }

    fn error(&self, _: Error) {
        println!("received error");
    }
}

fn main() {
    println!("a");
    let manager = Manager::new(Handler);
    println!("b");
    manager.request_authorization();
    println!("c");
    // manager.request_location();
}
