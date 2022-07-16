use blush::run;

fn main() {
	let program =
r#"
let a = 123;
"#;
	run(program);
}