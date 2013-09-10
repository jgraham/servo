//use dom::bindings::attr;
use dom::bindings::utils::{null_string, str};
use dom::bindings::utils::{BindingObject, CacheableWrapper, DOMString, ErrorResult, WrapperCache};
use dom::namespace;
use dom::namespace::Namespace;

use std::str::eq_slice;

pub struct Attr {
    priv local_name: Option<~str>,
    value: DOMString,
    name: ~str,
    namespace: Namespace,
    prefix: DOMString
}

impl Attr {
    pub fn new(name: ~str, value: ~str) -> Attr {
        Attr {
            local_name: None, //Only store local_name if it is different from name
            value: str(value),
            name: name, //TODO: Atomise attribute names
            namespace: namespace::Null,
            prefix: null_string
        }
    }

    pub fn new_ns(local_name: ~str, value: ~str,  name: ~str, namespace: Namespace, prefix: Option<~str>) -> Attr {
        
        Attr {
            local_name: if eq_slice(local_name, name) {None} else {Some(local_name)},
            value: str(value),
            name: name,
            namespace: namespace,
            prefix: match prefix {Some(x) => str(x), None => null_string}
        }
    }

    pub fn local_name(&self) -> ~str {
        match self.local_name {
            Some(ref x) => x.to_owned(),
            None => self.name.clone()
        }
    }

    pub fn GetLocalName(&self) -> DOMString {
        str(self.local_name())
    }

    pub fn GetValue(&self) -> DOMString {
        self.value.clone()
    }

    pub fn SetValue(&mut self, value: &DOMString) {
        self.value = (*value).clone()
    }

    pub fn GetName(&self) -> DOMString {
        str(self.name.clone())
    }

    pub fn GetNamespace(&self) -> DOMString {
        self.namespace.to_str()
    }

    pub fn GetPrefix(&self) -> DOMString {
        self.prefix.clone()
    }
}
