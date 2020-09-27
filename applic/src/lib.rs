use proc_macro::TokenStream as OldTokenStream;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized, parse_macro_input, punctuated::Punctuated, token, Expr, Ident, Pat, Path,
    ReturnType, Token,
};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Error,
};
use token::{Bang, Comma, FatArrow, Move, Or, Question, Underscore};

// pipe separated items, ApplicItem | ApplicItem
struct Applic {
    items: Punctuated<ApplicItem, FatArrow>,
}

impl Parse for Applic {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            items: input.parse_terminated(ApplicItem::parse)?,
        })
    }
}

struct ApplicItem {
    ident: Path,
    _brace_token: token::Paren,
    args: Punctuated<Underscore, Comma>,
    finish: ApplicFinish,
}

struct ClosureItem {
    capture: Option<Move>,
    or1_token: Or,
    inputs: Punctuated<Pat, Comma>,
    or2_token: Or,
    output: ReturnType,
    body: Box<Expr>,
}

impl ClosureItem {
    pub fn quote(&self) -> TokenStream {
        let Self {
            capture,
            or1_token,
            inputs,
            or2_token,
            output,
            body,
        } = self;

        quote! { #capture #or1_token #inputs #or2_token #output #body }
    }
}

impl Parse for ClosureItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            capture: input.parse()?,
            or1_token: input.parse()?,
            inputs: input.parse_terminated(Pat::parse)?,
            or2_token: input.parse()?,
            output: input.parse()?,
            body: input.parse()?,
        })
    }
}

enum ApplicFinish {
    None,
    Map(Question),
}

impl ApplicFinish {
    pub fn is_map(&self) -> bool {
        matches!(self, ApplicFinish::Map(_))
    }
}

impl Parse for ApplicFinish {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lk = input.lookahead1();

        if lk.peek(Question) {
            Ok(ApplicFinish::Map(input.parse()?))
        } else {
            Ok(ApplicFinish::None)
        }
    }
}

impl ApplicItem {
    fn args(&self, prefix: &str) -> TokenStream {
        let args = self
            .args
            .iter()
            .enumerate()
            .map(|(idx, _)| idx)
            .map(|idx| format_ident!("{}arg{}", prefix, idx));
        quote! { #(#args),* }
    }
}

impl Parse for ApplicItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            ident: input.parse()?,
            _brace_token: parenthesized!(content in input),
            args: content.parse_terminated(Underscore::parse)?,
            finish: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn just_testing(stream: OldTokenStream) -> OldTokenStream {
    let closure_item = parse_macro_input!(stream as ClosureItem);
    closure_item.quote().into()
}

#[proc_macro]
pub fn applic(item: OldTokenStream) -> OldTokenStream {
    let applic = parse_macro_input!(item as Applic);

    if applic.items.is_empty() {
        return quote! {{}}.into();
    }

    let mut items = applic.items.iter();
    let first_item = items.next().unwrap();
    let ident = &first_item.ident;
    let args = first_item.args("_");

    let mut call_stack = quote! {
        #ident(#args)
    };

    let items = items.zip(applic.items.iter());

    for (applic, prev) in items {
        let ident = &applic.ident;

        if applic.args.len() > 1 {
            if prev.finish.is_map() {
                let args = applic.args("_x");

                call_stack = quote! {
                    #call_stack.and_then(|(#args)| #ident(#args))
                };
            } else {
                let args = applic.args("_x");
                // let args = quote!()
                call_stack = quote! {
                    {
                        let (#args) = #call_stack;
                        #ident(#args)
                    }
                };
            }
        } else {
            if prev.finish.is_map() {
                let args = applic.args("_x");

                call_stack = quote! {
                    #call_stack.and_then(|#args| #ident(#args))
                };
            } else {
                call_stack = quote! {
                    #ident(#call_stack)
                };
            }
        }
    }

    let res = quote! {{
        |#args| #call_stack
    }};

    println!("res = {}", &res);

    res.into()
}
