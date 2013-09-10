/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.
use dom::bindings::utils::{null_string, str};
use dom::bindings::utils::{BindingObject, CacheableWrapper, DOMString, ErrorResult, WrapperCache};
use dom::htmlcollection::HTMLCollection;
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::node::{ElementNodeTypeId, Node, ScriptView, AbstractNode};
use dom::attr:: Attr;
use dom::document;
use dom::namespace;
use dom::namespace::Namespace;
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse};
use newcss::stylesheet::Stylesheet;
use dom::util::{ascii_lowercase};

use js::jsapi::{JSContext, JSObject};

use std::cell::Cell;
use std::comm;
use std::str::{eq, eq_slice};
use std::ascii::StrAsciiExt;
use std::FromStr;

pub struct Element {
    parent: Node<ScriptView>,
    tag_name: ~str,     // TODO: This should be an atom, not a ~str.
    attrs: ~[Attr],
    style_attribute: Option<Stylesheet>,
}

impl CacheableWrapper for Element {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        self.parent.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!("no wrapping")
    }
}

impl BindingObject for Element {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.parent.GetParentObject(cx)
    }
}

#[deriving(Eq)]
pub enum ElementTypeId {
    HTMLElementTypeId,
    HTMLAnchorElementTypeId,
    HTMLAppletElementTypeId,
    HTMLAreaElementTypeId,
    HTMLAudioElementTypeId,
    HTMLBaseElementTypeId,
    HTMLBRElementTypeId,
    HTMLBodyElementTypeId,
    HTMLButtonElementTypeId,
    HTMLCanvasElementTypeId,
    HTMLDataElementTypeId,
    HTMLDataListElementTypeId,
    HTMLDirectoryElementTypeId,
    HTMLDListElementTypeId,
    HTMLDivElementTypeId,
    HTMLEmbedElementTypeId,
    HTMLFieldSetElementTypeId,
    HTMLFontElementTypeId,
    HTMLFormElementTypeId,
    HTMLFrameElementTypeId,
    HTMLFrameSetElementTypeId,
    HTMLHRElementTypeId,
    HTMLHeadElementTypeId,
    HTMLHeadingElementTypeId,
    HTMLHtmlElementTypeId,
    HTMLIframeElementTypeId,
    HTMLImageElementTypeId,
    HTMLInputElementTypeId,
    HTMLLabelElementTypeId,
    HTMLLegendElementTypeId,
    HTMLLinkElementTypeId,
    HTMLLIElementTypeId,
    HTMLMapElementTypeId,
    HTMLMediaElementTypeId,
    HTMLMetaElementTypeId,
    HTMLMeterElementTypeId,
    HTMLModElementTypeId,
    HTMLObjectElementTypeId,
    HTMLOListElementTypeId,
    HTMLOptGroupElementTypeId,
    HTMLOptionElementTypeId,
    HTMLOutputElementTypeId,
    HTMLParagraphElementTypeId,
    HTMLParamElementTypeId,
    HTMLPreElementTypeId,
    HTMLProgressElementTypeId,
    HTMLQuoteElementTypeId,
    HTMLScriptElementTypeId,
    HTMLSelectElementTypeId,
    HTMLSourceElementTypeId,
    HTMLSpanElementTypeId,
    HTMLStyleElementTypeId,
    HTMLTableElementTypeId,
    HTMLTableCaptionElementTypeId,
    HTMLTableCellElementTypeId,
    HTMLTableColElementTypeId,
    HTMLTableRowElementTypeId,
    HTMLTableSectionElementTypeId,
    HTMLTemplateElementTypeId,
    HTMLTextAreaElementTypeId,
    HTMLTimeElementTypeId,
    HTMLTitleElementTypeId,
    HTMLTrackElementTypeId,
    HTMLUListElementTypeId,
    HTMLVideoElementTypeId,
    HTMLUnknownElementTypeId,
}

//
// Element methods
//

