use pprint::core::basic::text;

fn main() {
    let test = text("hello")
        .cat_with_line(text("world"))
        .nest(2)
        .cat_with_line(text("!"))
        .group();
    let w = 20;
    println!("{}", test.pretty(w));
}
