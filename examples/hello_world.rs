use robius_location::{Error, Location, Manager};

struct Handler;

impl robius_location::Handler for Handler {
    fn handle(&self, location: Location<'_>) {
        println!("received location: {:?}", location.coordinates());
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
    manager.update_once();

    loop {}
}
