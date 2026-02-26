use ferric::ferric_main;

fn main() {
    assert_eq!(1, 1);
    let source = include_str!("src.txt");
    ferric_main(source);
}
