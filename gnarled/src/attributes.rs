pub trait AttributeReverse {
    fn reverse(&self) -> Self;
}

pub trait AttributeRange {
    fn range(&self, t0: f32, t1: f32) -> Self;
}

impl AttributeReverse for () {
    fn reverse(&self) -> Self {
        ()
    }
}

pub trait AttributeSVGLine {
    fn line_attributes(&self) -> String;
}

impl AttributeRange for () {
    fn range(&self, _t0: f32, _t1: f32) -> Self {
        ()
    }
}

impl AttributeSVGLine for () {
    fn line_attributes(&self) -> String {
        return "stroke=\"black\"".to_string();
    }
}