impl<'self> Element {
    pub fn new(type_id: ElementTypeId, tag_name: ~str) -> Element {
        Element {
            parent: Node::new(ElementNodeTypeId(type_id)),
            tag_name: tag_name,
            attrs: ~[],
            style_attribute: None,
        }
    }

    pub fn normalise_attr_name(&self, &DOMString name) -> ~str {
        //FIXME: Throw for XML-invalid names
        let owner = self.parent.owner_doc;
        match owner {
            Some(document) => {
                if document.with_base(|doc| doc.doctype) == document::HTML { // && self.namespace == Namespace::HTML
                        ascii_lowercase(name.to_str())
                } else {
                    name.to_str()
                }
            },
            None => fail!("Elements should always have an owner")
        }
    }

    pub fn get_reflect_attr(&'self self, name: &str) -> Option<&'self str> {
        self.get_attribute(None, name)
    }

    pub fn get_attribute(&'self self, namespace_url: Option<&DOMString>, name: &str) -> Option<&'self str> {
        let namespace = match (namespace_url) {
            Some(x) => Namespace::from_str(x.get_ref()),
            None => namespace::Null
        };
        for attr in self.attrs.iter() {
            if (eq_slice(attr.local_name(), name) &&
                attr.namespace == namespace)
                {
                let val: &str = attr.value.get_ref();
                return Some(val);
            }
        }
        return None;    
    }

    pub fn set_attribute(&mut self, namespace: Namespace, name: ~str, value: &DOMString) {
        //FIXME: Throw for XML-invalid names
        //FIXME: Throw for XMLNS-invalid names

        let (prefix, local_name) = if name.contains(":")  {
            let parts: ~[&str] = name.splitn_iter(':', 1).collect();
            (Some(parts[0].to_owned()), parts[1].to_owned())
        } else {
            (None, name.clone())
        };
        match prefix {
            Some(ref prefix_str) => {
                if (namespace == namespace::Null ||
                    (eq(prefix_str, &~"xml") && namespace != namespace::XML) ||
                    (eq(prefix_str, &~"xmlns") && namespace != namespace::XMLNS)) {
                    fail!("NamespaceError");
                }
            },
            None => {}
        }
        let value_cell = Cell::new(value.to_str());
        let mut found = false;
        for attr in self.attrs.mut_iter() {
            if (eq_slice(attr.local_name().to_str(), name) &&
                attr.namespace == namespace) {
                attr.value = str(value_cell.take().clone());
                found = true;
                break;
            }
        }
        if !found {
            self.attrs.push(Attr::new_ns(local_name.clone(), value_cell.take().clone(),
                                         name.to_str(), namespace.clone(), prefix));
        }
        self.after_set_attr(&namespace, local_name, value)
    }

    fn after_set_attr(&mut self, namespace: &Namespace, local_name: ~str, value: &DOMString) {
        if "style" == local_name && *namespace == namespace::Null {
            self.style_attribute = Some(
                Stylesheet::from_attribute(
                    FromStr::from_str("http://www.example.com/").unwrap(),
                    value.get_ref()));
        }

        match self.parent.owner_doc {
            Some(owner) => do owner.with_base |owner| { owner.content_changed() },
            None => {}
        }
    }

    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let doc = self.parent.owner_doc.unwrap();
        let win = doc.with_base(|doc| doc.window.unwrap());
        let cx = unsafe {(*win.page).js_info.get_ref().js_compartment.cx.ptr};
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }
}

impl Element {
    pub fn TagName(&self) -> DOMString {
        str(self.tag_name.to_owned().to_ascii_upper())
    }

    pub fn Id(&self) -> DOMString {
        let id = self.get_reflect_attr(&"id");
        match (id) {
            Some(x) => str(x.to_owned()),
            None => str(~"")
        }
    }

    pub fn SetId(&mut self, id: &DOMString) {
        self.set_attribute(namespace::Null, ~"id", id)
    }

    pub fn GetAttribute(&self, name: &DOMString) -> DOMString {
        let new_name = self.normalise_attr_name(name);
        for attr in self.attrs.iter() {
            if (eq_slice(attr.name, name)) {
                return attr.val;
            }
        }
        null_string;
    }

    pub fn GetAttributeNS(&self, namespace: &DOMString, local_name: &DOMString) -> DOMString {
        match self.get_attribute(Some(namespace), local_name.to_str()) {
            Some(x) => str(x.to_owned()),
            None => null_string
        }
    }

    pub fn SetAttribute(&mut self, name: &DOMString, value: &DOMString, _rv: &mut ErrorResult) {
        let new_name = self.normalise_attr_name(name);
        let value_cell = Cell::new(value.to_str());

        let mut found = false;
        for attr in self.attrs.mut_iter() {
            if (eq_slice(attr.name, new_name)) {
                attr.value = str(value_cell.take().clone());
                found = true;
                break;
            }
        }
        if (!found) {
            self.attrs.push(Attr::new(new_name.clone(), value_cell.take().clone()));
        }

        self.after_set_attr(&namespace::Null, new_name.clone(), value)
    }

    pub fn SetAttributeNS(&mut self, namespace_url: &DOMString, name: &DOMString, value: &DOMString, _rv: &mut ErrorResult) {
        let namespace = match namespace_url.get_ref() {
            "" => namespace::Null,
            x => Namespace::from_str(x)
        };
        self.set_attribute(namespace, name.to_str(), value);
    }

