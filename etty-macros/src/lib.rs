use quote::quote;
use syn::parse_macro_input;
use syn::Token;

#[proc_macro]
#[doc(hidden)]
pub fn gen_csi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as GenCsi).tts.into()
}

struct GenCsi {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for GenCsi {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let csis = syn::punctuated::Punctuated::<CsiParse, Token![;]>::parse_terminated(input)?;
        let tts = csis
            .iter()
            .map(|csi| {
                let tts = &csi.tts;
                quote!(#tts)
            })
            .collect::<proc_macro2::TokenStream>();
        Ok(GenCsi { tts })
    }
}

struct CsiParse {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for CsiParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let visi = input.parse::<syn::Visibility>()?;
        let nm = input.parse::<syn::Ident>()?;
        let _fat_arrow = input.parse::<Token![=>]>()?;
        let CsiFmtParse { doc, fmt, nms_ord } = input.parse::<CsiFmtParse>()?;

        let args = {
            #[derive(Clone)]
            struct Arg {
                nm: proc_macro2::Ident,
                ty: proc_macro2::Ident,
            }
            let mut args = Vec::<Arg>::with_capacity(nms_ord.len());
            for _ in 0..nms_ord.len() {
                let _comma = input.parse::<Token![,]>()?;
                let CsiArgParse { nm, ty } = input.parse::<CsiArgParse>()?;
                args.push(Arg { nm, ty });
            }
            if nms_ord.len() != args.len() {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "unmatch args' count",
                ));
            }
            args
        };

        let arg_nms = {
            let mut nms = Vec::<proc_macro2::Ident>::with_capacity(args.len());
            let mut args = args.clone(); // clone `args` and reorder it according to `nms_ord`
            for ref nm_ord in nms_ord {
                let idx = args
                    .iter()
                    .enumerate()
                    .find_map(|(i, arg)| (nm_ord == &arg.nm.to_string()).then_some(i));
                let Some(idx) = idx else {
                    return Err(syn::Error::new(proc_macro2::Span::call_site(), "unmatch args' name"));
                };
                let arg = args.remove(idx);
                nms.push(arg.nm);
            }
            nms
        };

        let tts = {
            let doc = format!("`\\x1b[{}`", doc);
            let arg_exprs = args.iter().map(|arg| {
                let nm = &arg.nm;
                let ty = &arg.ty;
                quote! { #nm: #ty }
            });
            let ret = {
                if args.is_empty() {
                    quote! { Csi(std::borrow::Cow::from(#fmt)) }
                } else {
                    quote! { Csi(std::borrow::Cow::from(std::format!(#fmt, #(#arg_nms,)*))) }
                }
            };
            quote! {
                #[doc = #doc]
                #visi fn #nm (#(#arg_exprs,)*) ->  Csi<'static> {
                    #ret
                }
            }
        };
        Ok(CsiParse { tts })
    }
}

struct CsiArgParse {
    nm: proc_macro2::Ident,
    ty: proc_macro2::Ident,
}

impl syn::parse::Parse for CsiArgParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        static INT_TYS: [&str; 6] = ["u8", "u16", "u32", "u64", "u128", "usize"];
        let nm = input.parse::<proc_macro2::Ident>()?;
        let Ok(_colon) = input.parse::<Token![:]>() else {
            let ty = proc_macro2::Ident::new("u16", proc_macro2::Span::call_site());
            return Ok(CsiArgParse { nm, ty })
        };
        let ty = input.parse::<proc_macro2::Ident>()?;
        if !INT_TYS.contains(&ty.to_string().as_str()) {
            let msg = format!("expect {:?}", INT_TYS);
            return Err(syn::Error::new_spanned(ty, msg));
        }
        Ok(CsiArgParse { nm, ty })
    }
}

struct CsiFmtParse {
    doc: String,
    fmt: proc_macro2::Literal,
    nms_ord: Vec<String>,
}

impl syn::parse::Parse for CsiFmtParse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let litstr = input.parse::<syn::LitStr>()?;
        let s = litstr.value();
        let mut bytes = s.bytes().peekable();
        let mut nms_ord = Vec::<String>::new();
        let mut docbuf = Vec::<u8>::with_capacity(s.len() * 2);
        let mut fmtbuf = {
            let mut v = Vec::<u8>::with_capacity(s.len() + 2);
            v.push(b'\x1b');
            v.push(b'[');
            v
        };
        let mut nmbuf = Vec::<u8>::new();

        while let Some(byte) = bytes.next() {
            match byte {
                b @ b'{' => {
                    docbuf.push(b);
                    fmtbuf.push(b);
                    'cb: loop {
                        match bytes.next() {
                            None => {
                                return Err(syn::Error::new_spanned(
                                    litstr,
                                    "expect a closing brace `}`",
                                ));
                            }
                            Some(b @ b'{') => {
                                fmtbuf.push(b);
                                break 'cb;
                            }
                            Some(b @ b'}') => {
                                docbuf.push(b);
                                fmtbuf.push(b);
                                let nm = if nmbuf.is_empty() {
                                    return Err(syn::Error::new_spanned(
                                        litstr,
                                        "expect arg name inside the `{}`",
                                    ));
                                } else {
                                    let bytes = nmbuf.drain(..).collect::<Vec<u8>>();
                                    String::from_utf8(bytes).unwrap()
                                };
                                nms_ord.push(nm);
                                break 'cb;
                            }
                            Some(b) => {
                                docbuf.push(b);
                                nmbuf.push(b);
                                continue;
                            }
                        }
                    }
                }
                b @ b'}' => {
                    let Some(b'}') = bytes.next() else{
                        return Err(syn::Error::new_spanned(
                            litstr,
                            "expect a `{` before a `}`",
                        ));
                    };
                    docbuf.push(b);
                    fmtbuf.push(b);
                    fmtbuf.push(b);
                }
                b => {
                    docbuf.push(b);
                    fmtbuf.push(b);
                    continue;
                }
            };
        }
        let doc = String::from_utf8(docbuf).unwrap();
        let fmt = {
            let s = String::from_utf8(fmtbuf).unwrap();
            proc_macro2::Literal::string(&s)
        };
        Ok(CsiFmtParse { doc, fmt, nms_ord })
    }
}

