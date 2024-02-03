use clone_box::clone_box;

#[clone_box]
trait Answer {}

#[derive(Clone)]
pub struct Yes;

#[clone_box]
impl Answer for Yes {}

fn main()
{
    let yes = Yes;

    let test = yes.clone_box();
}