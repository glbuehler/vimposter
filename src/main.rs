use editor::*;

fn main() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        exit();
        println!("");
        hook(info);
    }));

    enter();
    run();
    exit();
}
