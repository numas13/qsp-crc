use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Generics, Meta, Path, PredicateType, Token,
    TraitBound, TraitBoundModifier, Type, TypeParamBound, WherePredicate, parse_macro_input,
    parse_quote, punctuated::Punctuated,
};

type Error = String;

fn make_type_param_trait_bound(path: Path) -> TypeParamBound {
    TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path,
    })
}

fn make_where_predicate_type(bounded_ty: Type, bound: TypeParamBound) -> WherePredicate {
    WherePredicate::Type(PredicateType {
        lifetimes: None,
        bounded_ty,
        colon_token: Token![:](Span::call_site()),
        bounds: Punctuated::from_iter([bound.clone()]),
    })
}

fn make_generic_bounds(generics: &Generics, path: Path) -> Generics {
    let bound = make_type_param_trait_bound(path);
    let mut ret = generics.clone();
    ret.make_where_clause().predicates.extend({
        generics.type_params().map(|i| {
            let ty = Type::Verbatim(i.ident.to_token_stream());
            make_where_predicate_type(ty, bound.clone())
        })
    });
    ret
}

fn make_crc_fn_body_for_struct(data: &DataStruct) -> TokenStream {
    let mut tokens = TokenStream::new();
    for i in &data.fields {
        let name = &i.ident;
        quote! { self.#name.crc(state); }.to_tokens(&mut tokens);
    }
    tokens
}

fn make_impl_crc_body(input: &DeriveInput) -> Result<TokenStream, Error> {
    match &input.data {
        Data::Struct(data) => {
            let body = make_crc_fn_body_for_struct(data);
            Ok(quote! {
                fn crc<S: qsp::CrcState>(&self, state: &mut S) {
                    #body
                }
            })
        }
        _ => Err("derive Crc for enum or union is not supported".into()),
    }
}

fn make_impl_crc_trait(input: &DeriveInput) -> Result<TokenStream, Error> {
    let body = make_impl_crc_body(input)?;
    let name = &input.ident;
    let generics = make_generic_bounds(&input.generics, parse_quote!(qsp::Crc));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let impl_tokens = quote! {
        impl #impl_generics qsp::Crc for #name #ty_generics #where_clause {
            #body
        }
    };
    Ok(impl_tokens)
}

#[proc_macro_derive(Crc)]
pub fn derive_crc(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    match make_impl_crc_trait(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => quote! { compile_error!(#err); }.into(),
    }
}

fn has_repr_packed(attrs: &[Attribute]) -> Result<bool, Error> {
    for attr in attrs.iter() {
        if let Meta::List(list) = &attr.meta {
            if list.path.is_ident("repr") {
                for i in list.tokens.to_string().split(",").map(|i| i.trim()) {
                    if i == "packed" || i == "packed(1)" {
                        return Ok(true);
                    } else if i.starts_with("packed(") {
                        return Err(
                            "repr(packed(N > 1)) is not supported because of trailing padding"
                                .into(),
                        );
                    }
                }
            }
        }
    }
    Ok(false)
}

fn make_where_clause_for_fields(
    data: &Data,
    generics: &Generics,
    bound: TypeParamBound,
) -> Result<Generics, Error> {
    match &data {
        Data::Struct(data) => {
            let mut ret = generics.clone();
            let predicates = &mut ret.make_where_clause().predicates;
            predicates.extend(
                data.fields
                    .iter()
                    .map(|i| make_where_predicate_type(i.ty.clone(), bound.clone())),
            );
            Ok(ret)
        }
        _ => Err("derive Crc for enum or union is not supported".into()),
    }
}

fn make_impl_packed(input: &DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;
    if !has_repr_packed(&input.attrs)? {
        return Err(format!("missing repr(packed) for struct {name}"));
    }
    let generics =
        make_where_clause_for_fields(&input.data, &input.generics, parse_quote!(qsp::Packed))?;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let impl_tokens = quote! {
        unsafe impl #impl_generics qsp::Packed for #name #ty_generics #where_clause {}
    };
    Ok(impl_tokens)
}

#[proc_macro_derive(Packed)]
pub fn derive_packed(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    match make_impl_packed(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => quote! { compile_error!(#err); }.into(),
    }
}
