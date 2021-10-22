use website_icon_extract::ImageLink;

pub fn get_best_icon(list: Vec<ImageLink>, minsize: usize, maxsize: usize) -> Option<ImageLink> {
    for image in list {
        if image.width >= minsize
            && image.width <= maxsize
            && image.height >= minsize
            && image.height <= maxsize
        {
            return Some(image);
        }
    }
    None
}
