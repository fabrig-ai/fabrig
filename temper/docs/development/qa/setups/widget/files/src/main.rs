const TABLE: [(&str, u32); 3] = [("apples", 3), ("pears", 5), ("plums", 2)];

fn main() {
    for (name, count) in TABLE {
        println!("{name:<8}{count}");
    }
}
