use quote::quote;
use syn::parse_macro_input;
use syn::Token;

#[proc_macro]
pub fn gen_csi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as GenCsi).tts.into()
}

struct GenCsi {
    tts: proc_macro2::TokenStream,
}

// impl syn::parse::Parse for GenCsi {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         use syn::punctuated::Punctuated;
//         let tts = Punctuated::<Csi, Token![;]>::parse_terminated(input)?
//             .into_iter()
//             .map(|csi| {
//                 let tts = csi.tts;
//                 quote!(#tts)
//             })
//             .collect::<proc_macro2::TokenStream>();
//         Ok(GenCsi { tts })
//     }
// }

impl syn::parse::Parse for GenCsi {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::punctuated::Punctuated;
        let mod_visi = input.parse::<syn::Visibility>()?;
        let _mod = input.parse::<Token![mod]>()?;
        let mod_nm = input.parse::<proc_macro2::Ident>()?;
        let _semi = input.parse::<Token![;]>()?;

        let csis = Punctuated::<Csi, Token![;]>::parse_terminated(input)?;
        let csi_import = csis
            .iter()
            .filter_map(|csi| {
                if let syn::Visibility::Inherited = csi.visi {
                    None
                } else {
                    let visi = &csi.visi;
                    let nm_snake = &csi.nm_snake;
                    Some(quote!(#visi use #mod_nm::#nm_snake;))
                }
            })
            .collect::<proc_macro2::TokenStream>();
        let csi_impls = csis
            .iter()
            .map(|csi| {
                let tts = &csi.tts;
                quote!(#tts)
            })
            .collect::<proc_macro2::TokenStream>();
        let tts = quote! {
            #csi_import
            #mod_visi mod #mod_nm {
                use std::io::Write;
                #csi_impls
            }
        };
        Ok(GenCsi { tts })
    }
}

struct Csi {
    visi: syn::Visibility,
    nm_snake: proc_macro2::Ident,
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for Csi {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let visi = input.parse::<syn::Visibility>()?;
        // let visi = proc_macro2::Ident::new("pub", proc_macro2::Span::call_site());
        let (nm_pascal, nm_snake) = {
            let ident = input.parse::<syn::Ident>()?;
            let pascal = match snake_to_pascal(&ident.to_string()) {
                Some(s) => proc_macro2::Ident::new(&s, ident.span()),
                None => return Err(syn::Error::new_spanned(ident, "expect snake case")),
            };
            (pascal, ident)
        };
        let _fat_arrow = input.parse::<Token![=>]>()?;
        let CsiFmtTmp { fmt, nms_ord } = input.parse::<CsiFmtTmp>()?;

        let args = {
            #[derive(Clone)]
            struct Arg {
                nm: proc_macro2::Ident,
                ty: proc_macro2::Ident,
            }
            let mut args = Vec::<Arg>::with_capacity(nms_ord.len());
            for _ in 0..nms_ord.len() {
                let _comma = input.parse::<Token![,]>()?;
                let CsiArgTmp { nm, ty } = input.parse::<CsiArgTmp>()?;
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

        let fmt_arg_nms = {
            let mut arg_nms = Vec::<proc_macro2::Ident>::with_capacity(args.len());
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
                arg_nms.push(arg.nm);
            }
            arg_nms
        };

        let tts = {
            let struct_tts = match args.is_empty() {
                true => quote! { #visi struct #nm_pascal; },
                false => {
                    let tys = args.iter().map(|arg| arg.ty.clone());
                    quote! { #visi struct #nm_pascal(#(#tys,)*); }
                }
            };
            let impl_display_tts = {
                let write_args = (0..args.len()).map(|num| {
                    let lit = proc_macro2::Literal::usize_unsuffixed(num);
                    quote! { self.#lit }
                });
                quote! {
                    impl std::fmt::Display for #nm_pascal {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            std::write!(f, #fmt, #(#write_args,)*)
                        }
                    }
                }
            };
            let impl_methods_tts = {
                let path = quote! { crate::csi:: };
                quote! {
                    impl #path Csi for #nm_pascal {}
                    // impl #nm_pascal {
                    //     #visi fn write(&self) {
                    //         std::write!(std::io::stdout(), "{}", self).unwrap();
                    //     }
                    //     #visi fn bytes(&self) -> std::vec::Vec<u8> {
                    //         self.to_string().into_bytes()
                    //     }
                    // }
                }
            };
            let factory_tts = {
                let arg_exprs = args.iter().map(|arg| {
                    let nm = &arg.nm;
                    let ty = &arg.ty;
                    quote! { #nm: #ty }
                });
                let ret = match args.is_empty() {
                    true => quote! { #nm_pascal },
                    false => quote! { #nm_pascal(#(#fmt_arg_nms,)*) },
                };
                quote! { #visi fn #nm_snake (#(#arg_exprs,)*) -> #nm_pascal{ #ret } }
            };
            quote! {
                #factory_tts
                #struct_tts
                #impl_methods_tts
                #impl_display_tts
            }
        };
        Ok(Csi {
            visi,
            nm_snake,
            tts,
        })
    }
}

struct CsiArgTmp {
    nm: proc_macro2::Ident,
    ty: proc_macro2::Ident,
}

impl syn::parse::Parse for CsiArgTmp {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        static INT_TYS: [&str; 6] = ["u8", "u16", "u32", "u64", "u128", "usize"];
        let nm = input.parse::<proc_macro2::Ident>()?;
        let Ok(_colon) = input.parse::<Token![:]>() else {
            let ty = proc_macro2::Ident::new("u16", proc_macro2::Span::call_site());
            return Ok(CsiArgTmp { nm, ty })
        };
        let ty = input.parse::<proc_macro2::Ident>()?;
        if !INT_TYS.contains(&ty.to_string().as_str()) {
            let msg = format!("expect {:?}", INT_TYS);
            return Err(syn::Error::new_spanned(ty, msg));
        }
        Ok(CsiArgTmp { nm, ty })
    }
}

struct CsiFmtTmp {
    fmt: proc_macro2::Literal,
    nms_ord: Vec<String>,
}

impl syn::parse::Parse for CsiFmtTmp {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let litstr = input.parse::<syn::LitStr>()?;
        let s = litstr.value();
        let mut bytes = s.bytes();
        let mut nms_ord = Vec::<String>::new();
        let mut fmtbuf = {
            let mut v = Vec::<u8>::with_capacity(s.len());
            v.push(b'\x1b');
            v.push(b'[');
            v
        };
        let mut nmbuf = Vec::<u8>::new();
        while let Some(byte) = bytes.next() {
            match byte {
                b @ b'{' => {
                    fmtbuf.push(b);
                    'cb: loop {
                        match bytes.next() {
                            None | Some(b'{') => {
                                return Err(syn::Error::new_spanned(
                                    litstr,
                                    "expect a closing brace `}`",
                                ));
                            }
                            Some(b @ b'}') => {
                                fmtbuf.push(b);
                                let nm = if nmbuf.is_empty() {
                                    todo!()
                                } else {
                                    let bytes = nmbuf.drain(..).collect::<Vec<u8>>();
                                    String::from_utf8(bytes).unwrap()
                                };
                                nms_ord.push(nm);
                                break 'cb;
                            }
                            Some(b) => {
                                nmbuf.push(b);
                                continue;
                            }
                        }
                    }
                }
                b @ b'}' => {
                    fmtbuf.push(b);
                    return Err(syn::Error::new_spanned(litstr, "expect a `{` before a `}`"));
                }
                b => {
                    fmtbuf.push(b);
                    continue;
                }
            };
        }
        let fmt = {
            let s = String::from_utf8(fmtbuf).unwrap();
            proc_macro2::Literal::string(&s)
        };
        Ok(CsiFmtTmp { fmt, nms_ord })
    }
}

fn snake_to_pascal(s: &str) -> Option<String> {
    let mut buf = Vec::<u8>::with_capacity(s.len());
    let mut bytes = s.bytes();
    let b = bytes
        .by_ref()
        .skip_while(|b| *b == b'_')
        .find(|b| b.is_ascii_alphabetic())?;
    buf.push(b.to_ascii_uppercase());
    let mut flag = false;
    for b in bytes {
        match b {
            b'_' => flag = true,
            b if b.is_ascii_alphanumeric() => {
                let b = match flag && b.is_ascii_alphabetic() {
                    true => b.to_ascii_uppercase(),
                    false => b.to_ascii_lowercase(),
                };
                buf.push(b);
                flag = false;
            }
            _ => return None,
        }
    }
    String::from_utf8(buf).ok()
}

// =============================================================

#[proc_macro]
pub fn gen_color_const(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as GenColorConst).tts.into()
}

struct GenColorConst {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for GenColorConst {
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
        Ok(GenColorConst { tts })
    }
}

// =============================================================

#[proc_macro]
pub fn gen_style_const(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as GenStyleConst).tts.into()
}

struct GenStyleConst {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for GenStyleConst {
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
        Ok(GenStyleConst { tts })
    }
}

// =============================================================

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
        let tts = quote! {{
             use std::io::Write;
            std::write!(std::io::stdout(), #fmt, #(#exprs as u8,)*).unwrap();
        }};
        Ok(Sgr { tts })
    }
}

// =============================================================

#[proc_macro]
pub fn sgr_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as SgrBytes).tts.into()
}

