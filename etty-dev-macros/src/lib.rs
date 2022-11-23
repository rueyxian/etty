use quote::quote;
use syn::parse::ParseStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn gen_e10(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct Tts(proc_macro2::TokenStream);
    impl syn::parse::Parse for Tts {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let nm = input.parse::<proc_macro2::Ident>()?;
            const LEN: usize = 21;
            let mut e10: u128 = 1;
            let lits = (0..LEN).map(|_| {
                let lit = proc_macro2::Literal::u128_unsuffixed(e10);
                e10 *= 10;
                lit
            });
            let len = proc_macro2::Literal::usize_unsuffixed(LEN);
            let tts = quote! { const #nm : [u128; #len] = [ #(#lits,)* ]; };
            Ok(Tts(tts))
        }
    }
    parse_macro_input!(input as Tts).0.into()
}

// struct GenCsi {
//     tts: proc_macro2::TokenStream,
// }

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
