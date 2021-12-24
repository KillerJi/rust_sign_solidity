use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct,
    DeriveInput, Fields, Ident, Lit,
};
fn expand_sign_v4(
    ident: Ident,
    data: DataStruct,
    attrs: Vec<Attribute>,
) -> Result<TokenStream, Box<dyn std::error::Error>> {
    let mut is_primary_type = false;
    let mut dapp_domain_info: Punctuated<_, Comma> = Punctuated::new();
    for attr in attrs.iter() {
        if attr.path.get_ident().map(|i| i == "primary_type") == Some(true) {
            is_primary_type = true;
        }
        if attr.path.get_ident().map(|i| i == "domain_712") != Some(true) {
            continue;
        }
        if let Ok(list) = attr.parse_args_with(Punctuated::<Lit, Comma>::parse_terminated) {
            dapp_domain_info.extend(list.into_iter());
            break;
        }
    }
    let mut dapp_params_info: Punctuated<_, Comma> = Punctuated::new();
    let mut dapp_params_array: Punctuated<_, Comma> = Punctuated::new();
    if let Fields::Named(fields) = &data.fields {
        for field in fields.named.iter() {
            let field_name = field.ident.clone().ok_or("ident must be set")?;
            if let Some(attr) = field
                .attrs
                .iter()
                .find(|&v| format!("{}", v.path.to_token_stream()) == String::from("web3_type"))
            {
                let sol_type = attr.parse_args::<Lit>()?;
                if let Lit::Str(s) = sol_type {
                    let ty_str = s.value();
                    let ty: TokenStream = syn::parse_str(&ty_str)?;
                    dapp_params_info.push(quote! {
                        web3::ethabi::Param {
                            name: stringify!(#field_name).to_string(),
                            kind: #ty
                        }
                    });
                    let token = match ty_str.as_str() {
                        "ParamType::Address" => {
                            quote! {web3::ethabi::Token::Address(self.#field_name)}
                        }
                        "ParamType::Bytes" => {
                            quote! {web3::ethabi::Token::Bytes(self.#field_name)}
                        }
                        "ParamType::Bool" => {
                            quote! {web3::ethabi::Token::Bool(self.#field_name)}
                        }
                        "ParamType::String" => {
                            quote! {web3::ethabi::Token::String(self.#field_name)}
                        }
                        "ParamType::Array" => {
                            quote! {web3::ethabi::Token::Array(self.#field_name)}
                        }
                        "ParamType::FixedBytes" => {
                            quote! {web3::ethabi::Token::FixedBytes(self.#field_name)}
                        }
                        "ParamType::FixedArray" => {
                            quote! {web3::ethabi::Token::FixedArray(self.#field_name)}
                        }
                        "ParamType::Tuple" => {
                            quote! {web3::ethabi::Token::Tuple(self.#field_name)}
                        }
                        "ParamType::Uint(256)" => {
                            quote! {web3::ethabi::Token::Uint(web3::ethabi::Uint::from(self.#field_name))}
                        }
                        "ParamType::Int(256)" => {
                            quote! {web3::ethabi::Token::Int(web3::ethabi::Int::from(self.#field_name))}
                        }
                        _ => {
                            panic!("illegal type")
                        }
                    };
                    dapp_params_array.push(token);
                } else {
                    panic!("field attributes must string");
                }
            }
        }
    }
    let dapp_domain_info: TokenStream = if is_primary_type {
        quote! { Some((#dapp_domain_info)) }
    } else {
        quote! { None }
    };
    Ok(quote! {
        impl #ident {
            fn get_domain_type_hash() -> [u8; 32] {
                let name = "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";
                let data: Vec<u8> = From::from(name.to_string());
                web3::signing::keccak256(&data)
            }
            fn encode_domain_abi(chain_id: &str, address: web3::types::H160) -> std::result::Result<std::vec::Vec<u8>, std::boxed::Box<dyn std::error::Error>> {
                let (name, version) = #dapp_domain_info.ok_or("not primary type")?;
                let chain_id = web3::types::U256::from_str_radix(chain_id, 16)?;
                let tokens = [
                    web3::ethabi::Token::FixedBytes(web3::signing::keccak256(name.as_bytes()).to_vec()),
                    web3::ethabi::Token::FixedBytes(web3::signing::keccak256(version.as_bytes()).to_vec()),
                    web3::ethabi::Token::Uint(chain_id),
                    web3::ethabi::Token::Address(address),
                ];
                let encoded = web3::ethabi::encode(&tokens);
                let signed = Self::get_domain_type_hash();
                Ok(signed.into_iter().chain(encoded.into_iter()).collect())
            }
            fn domain_separator(chain_id: &str, address: web3::types::H160) -> std::result::Result<[u8; 32], std::boxed::Box<dyn std::error::Error>> {
                let abi = Self::encode_domain_abi(chain_id, address)?;
                Ok(web3::signing::keccak256(&abi))
            }
            fn get_type() -> std::string::String {
                let params = [#dapp_params_info].iter()
                    .map(|v| format!("{} {}", v.kind, v.name))
                    .collect::<std::vec::Vec<std::string::String>>()
                    .join(",");
                format!("{}({})", stringify!(#ident), params)
            }
            fn encode_abi(&self) -> std::result::Result<std::vec::Vec<u8>, web3::ethabi::Error> {
                let params: std::vec::Vec<web3::ethabi::ParamType> = [#dapp_params_info].iter().map(|p| p.kind.clone()).collect();
                let tokens = [#dapp_params_array];
                if !web3::ethabi::Token::types_check(&tokens, &params) {
                    return Err(web3::ethabi::Error::InvalidData);
                }
                let data: std::vec::Vec<u8> = From::from(Self::get_type().as_str());
                let signed = web3::signing::keccak256(&data);
                let _tokens = tokens
                    .iter()
                    .map(|v| {
                        if let web3::ethabi::Token::String(s) = v {
                            let x: std::vec::Vec<u8> = s.as_bytes().to_vec();
                            web3::ethabi::Token::FixedBytes(web3::signing::keccak256(&x).to_vec())
                        } else {
                            v.clone()
                        }
                    })
                    .collect::<std::vec::Vec<web3::ethabi::Token>>();
                let encoded = web3::ethabi::encode(&_tokens);
                Ok(signed.into_iter().chain(encoded.into_iter()).collect())
            }
            pub fn sign_hash(&self, chain_id: &str, address: web3::types::H160) -> std::result::Result<[u8; 32], std::boxed::Box<dyn std::error::Error>> {
                let domain_separator = Self::domain_separator(chain_id, address)?;
                let abi = self.encode_abi()?;
                let struct_hash = web3::signing::keccak256(&abi);

                let mut hash = vec![0x19u8, 0x01u8];
                hash.extend(domain_separator);
                hash.extend(struct_hash);
                Ok(web3::signing::keccak256(&hash))
            }
        }
    })
}
#[proc_macro_derive(SignV4, attributes(domain_712, web3_type, primary_type))]
pub fn signv4_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input as DeriveInput);
    let data = match data {
        Data::Struct(ds) => ds,
        _ => panic!("SignV4 must be derived on a struct"),
    };
    expand_sign_v4(ident, data, attrs).unwrap().into()
}