struct SgrBytes {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for SgrBytes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let exprs = syn::punctuated::Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?;
        if exprs.is_empty() {
            let err = syn::parse::Error::new_spanned(exprs, "expect at least one expression");
            return Err(err);
        };
        let exprs = exprs.into_iter();
        let tts = quote! {{
            let u8ints = [#(#exprs as u8),*];
            const E10: [u16; 4] = [1, 10, 100, 1000];
            let mut buf = Vec::<u8>::with_capacity(u8ints.len() * 3);
            for (idx, &int) in u8ints.iter().enumerate() {
                let d = (f32::log10(int as f32) + 1.0) as usize;
                for i in (0..d).rev() {
                    let d = ((int as u16 % E10[i + 1]) / E10[i]) as u8;
                    buf.push(d + 48);
                }
                if idx < u8ints.len() - 1 {
                    buf.push(b';');
                } else {
                    buf.push(b'm');
                }
            }
            [b'\x1b', b'['].into_iter().chain(buf.into_iter())
        }};
        Ok(SgrBytes { tts })
    }
}

// =============================================================

#[proc_macro]
pub fn write_fmt(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as WriteFmt).tts.into()
}

struct WriteFmt {
    tts: proc_macro2::TokenStream,
}

impl syn::parse::Parse for WriteFmt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let exprs = syn::punctuated::Punctuated::<syn::Expr, Token![,]>::parse_terminated(input)?
            .into_iter();
        let tts = quote! {{
            use std::io::Write;
            std::write!(std::io::stdout(), #(#exprs,)*).unwrap();
        }};
        Ok(WriteFmt { tts })
    }
}
