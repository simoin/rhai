#![allow(unused)]

#[cfg(no_std)]
use core::mem;
#[cfg(not(no_std))]
use std::mem;

#[cfg(no_std)]
use alloc::format;
#[cfg(not(no_std))]
use std::format;

use std::borrow::Cow;

use quote::{quote, quote_spanned};
use syn::{parse::Parse, parse::ParseStream, parse::Parser, spanned::Spanned};

use crate::attrs::{ExportInfo, ExportScope, ExportedParams};
use crate::rhai_module::flatten_type_groups;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Index {
    Get,
    Set,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Property {
    Get(syn::Ident),
    Set(syn::Ident),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FnSpecialAccess {
    None,
    Index(Index),
    Property(Property),
}

impl Default for FnSpecialAccess {
    fn default() -> FnSpecialAccess {
        FnSpecialAccess::None
    }
}

#[derive(Debug, Default)]
pub(crate) struct ExportedFnParams {
    pub name: Option<Vec<String>>,
    pub return_raw: bool,
    pub skip: bool,
    pub span: Option<proc_macro2::Span>,
    pub special: FnSpecialAccess,
}

pub const FN_IDX_GET: &str = "index$get$";
pub const FN_IDX_SET: &str = "index$set$";

impl Parse for ExportedFnParams {
    fn parse(args: ParseStream) -> syn::Result<Self> {
        if args.is_empty() {
            return Ok(ExportedFnParams::default());
        }

        let info = crate::attrs::parse_attr_items(args)?;
        Self::from_info(info)
    }
}

impl ExportedParams for ExportedFnParams {
    fn parse_stream(args: ParseStream) -> syn::Result<Self> {
        Self::parse(args)
    }

    fn no_attrs() -> Self {
        Default::default()
    }

    fn from_info(info: crate::attrs::ExportInfo) -> syn::Result<Self> {
        let ExportInfo {
            item_span: span,
            items: attrs,
        } = info;
        let mut name = Vec::new();
        let mut return_raw = false;
        let mut skip = false;
        let mut special = FnSpecialAccess::None;
        for attr in attrs {
            let crate::attrs::AttrItem {
                key,
                value,
                span: item_span,
            } = attr;
            match (key.to_string().as_ref(), value) {
                ("get", None) | ("set", None) | ("name", None) => {
                    return Err(syn::Error::new(key.span(), "requires value"))
                }
                ("name", Some(s)) if &s.value() == FN_IDX_GET => {
                    return Err(syn::Error::new(
                        item_span,
                        "use attribute 'index_get' instead",
                    ))
                }
                ("name", Some(s)) if &s.value() == FN_IDX_SET => {
                    return Err(syn::Error::new(
                        item_span,
                        "use attribute 'index_set' instead",
                    ))
                }
                ("name", Some(s)) if s.value().starts_with("get$") => {
                    return Err(syn::Error::new(
                        item_span,
                        format!(
                            "use attribute 'getter = \"{}\"' instead",
                            &s.value()["get$".len()..]
                        ),
                    ))
                }
                ("name", Some(s)) if s.value().starts_with("set$") => {
                    return Err(syn::Error::new(
                        item_span,
                        format!(
                            "use attribute 'setter = \"{}\"' instead",
                            &s.value()["set$".len()..]
                        ),
                    ))
                }
                ("name", Some(s)) if s.value().contains('$') => {
                    return Err(syn::Error::new(
                        s.span(),
                        "Rhai function names may not contain dollar sign",
                    ))
                }
                ("name", Some(s)) if s.value().contains('.') => {
                    return Err(syn::Error::new(
                        s.span(),
                        "Rhai function names may not contain dot",
                    ))
                }
                ("name", Some(s)) => name.push(s.value()),
                ("set", Some(s)) => {
                    special = match special {
                        FnSpecialAccess::None => FnSpecialAccess::Property(Property::Set(
                            syn::Ident::new(&s.value(), s.span()),
                        )),
                        _ => return Err(syn::Error::new(item_span.span(), "conflicting setter")),
                    }
                }
                ("get", Some(s)) => {
                    special = match special {
                        FnSpecialAccess::None => FnSpecialAccess::Property(Property::Get(
                            syn::Ident::new(&s.value(), s.span()),
                        )),
                        _ => return Err(syn::Error::new(item_span.span(), "conflicting getter")),
                    }
                }
                ("index_get", None) => {
                    special = match special {
                        FnSpecialAccess::None => FnSpecialAccess::Index(Index::Get),
                        _ => {
                            return Err(syn::Error::new(item_span.span(), "conflicting index_get"))
                        }
                    }
                }

                ("index_set", None) => {
                    special = match special {
                        FnSpecialAccess::None => FnSpecialAccess::Index(Index::Set),
                        _ => {
                            return Err(syn::Error::new(item_span.span(), "conflicting index_set"))
                        }
                    }
                }
                ("return_raw", None) => return_raw = true,
                ("index_get", Some(s)) | ("index_set", Some(s)) | ("return_raw", Some(s)) => {
                    return Err(syn::Error::new(s.span(), "extraneous value"))
                }
                ("skip", None) => skip = true,
                ("skip", Some(s)) => return Err(syn::Error::new(s.span(), "extraneous value")),
                (attr, _) => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown attribute '{}'", attr),
                    ))
                }
            }
        }

        Ok(ExportedFnParams {
            name: if name.is_empty() { None } else { Some(name) },
            return_raw,
            skip,
            special,
            span: Some(span),
            ..Default::default()
        })
    }
}