// fn snake_to_pascal(s: &str) -> Option<String> {
//     let mut buf = Vec::<u8>::with_capacity(s.len());
//     let mut bytes = s.bytes();
//     let b = bytes
//         .by_ref()
//         .skip_while(|b| *b == b'_')
//         .find(|b| b.is_ascii_alphabetic())?;
//     buf.push(b.to_ascii_uppercase());
//     let mut flag = false;
//     for b in bytes {
//         match b {
//             b'_' => flag = true,
//             b if b.is_ascii_alphanumeric() => {
//                 let b = match flag && b.is_ascii_alphabetic() {
//                     true => b.to_ascii_uppercase(),
//                     false => b.to_ascii_lowercase(),
//                 };
//                 buf.push(b);
//                 flag = false;
//             }
//             _ => return None,
//         }
//     }
//     String::from_utf8(buf).ok()
// }

// =============================================================

#[doc(hidden)]
#[proc_macro]
pub fn gen_clr_const(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as GenClrConst).tts.into()
}

struct GenClrConst {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for GenClrConst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut val = {
            let lit = input.parse::<syn::LitInt>()?;
            lit.base10_parse::<u8>()?
        };
        let _fat_arrow = input.parse::<Token![=>]>()?;
        let tts = syn::punctuated::Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?
            .into_iter()
            .map(|expr| {
                let ident = {
                    let mut tts = (quote!(#expr)).into_iter();
                    match (tts.next(), tts.next()) {
                        (Some(ident), None) => ident,
                        (None, Some(_)) => unreachable!(),
                        _ => {
                            return syn::Error::new_spanned(expr, "expect ident")
                                .into_compile_error();
                        }
                    }
                };
                let (fg, bg) = {
                    let s = ident.to_string();
                    let fg = proc_macro2::Ident::new(&format!("FG_{}", s), ident.span());
                    let bg = proc_macro2::Ident::new(&format!("BG_{}", s), ident.span());
                    (fg, bg)
                };
                let (fgval, bgval) = {
                    let fg = proc_macro2::Literal::u8_unsuffixed(val);
                    let bg = proc_macro2::Literal::u8_unsuffixed(val + 10);
                    val += 1;
                    (fg, bg)
                };
                quote! {
                    pub const #fg: u8 = #fgval;
                    pub const #bg: u8 = #bgval;
                }
            })
            .collect::<proc_macro2::TokenStream>();
        Ok(GenClrConst { tts })
    }
}

// =============================================================

#[doc(hidden)]
#[proc_macro]
pub fn gen_sty_const(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as GenStyConst).tts.into()
}

