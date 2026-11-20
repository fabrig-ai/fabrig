const TABLE: [(&str, u32); 3] = [("apples", 3), ("pears", 5), ("plums", 2)];

fn main() {
    let csv = std::env::args().any(|arg| arg == "--csv");
    if csv {
        println!("name,count");
    }
    for (name, count) in TABLE {
        if csv {
            println!("{name},{count}");
        } else {
            println!("{name:<8}{count}");
        }
    }
}
