pub mod base;
pub mod index;
pub mod login;
pub mod misc;
pub mod new;
pub mod view;

/*
pub fn render<T: stpl::Render>(
    template: &T,
    data: &<T as Template>::Argument,
) -> Vec<u8>
where
    <T as Template>::Argument: serde::Serialize + 'static,
{

    let mut out = vec![];

    template.render(data, &mut out).unwrap();

    out
}
*/
