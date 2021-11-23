use website_icon_extract::ImageLink;

fn proximity(optimal: i32, link: &ImageLink) -> i32
{
    let width: i32 = link.width as i32;
    let height: i32 = link.height as i32;
    (optimal - (width + height) / 2).abs()
}

pub fn get_best_icon(
    mut list: Vec<ImageLink>,
    optimal: usize,
    minsize: usize,
    maxsize: usize,
) -> Option<ImageLink> {
    if list.len() > 0 {
        let mut new_list: Vec<ImageLink> = list
            .drain(..)
            .filter(|image| {
                image.width >= minsize
                    && image.width <= maxsize
                    && image.height >= minsize
                    && image.height <= maxsize
            })
            .collect();
        new_list.sort_unstable_by(|a, b| {
            proximity(optimal as i32, b).cmp(&proximity(optimal as i32, a))
        });
        new_list.pop()
    } else {
        None
    }
}
