fn main() {
  futures::executor::block_on(app::run()).unwrap();
}