#[derive(Debug)]
pub(crate) struct ExportedFn {
    entire_span: proc_macro2::Span,
    signature: syn::Signature,
    is_public: bool,
    mut_receiver: bool,
    params: ExportedFnParams,
}

impl Parse for ExportedFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fn_all: syn::ItemFn = input.parse()?;
        let entire_span = fn_all.span();
        let str_type_path = syn::parse2::<syn::Path>(quote! { str }).unwrap();

        // #[cfg] attributes are not allowed on functions due to what is generated for them
        crate::attrs::deny_cfg_attr(&fn_all.attrs)?;

        // Determine if the function is public.
        let is_public = matches!(fn_all.vis, syn::Visibility::Public(_));
        // Determine whether function generates a special calling convention for a mutable
        // reciever.
        let mut_receiver = {
            if let Some(first_arg) = fn_all.sig.inputs.first() {
                match first_arg {
                    syn::FnArg::Receiver(syn::Receiver {
                        reference: Some(_), ..
                    }) => true,
                    syn::FnArg::Typed(syn::PatType { ref ty, .. }) => {
                        match flatten_type_groups(ty.as_ref()) {
                            &syn::Type::Reference(syn::TypeReference {
                                mutability: Some(_),
                                ..
                            }) => true,
                            &syn::Type::Reference(syn::TypeReference {
                                mutability: None,
                                ref elem,
                                ..
                            }) => match flatten_type_groups(elem.as_ref()) {
                                &syn::Type::Path(ref p) if p.path == str_type_path => false,
                                _ => {
                                    return Err(syn::Error::new(
                                        ty.span(),
                                        "references from Rhai in this position \
                                            must be mutable",
                                    ))
                                }
                            },
                            _ => false,
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        };

        // All arguments after the first must be moved except for &str.
        for arg in fn_all.sig.inputs.iter().skip(1) {
            let ty = match arg {
                syn::FnArg::Typed(syn::PatType { ref ty, .. }) => ty,
                _ => panic!("internal error: receiver argument outside of first position!?"),
            };
            let is_ok = match flatten_type_groups(ty.as_ref()) {
                &syn::Type::Reference(syn::TypeReference {
                    mutability: Some(_),
                    ..
                }) => false,
                &syn::Type::Reference(syn::TypeReference {
                    mutability: None,
                    ref elem,
                    ..
                }) => {
                    matches!(flatten_type_groups(elem.as_ref()), &syn::Type::Path(ref p) if p.path == str_type_path)
                }
                &syn::Type::Verbatim(_) => false,
                _ => true,
            };
            if !is_ok {
                return Err(syn::Error::new(
                    ty.span(),
                    "this type in this position passes from \
                                                        Rhai by value",
                ));
            }
        }

        // No returning references or pointers.
        if let syn::ReturnType::Type(_, ref rtype) = fn_all.sig.output {
            match rtype.as_ref() {
                &syn::Type::Ptr(_) => {
                    return Err(syn::Error::new(
                        fn_all.sig.output.span(),
                        "cannot return a pointer to Rhai",
                    ))
                }
                &syn::Type::Reference(_) => {
                    return Err(syn::Error::new(
                        fn_all.sig.output.span(),
                        "cannot return a reference to Rhai",
                    ))
                }
                _ => {}
            }
        }
        Ok(ExportedFn {
            entire_span,
            signature: fn_all.sig,
            is_public,
            mut_receiver,
            params: ExportedFnParams::default(),
        })
    }
}

impl ExportedFn {
    pub(crate) fn params(&self) -> &ExportedFnParams {
        &self.params
    }

    pub(crate) fn update_scope(&mut self, parent_scope: &ExportScope) {
        let keep = match (self.params.skip, parent_scope) {
            (true, _) => false,
            (_, ExportScope::PubOnly) => self.is_public,
            (_, ExportScope::Prefix(s)) => self.name().to_string().starts_with(s),
            (_, ExportScope::All) => true,
        };
        self.params.skip = !keep;
    }

    pub(crate) fn skipped(&self) -> bool {
        self.params.skip
    }

    pub(crate) fn signature(&self) -> &syn::Signature {
        &self.signature
    }

    pub(crate) fn mutable_receiver(&self) -> bool {
        self.mut_receiver
    }

    pub(crate) fn is_public(&self) -> bool {
        self.is_public
    }

    pub(crate) fn span(&self) -> &proc_macro2::Span {
        &self.entire_span
    }

    pub(crate) fn name(&self) -> &syn::Ident {
        &self.signature.ident
    }

    pub(crate) fn exported_names(&self) -> Vec<syn::LitStr> {
        let mut literals = self
            .params
            .name
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|s| syn::LitStr::new(s, proc_macro2::Span::call_site()))
                    .collect()
            })
            .unwrap_or_else(|| Vec::new());

        match self.params.special {
            FnSpecialAccess::None => {}
            FnSpecialAccess::Property(Property::Get(ref g)) => literals.push(syn::LitStr::new(
                &format!("get${}", g.to_string()),
                g.span(),
            )),
            FnSpecialAccess::Property(Property::Set(ref s)) => literals.push(syn::LitStr::new(
                &format!("set${}", s.to_string()),
                s.span(),
            )),
            FnSpecialAccess::Index(Index::Get) => {
                literals.push(syn::LitStr::new(FN_IDX_GET, proc_macro2::Span::call_site()))
            }
            FnSpecialAccess::Index(Index::Set) => {
                literals.push(syn::LitStr::new(FN_IDX_SET, proc_macro2::Span::call_site()))
            }
        }

        if literals.is_empty() {
            literals.push(syn::LitStr::new(
                &self.signature.ident.to_string(),
                self.signature.ident.span(),
            ));
        }

        literals
    }

    pub(crate) fn exported_name<'n>(&'n self) -> Cow<'n, str> {
        if let Some(ref name) = self.params.name {
            Cow::Borrowed(name.last().unwrap().as_str())
        } else {
            Cow::Owned(self.signature.ident.to_string())
        }
    }

    pub(crate) fn arg_list(&self) -> impl Iterator<Item = &syn::FnArg> {
        self.signature.inputs.iter()
    }

    pub(crate) fn arg_count(&self) -> usize {
        self.signature.inputs.len()
    }

    pub(crate) fn return_type(&self) -> Option<&syn::Type> {
        if let syn::ReturnType::Type(_, ref rtype) = self.signature.output {
            Some(rtype)
        } else {
            None
        }
    }

    pub fn set_params(&mut self, mut params: ExportedFnParams) -> syn::Result<()> {
        // Several issues are checked here to avoid issues with diagnostics caused by raising them
        // later.
        //
        // 1. Do not allow non-returning raw functions.
        //
        if params.return_raw
            && mem::discriminant(&self.signature.output)
                == mem::discriminant(&syn::ReturnType::Default)
        {
            return Err(syn::Error::new(
                self.signature.span(),
                "return_raw functions must return Result<T>",
            ));
        }

        match params.special {
            // 2a. Property getters must take only the subject as an argument.
            FnSpecialAccess::Property(Property::Get(_)) if self.arg_count() != 1 => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "property getter requires exactly 1 argument",
                ))
            }
            // 2b. Property getters must return a value.
            FnSpecialAccess::Property(Property::Get(_)) if self.return_type().is_none() => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "property getter must return a value",
                ))
            }
            // 3a. Property setters must take the subject and a new value as arguments.
            FnSpecialAccess::Property(Property::Set(_)) if self.arg_count() != 2 => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "property setter requires exactly 2 arguments",
                ))
            }
            // 3b. Property setters must return nothing.
            FnSpecialAccess::Property(Property::Set(_)) if self.return_type().is_some() => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "property setter must return no value",
                ))
            }
            // 4a. Index getters must take the subject and the accessed "index" as arguments.
            FnSpecialAccess::Index(Index::Get) if self.arg_count() != 2 => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "index getter requires exactly 2 arguments",
                ))
            }
            // 4b. Index getters must return a value.
            FnSpecialAccess::Index(Index::Get) if self.return_type().is_none() => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "index getter must return a value",
                ))
            }
            // 5a. Index setters must take the subject, "index", and new value as arguments.
            FnSpecialAccess::Index(Index::Set) if self.arg_count() != 3 => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "index setter requires exactly 3 arguments",
                ))
            }
            // 5b. Index setters must return nothing.
            FnSpecialAccess::Index(Index::Set) if self.return_type().is_some() => {
                return Err(syn::Error::new(
                    self.signature.span(),
                    "index setter must return no value",
                ))
            }
            _ => {}
        }

        self.params = params;
        Ok(())
    }

    pub fn generate(self) -> proc_macro2::TokenStream {
        let name: syn::Ident =
            syn::Ident::new(&format!("rhai_fn_{}", self.name()), self.name().span());
        let impl_block = self.generate_impl("Token");
        let callable_block = self.generate_callable("Token");
        let input_types_block = self.generate_input_types("Token");
        let dyn_result_fn_block = self.generate_dynamic_fn();
        quote! {
            #[allow(unused)]
            pub mod #name {
                use super::*;
                struct Token();
                #impl_block
                #callable_block
                #input_types_block
                #dyn_result_fn_block
            }
        }
    }

    pub fn generate_dynamic_fn(&self) -> proc_macro2::TokenStream {
        let name = self.name().clone();

        let mut dynamic_signature = self.signature.clone();
        dynamic_signature.ident =
            syn::Ident::new("dynamic_result_fn", proc_macro2::Span::call_site());
        dynamic_signature.output = syn::parse2::<syn::ReturnType>(quote! {
            -> Result<Dynamic, EvalBox>
        })
        .unwrap();
        let arguments: Vec<syn::Ident> = dynamic_signature
            .inputs
            .iter()
            .filter_map(|fnarg| {
                if let syn::FnArg::Typed(syn::PatType { ref pat, .. }) = fnarg {
                    if let syn::Pat::Ident(ref ident) = pat.as_ref() {
                        Some(ident.ident.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let return_span = self
            .return_type()
            .map(|r| r.span())
            .unwrap_or_else(|| proc_macro2::Span::call_site());
        if !self.params.return_raw {
            quote_spanned! { return_span=>
                type EvalBox = Box<EvalAltResult>;
                pub #dynamic_signature {
                    Ok(Dynamic::from(super::#name(#(#arguments),*)))
                }
            }
        } else {
            quote_spanned! { return_span=>
                type EvalBox = Box<EvalAltResult>;
                pub #dynamic_signature {
                    super::#name(#(#arguments),*)
                }
            }
        }
    }

    pub fn generate_callable(&self, on_type_name: &str) -> proc_macro2::TokenStream {
        let token_name: syn::Ident = syn::Ident::new(on_type_name, self.name().span());
        let callable_fn_name: syn::Ident = syn::Ident::new(
            format!("{}_callable", on_type_name.to_lowercase()).as_str(),
            self.name().span(),
        );
        quote! {
            pub fn #callable_fn_name() -> CallableFunction {
                CallableFunction::from_plugin(#token_name())
            }
        }
    }

    pub fn generate_input_types(&self, on_type_name: &str) -> proc_macro2::TokenStream {
        let token_name: syn::Ident = syn::Ident::new(on_type_name, self.name().span());
        let input_types_fn_name: syn::Ident = syn::Ident::new(
            format!("{}_input_types", on_type_name.to_lowercase()).as_str(),
            self.name().span(),
        );
        quote! {
            pub fn #input_types_fn_name() -> Box<[TypeId]> {
                #token_name().input_types()
            }
        }
    }

    pub fn generate_impl(&self, on_type_name: &str) -> proc_macro2::TokenStream {
        let sig_name = self.name().clone();
        let name = self.params.name.as_ref().map_or_else(
            || self.name().to_string(),
            |names| names.last().unwrap().clone(),
        );

        let arg_count = self.arg_count();
        let is_method_call = self.mutable_receiver();

        let mut unpack_stmts: Vec<syn::Stmt> = Vec::new();
        let mut unpack_exprs: Vec<syn::Expr> = Vec::new();
        let mut input_type_exprs: Vec<syn::Expr> = Vec::new();
        let skip_first_arg;

        // Handle the first argument separately if the function has a "method like" receiver
        if is_method_call {
            skip_first_arg = true;
            let first_arg = self.arg_list().next().unwrap();
            let var = syn::Ident::new("arg0", proc_macro2::Span::call_site());
            match first_arg {
                syn::FnArg::Typed(pattern) => {
                    let arg_type: &syn::Type = match flatten_type_groups(pattern.ty.as_ref()) {
                        &syn::Type::Reference(syn::TypeReference { ref elem, .. }) => elem.as_ref(),
                        p => p,
                    };
                    let downcast_span = quote_spanned!(
                        arg_type.span()=> &mut args[0usize].write_lock::<#arg_type>().unwrap());
                    unpack_stmts.push(
                        syn::parse2::<syn::Stmt>(quote! {
                            let #var: &mut _ = #downcast_span;
                        })
                        .unwrap(),
                    );
                    input_type_exprs.push(
                        syn::parse2::<syn::Expr>(quote_spanned!(
                            arg_type.span()=> TypeId::of::<#arg_type>()
                        ))
                        .unwrap(),
                    );
                }
                syn::FnArg::Receiver(_) => todo!("true self parameters not implemented yet"),
            }
            unpack_exprs.push(syn::parse2::<syn::Expr>(quote! { #var }).unwrap());
        } else {
            skip_first_arg = false;
        }

        // Handle the rest of the arguments, which all are passed by value.
        //
        // The only exception is strings, which need to be downcast to ImmutableString to enable a
        // zero-copy conversion to &str by reference, or a cloned String.
        let str_type_path = syn::parse2::<syn::Path>(quote! { str }).unwrap();
        let string_type_path = syn::parse2::<syn::Path>(quote! { String }).unwrap();
        for (i, arg) in self.arg_list().enumerate().skip(skip_first_arg as usize) {
            let var = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
            let is_string;
            let is_ref;
            match arg {
                syn::FnArg::Typed(pattern) => {
                    let arg_type: &syn::Type = pattern.ty.as_ref();
                    let downcast_span = match flatten_type_groups(pattern.ty.as_ref()) {
                        &syn::Type::Reference(syn::TypeReference {
                            mutability: None,
                            ref elem,
                            ..
                        }) => match flatten_type_groups(elem.as_ref()) {
                            &syn::Type::Path(ref p) if p.path == str_type_path => {
                                is_string = true;
                                is_ref = true;
                                quote_spanned!(arg_type.span()=>
                                               mem::take(args[#i]).take_immutable_string().unwrap())
                            }
                            _ => panic!("internal error: why wasn't this found earlier!?"),
                        },
                        &syn::Type::Path(ref p) if p.path == string_type_path => {
                            is_string = true;
                            is_ref = false;
                            quote_spanned!(arg_type.span()=>
                                           mem::take(args[#i]).take_string().unwrap())
                        }
                        _ => {
                            is_string = false;
                            is_ref = false;
                            quote_spanned!(arg_type.span()=>
                                           mem::take(args[#i]).cast::<#arg_type>())
                        }
                    };

                    unpack_stmts.push(
                        syn::parse2::<syn::Stmt>(quote! {
                            let #var = #downcast_span;
                        })
                        .unwrap(),
                    );
                    if !is_string {
                        input_type_exprs.push(
                            syn::parse2::<syn::Expr>(quote_spanned!(
                                arg_type.span()=> TypeId::of::<#arg_type>()
                            ))
                            .unwrap(),
                        );
                    } else {
                        input_type_exprs.push(
                            syn::parse2::<syn::Expr>(quote_spanned!(
                                arg_type.span()=> TypeId::of::<ImmutableString>()
                            ))
                            .unwrap(),
                        );
                    }
                }
                syn::FnArg::Receiver(_) => panic!("internal error: how did this happen!?"),
            }
            if !is_ref {
                unpack_exprs.push(syn::parse2::<syn::Expr>(quote! { #var }).unwrap());
            } else {
                unpack_exprs.push(syn::parse2::<syn::Expr>(quote! { &#var }).unwrap());
            }
        }

        // In method calls, the first argument will need to be mutably borrowed. Because Rust marks
        // that as needing to borrow the entire array, all of the previous argument unpacking via
        // clone needs to happen first.
        if is_method_call {
            let arg0 = unpack_stmts.remove(0);
            unpack_stmts.push(arg0);
        }

        // Handle "raw returns", aka cases where the result is a dynamic or an error.
        //
        // This allows skipping the Dynamic::from wrap.
        let return_span = self
            .return_type()
            .map(|r| r.span())
            .unwrap_or_else(|| proc_macro2::Span::call_site());
        let return_expr = if !self.params.return_raw {
            quote_spanned! { return_span=>
                Ok(Dynamic::from(#sig_name(#(#unpack_exprs),*)))
            }
        } else {
            quote_spanned! { return_span=>
                #sig_name(#(#unpack_exprs),*)
            }
        };

        let type_name = syn::Ident::new(on_type_name, proc_macro2::Span::call_site());
        quote! {
            impl PluginFunction for #type_name {
                fn call(&self,
                        args: &mut [&mut Dynamic]
                ) -> Result<Dynamic, Box<EvalAltResult>> {
                    debug_assert_eq!(args.len(), #arg_count,
                                     "wrong arg count: {} != {}",
                                     args.len(), #arg_count);
                    #(#unpack_stmts)*
                    #return_expr
                }

                fn is_method_call(&self) -> bool { #is_method_call }
                fn is_varadic(&self) -> bool { false }
                fn clone_boxed(&self) -> Box<dyn PluginFunction> { Box::new(#type_name()) }
                fn input_types(&self) -> Box<[TypeId]> {
                    new_vec![#(#input_type_exprs),*].into_boxed_slice()
                }
            }
        }
    }
}