struct GenStyConst {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for GenStyConst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut val = {
            let lit = input.parse::<syn::LitInt>()?;
            lit.base10_parse::<u8>()?
        };
        let tts = syn::punctuated::Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?
            .into_iter()
            .map(|expr| {
                let ident = {
                    let mut tts = (quote!(#expr)).into_iter();
                    match (tts.next(), tts.next()) {
                        (Some(ident), None) => ident,
                        (None, Some(_)) => unreachable!(),
                        _ => {
                            return syn::Error::new_spanned(expr, "expect ident")
                                .into_compile_error();
                        }
                    }
                };
                let (set, unset) = {
                    let s = ident.to_string();
                    let set = proc_macro2::Ident::new(&format!("STY_{}_SET", s), ident.span());
                    let unset = proc_macro2::Ident::new(&format!("STY_{}_RST", s), ident.span());
                    (set, unset)
                };
                let (setval, unsetval) = {
                    let set = proc_macro2::Literal::u8_unsuffixed(val);
                    let unset = proc_macro2::Literal::u8_unsuffixed(val + 10);
                    val += 1;
                    (set, unset)
                };
                quote! {
                    pub const #set: u8 = #setval;
                    pub const #unset: u8 = #unsetval;
                }
            })
            .collect::<proc_macro2::TokenStream>();
        Ok(GenStyConst { tts })
    }
}

// =============================================================

/// A macro for building [SGR][wiki-sgr].
///
/// `sgr!` creates [`etty::csi::Csi`](etty::csi::Csi) type.
/// It is expected to be used in conjunction with [`etty::sgr_const`][mod-sgr-const].
///
/// ```rust
/// etty::sgr!(etty::STY_BOLD_SET, etty::FG_BLU, etty::BG_RED).out();
/// etty::out!("I'm bold and blue, my background is red");
/// etty::sgr_rst().out();
/// ```
///
/// If multiple SGR parameters to be displayed consecutively, use this macro for a shorter sequence.
///
/// ``` rust
/// let sgr = etty::sgr!(etty::STY_BOLD_SET, etty::FG_BLU, etty::BG_RED).to_string();
/// assert_eq!(sgr, "\x1b[1;34;41m");
///
/// let sgr = format!("{}{}{}", etty::sty_bold_set(), etty::fg_blu(), etty::bg_red());
/// assert_eq!(sgr, "\x1b[1m\x1b[34m\x1b[41m");
/// ````
///
/// [wiki-sgr]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
/// [mod-sgr-const]: etty::sgr_const
#[proc_macro]
pub fn sgr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Sgr).tts.into()
}

struct Sgr {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for Sgr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (fmt, exprs) = {
            let exprs =
                syn::punctuated::Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?;
            if exprs.is_empty() {
                let err = syn::parse::Error::new_spanned(exprs, "expect at least one expression");
                return Err(err);
            };
            let mut buf = Vec::<&str>::with_capacity(2 + (exprs.len() * 2));
            buf.push("\x1b[");
            for i in 0..exprs.len() {
                buf.push("{}");
                if i == exprs.len() - 1 {
                    buf.push("m");
                } else {
                    buf.push(";");
                }
            }
            (buf.concat(), exprs.into_iter())
        };
        let tts = quote! { etty::csi::Csi(std::borrow::Cow::from(std::format!(#fmt, #(#exprs as u8,)*))) };
        Ok(Sgr { tts })
    }
}

// =============================================================

/// A convenience macro for writing into [`std::io::Stdout`](std::io::Stdout).
///
/// ```rust
/// etty::out!("{}{}hello world! {}", etty::ers_all(), etty::cus_home(), "????????????!????");
/// etty::out!(etty::cus_next_ln(1));
/// etty::out!(42);
/// etty::out!('\x20');
/// etty::out!('A');
/// etty::flush();
/// ```
///
/// Please be noted that we don't need string literal as the first argument if formatting isn't necessary.
///
/// ```rust
/// etty::out!(42);        // do this
/// etty::out!("{}", 42);  // instead of this
/// ```
///
#[proc_macro]
pub fn out(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct Out(proc_macro2::TokenStream);
    impl syn::parse::Parse for Out {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let fmtargs = input.parse::<FmtArgsExprs>()?.0;
            let tts = quote! {{
                use std::io::Write;
                std::write!(std::io::stdout(), #fmtargs).unwrap();
            }};
            Ok(Out(tts))
        }
    }
    parse_macro_input!(input as Out).0.into()
}

/// Same with [`etty::macros::out!`](etty::macros::out!) but with newline.
#[proc_macro]
pub fn outln(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct Outln(proc_macro2::TokenStream);
    impl syn::parse::Parse for Outln {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let fmtargs = input.parse::<FmtArgsExprs>()?.0;
            let tts = quote! {{
                use std::io::Write;
                std::writeln!(std::io::stdout(), #fmtargs).unwrap();
            }};
            Ok(Outln(tts))
        }
    }
    parse_macro_input!(input as Outln).0.into()
}

/// Same with [`etty::macros::out!`](etty::macros::out!) but perform [`std::io::Stdout::flush`](std::io::Stdout::flush) immediately.
#[proc_macro]
pub fn outf(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct Outf(proc_macro2::TokenStream);
    impl syn::parse::Parse for Outf {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let fmtargs = input.parse::<FmtArgsExprs>()?.0;
            let tts = quote! {{
                use std::io::Write;
                std::write!(std::io::stdout(), #fmtargs).unwrap();
                std::io::stdout().flush().unwrap();
            }};
            Ok(Outf(tts))
        }
    }
    parse_macro_input!(input as Outf).0.into()
}

struct FmtArgsExprs(proc_macro2::TokenStream);

impl syn::parse::Parse for FmtArgsExprs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lead = input.parse::<syn::Expr>()?;
        let tts = if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(litstr),
            ..
        }) = lead
        {
            if !input.is_empty() {
                let _comma = input.parse::<Token![,]>()?;
            }
            let exprs =
                syn::punctuated::Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?
                    .into_iter();
            quote! { #litstr, #(#exprs,)* }
        } else {
            quote! { "{}", #lead }
        };
        Ok(FmtArgsExprs(tts))
    }
}
