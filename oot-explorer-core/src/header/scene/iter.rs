use crate::header::scene::type_::SceneHeaderType;
use crate::header::scene::SceneHeader;

pub struct SceneHeaderIter<'a> {
    data: &'a [u8],
}

impl<'a> SceneHeaderIter<'a> {
    pub fn new(data: &'a [u8]) -> SceneHeaderIter<'a> {
        SceneHeaderIter { data }
    }
}

impl<'a> Iterator for SceneHeaderIter<'a> {
    type Item = SceneHeader<'a>;

    fn next(&mut self) -> Option<SceneHeader<'a>> {
        let header = SceneHeader::new(self.data);
        if header.type_() == SceneHeaderType::END {
            None
        } else {
            self.data = &self.data[SceneHeader::SIZE..];
            Some(header)
        }
    }
}
