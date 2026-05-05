use lib_questions::span;

fn main() {
    let mut sc = span::SpanCollection::new();
    sc.new_span(
        String::from("s1"),
        String::from("t1"),
        String::from("AuthService"),
        1000,
    );

    println!("{0}", sc.summarize_all());
}
