use dom::bindings::codegen::AttrBinding;
use dom::bindings::utils::{BindingObject, CacheableWrapper, DOMString, WrapperCache};
use dom::namespace;
use dom::namespace::Namespace;
use script_task::{page_from_context};

use js::jsapi::{JSObject, JSContext};

use std::cast;
use std::str::eq_slice;

pub struct Attr {
    wrapper: WrapperCache,
    priv local_name: Option<~str>,
    value: ~str,
    name: ~str,
    namespace: Namespace,
    prefix: Option<~str>
}

impl CacheableWrapper for Attr {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&mut self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        AttrBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Attr {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}

impl Attr {
    pub fn new(name: ~str, value: ~str) -> Attr {
        Attr {
            wrapper: WrapperCache::new(),
            local_name: None, //Only store local_name if it is different from name
            value: value,
            name: name, //TODO: Atomise attribute names
            namespace: namespace::Null,
            prefix: None
        }
    }

    pub fn new_ns(local_name: ~str, value: ~str,  name: ~str, namespace: Namespace, prefix: Option<~str>) -> Attr {
        Attr {
            wrapper: WrapperCache::new(),
            local_name: if eq_slice(local_name, name) {None} else {Some(local_name)},
            value: value,
            name: name,
            namespace: namespace,
            prefix: prefix
        }
    }

    pub fn local_name(&self) -> ~str {
        match self.local_name {
            Some(ref x) => x.to_owned(),
            None => self.name.clone()
        }
    }

    pub fn LocalName(&self) -> DOMString {
        Some(self.local_name())
    }

    pub fn Value(&self) -> DOMString {
        Some(self.value.clone())
    }

    pub fn SetValue(&mut self, value: &DOMString) {
        self.value = match *value {
            Some(ref x) => x.clone(),
            None => ~"null" //surely WebIDL should do this conversion?
        }
    }

    pub fn Name(&self) -> DOMString {
        Some(self.name.clone())
    }

    pub fn GetNamespaceURI(&self) -> DOMString {
        self.namespace.to_str()
    }

    pub fn GetPrefix(&self) -> DOMString {
        match self.prefix {
            Some(ref x) => Some(x.clone()),
            None => None
        }
    }
}
