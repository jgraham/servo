use dom::bindings::utils::{DOMString};

#[deriving(Eq, Clone)]
pub enum Namespace {
    Null,
    HTML,
    XML,
    XMLNS,
    XLink,
    SVG,
    MathML,
    Other(~str)
}

impl Namespace {
    pub fn from_str(url: &str) -> Namespace {
        match url {
            "http://www.w3.org/1999/xhtml" => HTML,
            "http://www.w3.org/XML/1998/namespace" => XML,
            "http://www.w3.org/2000/xmlns/" => XMLNS,
            "http://www.w3.org/1999/xlink" => XLink,
            "http://www.w3.org/2000/svg" => SVG,
            "http://www.w3.org/1998/Math/MathML" => MathML,
            _ => Other(url.to_owned())
        }
    }
    pub fn to_str(&self) -> DOMString {
        match *self {
            Null => None,
            HTML => Some(~"http://www.w3.org/1999/xhtml"),
            XML => Some(~"http://www.w3.org/XML/1998/namespace"),
            XMLNS => Some(~"http://www.w3.org/2000/xmlns/"),
            XLink => Some(~"http://www.w3.org/1999/xlink"),
            SVG => Some(~"http://www.w3.org/2000/svg"),
            MathML => Some(~"http://www.w3.org/1998/Math/MathML"),
            Other(ref x) => Some(x.to_owned())
        }
    }
}
