#![feature(proc_macro)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{parse_expr, parse_item, Abi, BareFnArg, BareFnTy, Block, Constness, Expr, ExprKind,
          FnArg, FnDecl, Generics, Ident, Item, ItemKind, Pat, Path, PathSegment, UnOp, Unsafety,
          Visibility};
use quote::{Tokens, ToTokens};
use std::option::Option;

#[proc_macro_attribute]
pub fn runtime_target_feature(features: TokenStream, function: TokenStream) -> TokenStream {
    let features_string = features.to_string();

    let features_str = strip_quotes(strip_parens(features_string.as_str()))
        .split(";")
        .map(str::trim)
        .collect::<Vec<_>>();

    let features = features_str
        .iter()
        .map(|x| {
                 x.split(",")
                     .map(str::trim)
                     .map(Feature::new)
                     .collect::<Vec<_>>()
             })
        .collect::<Vec<_>>();

    let have_features = features
        .iter()
        .map(|x| x.iter().map(|ref x| x.checker_expr()).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let function_string = function.to_string();

    let function_item = syn::parse_item(&function_string).unwrap();
    let function_type = function_item_to_type(&function_item.node);

    let (function_decl, function_unsafety, function_constness, function_abi, function_generics) =
        match function_item.node {
            ItemKind::Fn(ref a, b, c, ref d, ref e, _) => (a.as_ref(), b, c, d, e),
            _ => panic!("item must be a function"),
        };

    let args = function_decl
        .inputs
        .iter()
        .map(fn_arg_to_expr)
        .collect::<Vec<_>>();

    let setup_ident = Ident::new("setup");
    let default_ident = Ident::new("default");
    let features_ident: Vec<Ident> = features
        .iter()
        .map(|x| {
                 x.iter()
                     .map(|x| x.to_ident_string())
                     .fold("with".to_string(), |acc, x| acc + "_" + &x)
                     .into()
             })
        .collect();

    let function_item_default = Item {
        ident: default_ident.clone(),
        vis: Visibility::Inherited,
        attrs: function_item.attrs.clone(),
        node: function_item.node.clone(),
    };

    let function_item_with_features = features_ident
        .iter()
        .map(|x| {
                 Item {
                     ident: x.clone(),
                     vis: Visibility::Inherited,
                     attrs: function_item.attrs.clone(),
                     node: function_item.node.clone(),
                 }
             })
        .collect::<Vec<_>>();

    let setup_args = args.clone();

    let setup_block_tokens = quote! {
        {
            let chosen_function = #(if #(#have_features)&&* {
                #features_ident
            } else )*{
                #default_ident
            };

            PTR.store(chosen_function, rt::atomic::Ordering::Relaxed);

            chosen_function(#(#setup_args),*)
        }
    };

    let setup_block = parse_block(setup_block_tokens.as_str());
    let setup_node = ItemKind::Fn(Box::new(function_decl.clone()),
                                  function_unsafety,
                                  function_constness,
                                  function_abi.clone(),
                                  function_generics.clone(),
                                  setup_block);

    let function_item_setup = Item {
        ident: setup_ident.clone(),
        vis: Visibility::Inherited,
        attrs: function_item.attrs.clone(),
        node: setup_node,
    };

    let dispatch_function_block_tokens = quote! {
        {
            pub extern crate runtime_target_feature_rt as rt;

            static PTR: rt::atomic::Atomic<#function_type> = rt::atomic::Atomic::new(#setup_ident);

            #function_item_setup

            #function_item_default

            #(#[target_feature = #features_str]
            #function_item_with_features)*

            PTR.load(rt::atomic::Ordering::Relaxed)(#(#args),*)
        }
    };

    let dispatch_function_block = parse_block(dispatch_function_block_tokens.as_str());

    let dispatch_function_node = ItemKind::Fn(Box::new(function_decl.clone()),
                                              function_unsafety,
                                              function_constness,
                                              function_abi.clone(),
                                              function_generics.clone(),
                                              dispatch_function_block);

    let dispatch_function = Item {
        ident: function_item.ident.clone(),
        vis: function_item.vis,
        attrs: function_item.attrs.clone(),
        node: dispatch_function_node,
    };


    let mut tokens = Tokens::new();
    dispatch_function.to_tokens(&mut tokens);
    println!("{}", tokens);
    tokens.parse().unwrap()
}

fn parse_block(input: &str) -> Box<Block> {
    Box::new(match parse_expr(input).unwrap().node {
                 ExprKind::Block(_, block) => block,
                 _ => unreachable!(),
             })
}

fn strip_parens(str_with_parens: &str) -> &str {
    if !(str_with_parens.starts_with("(") || str_with_parens.ends_with(")")) {
        panic!("attribute arguments must begin with '(' and end with ')'");
    }

    &str_with_parens[1..str_with_parens.len() - 1].trim()
}

fn strip_quotes(str_with_quotes: &str) -> &str {
    if !(str_with_quotes.starts_with("\"") || str_with_quotes.ends_with("\"")) {
        panic!("attribute arguments must be a string literal");
    }

    &str_with_quotes[1..str_with_quotes.len() - 1].trim()
}

fn function_item_to_type(item_kind: &ItemKind) -> BareFnTy {
    let (decl, unsafety, abi, generics): (&FnDecl, Unsafety, &Option<Abi>, &Generics) =
        match *item_kind {
            ItemKind::Fn(ref a, b, _, ref d, ref e, _) => (&*a, b, d, e),
            _ => panic!("item must be a function"),
        };

    let inputs = decl.inputs
        .iter()
        .map(|x| match *x {
                 FnArg::Captured(_, ref ty) => ty,
                 FnArg::Ignored(ref ty) => ty,
                 _ => panic!("self not supported"),
             })
        .map(|ty| {
                 BareFnArg {
                     name: Option::None,
                     ty: ty.clone(),
                 }
             })
        .collect::<Vec<_>>();

    let lifetimes = Vec::new(); // TODO


    BareFnTy {
        unsafety: unsafety,
        abi: abi.clone(),
        lifetimes: lifetimes,
        inputs: inputs,
        output: decl.output.clone(),
        variadic: decl.variadic,
    }
}

fn fn_arg_to_expr(arg: &FnArg) -> Expr {
    match *arg {
        FnArg::Captured(ref pat, _) => {
            match *pat {
                Pat::Ident(_, ref ident, _) => {
                    Expr {
                        node: ExprKind::Path(Option::None, Path::from(ident.clone())),
                        attrs: Vec::new(),
                    }
                }
                _ => panic!("pattern must be identifier"),
            }
        }
        _ => panic!("argument must be captured"),
    }
}

enum Feature<'a> {
    Enable(&'a str),
    Disable(&'a str),
}

impl<'a> Feature<'a> {
    fn new(feature_str: &'a str) -> Self {
        if feature_str.starts_with("+") {
            Feature::Enable(&feature_str[1..])
        } else if feature_str.starts_with("-") {
            Feature::Disable(&feature_str[1..])
        } else {
            panic!("feature must begin with '+' or '-'");
        }
    }

    fn name(&self) -> String {
        match *self {
                Feature::Enable(x) => x,
                Feature::Disable(x) => x,
            }
            .replace(|c| c == '.' || c == '-', "_")
    }

    fn to_ident_string(&self) -> String {
        match *self {
                Feature::Enable(_) => "enable_",
                Feature::Disable(_) => "disable_",
            }
            .to_string() + &self.name()
    }

    fn checker_expr(&self) -> Expr {
        let segments = ["rt", &("have_".to_string() + &self.name())]
            .iter()
            .map(|x| PathSegment::from(*x))
            .collect();
        let path = Path {
            global: false,
            segments: segments,
        };
        let function = Expr {
            node: ExprKind::Path(Option::None, path),
            attrs: Vec::new(),
        };
        let call = Expr {
            node: ExprKind::Call(Box::new(function), Vec::new()),
            attrs: Vec::new(),
        };
        match *self {
            Feature::Enable(_) => call,
            Feature::Disable(_) => {
                Expr {
                    node: ExprKind::Unary(UnOp::Not, Box::new(call)),
                    attrs: Vec::new(),
                }
            }
        }
    }
}
