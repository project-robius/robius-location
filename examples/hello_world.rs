use robius_location::{Error, Location, Manager};

struct Handler;

impl robius_location::Handler for Handler {
    fn handle(&self, location: Location<'_>) {
        println!("received location: {:?}", location.coordinates());
    }

    fn error(&self, e: Error) {
        println!("received error: {e:?}");
    }
}

fn main() {
    let manager = Manager::new(Handler).unwrap();
    manager.request_authorization().unwrap();
    manager.update_once().unwrap();

    loop {}
}