    pub fn RemoveAttribute(&self, _name: &DOMString, _rv: &mut ErrorResult) -> bool {
        false
    }

    pub fn RemoveAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString, _rv: &mut ErrorResult) -> bool {
        false
    }

    pub fn HasAttribute(&self, name: &DOMString) -> bool {
        match self.GetAttribute(name) {
            null_string => false,
            _ => true
        }
    }

    pub fn HasAttributeNS(&self, namespace: &DOMString, localname: &DOMString) -> bool {
        false;
    }

    pub fn GetElementsByTagName(&self, _localname: &DOMString) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn GetElementsByTagNameNS(&self, _namespace: &DOMString, _localname: &DOMString, _rv: &mut ErrorResult) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn GetElementsByClassName(&self, _names: &DOMString) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn MozMatchesSelector(&self, _selector: &DOMString, _rv: &mut ErrorResult) -> bool {
        false
    }

    pub fn SetCapture(&self, _retargetToElement: bool) {
    }

    pub fn ReleaseCapture(&self) {
    }

    pub fn MozRequestFullScreen(&self) {
    }

    pub fn MozRequestPointerLock(&self) {
    }

    pub fn GetClientRects(&self, abstract_self: AbstractNode<ScriptView>) -> @mut ClientRectList {
        let (rects, cx, scope) = match self.parent.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let node = abstract_self;
                        assert!(node.is_element());
                        let page = win.page;
                        let (port, chan) = comm::stream();
                        match unsafe {(*page).query_layout(ContentBoxesQuery(node, chan), port)} {
                            Ok(ContentBoxesResponse(rects)) => {
                                let cx = unsafe {(*page).js_info.get_ref().js_compartment.cx.ptr};
                                let cache = win.get_wrappercache();
                                let scope = cache.get_wrapper();
                                let rects = do rects.map |r| {
                                    ClientRect::new(
                                         r.origin.y.to_f32(),
                                         (r.origin.y + r.size.height).to_f32(),
                                         r.origin.x.to_f32(),
                                         (r.origin.x + r.size.width).to_f32(),
                                         cx,
                                         scope)
                                };
                                Some((rects, cx, scope))
                            },
                            Err(()) => {
                                debug!("layout query error");
                                None
                            }
                        }
                    }
                    None => {
                        debug!("no window");
                        None
                    }
                }
            }
            None => {
                debug!("no document");
                None
            }
        }.unwrap();

        ClientRectList::new(rects, cx, scope)
    }

    pub fn GetBoundingClientRect(&self, abstract_self: AbstractNode<ScriptView>) -> @mut ClientRect {
        match self.parent.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let page = win.page;
                        let node = abstract_self;
                        assert!(node.is_element());
                        let (port, chan) = comm::stream();
                        match unsafe{(*page).query_layout(ContentBoxQuery(node, chan), port)} {
                            Ok(ContentBoxResponse(rect)) => {
                                let cx = unsafe {(*page).js_info.get_ref().js_compartment.cx.ptr};
                                let cache = win.get_wrappercache();
                                let scope = cache.get_wrapper();
                                ClientRect::new(
                                    rect.origin.y.to_f32(),
                                    (rect.origin.y + rect.size.height).to_f32(),
                                    rect.origin.x.to_f32(),
                                    (rect.origin.x + rect.size.width).to_f32(),
                                    cx,
                                    scope)
                            },
                            Err(()) => fail!("error querying layout")
                        }
                    }
                    None => fail!("no window")
                }
            }
            None => fail!("no document")
        }
    }

    pub fn ScrollIntoView(&self, _top: bool) {
    }

    pub fn ScrollTop(&self) -> i32 {
        0
    }

    pub fn SetScrollTop(&mut self, _scroll_top: i32) {
    }

    pub fn ScrollLeft(&self) -> i32 {
        0
    }

    pub fn SetScrollLeft(&mut self, _scroll_left: i32) {
    }

    pub fn ScrollWidth(&self) -> i32 {
        0
    }

    pub fn ScrollHeight(&self) -> i32 {
        0
    }

    pub fn ClientTop(&self) -> i32 {
        0
    }

    pub fn ClientLeft(&self) -> i32 {
        0
    }

    pub fn ClientWidth(&self) -> i32 {
        0
    }

    pub fn ClientHeight(&self) -> i32 {
        0
    }

    pub fn GetInnerHTML(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }

    pub fn SetInnerHTML(&mut self, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetOuterHTML(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }

    pub fn SetOuterHTML(&mut self, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn InsertAdjacentHTML(&mut self, _position: &DOMString, _text: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn QuerySelector(&self, _selectors: &DOMString, _rv: &mut ErrorResult) -> Option<AbstractNode<ScriptView>> {
        None
    }
}

