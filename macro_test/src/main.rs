#[macro_use]

use clone_box::clone_box;

fn main() {
    println!("Hello, world!");
}

#[clone_box]
trait Test
{
}

#[derive(Clone)]
pub struct Yes
{

}

#[clone_box]
impl Test for Yes
{

}