pub fn strtol(s: &str) -> (i32, &str) {
  let (num, count) = lexical::parse_partial(s).unwrap();

  (num, &s[count..])
}

fn main() {
  let args: Vec<_> = std::env::args().collect();
  if args.len() != 2 {
    eprintln!("{}: invalid number of arguments", args[0]);
    return;
  }

  println!("  .globl main");
  println!("main:");

  let (mut num, mut p) = strtol(args[1].as_str());
  println!("  mov ${}, %%rax", num);

  while !p.is_empty() {
    match p.chars().nth(0) {
      Some('+') => {
        (num, p) = strtol(&p[1..]);
        println!("  add ${}, %%rax", num);
      }
      Some('-') => {
        (num, p) = strtol(&p[1..]);
        println!("  sub ${}, %%rax", num);
      }

      ch => {
        eprintln!("unexpected character: '{}'", ch.unwrap());
        return;
      }
    }
  }

  println!("  ret");
}
