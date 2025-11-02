use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Marks a function as a previewable component.
///
/// Can be used with or without parameters:
/// ```rust
/// // No parameters - function must take no arguments
/// #[snowscape::preview]
/// pub fn my_component() -> Element<'_, Message> { ... }
///
/// // Single parameter set
/// #[snowscape::preview("Hello")]
/// pub fn my_text(text: &str) -> Element<'_, Message> { ... }
///
/// // Multiple parameter sets (stack multiple attributes)
/// #[snowscape::preview("Hello")]
/// #[snowscape::preview("World")]
/// pub fn my_text(text: &str) -> Element<'_, Message> { ... }
/// ```
#[proc_macro_attribute]
pub fn preview(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Parse attributes - for now just check if empty
    let attr_str = attr.to_string();
    let has_params = !attr_str.trim().is_empty();

    // Extract the function details
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_generics = &input.sig.generics;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;

    // Generate a unique preview name and label
    let (preview_label, fn_call) = if !has_params {
        let label = format!("{}", fn_name);
        let call = quote! { #fn_name() };
        (label, call)
    } else {
        // Parse the attribute string to extract literal value
        let param_str = attr_str.trim().trim_matches('"');
        let label = format!("{}({:?})", fn_name, param_str);

        // Generate function call with the parameter
        let param_tokens: proc_macro2::TokenStream = attr_str.parse().unwrap();
        let call = quote! { #fn_name(#param_tokens) };
        (label, call)
    };

    // Generate a unique function name for the preview creator
    // Include a hash of the parameters to make it unique for each preview variant
    let param_hash = if has_params {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        attr_str.hash(&mut hasher);
        hasher.finish()
    } else {
        0
    };

    let preview_fn_name = syn::Ident::new(
        &format!("__snowscape_preview_create_{}_{:x}", fn_name, param_hash),
        fn_name.span(),
    );

    // Keep the original function and add preview registration
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            #fn_block
        }

        // Generate a standalone function for creating the preview
        fn #preview_fn_name() -> Box<dyn ::snowscape::Preview> {
            Box::new(::snowscape::preview::StatelessPreview::new(|| {
                use ::iced::Element;
                (#fn_call).map(|_| ::snowscape::Message::Noop)
            }))
        }

        // Generate the preview registration using a function pointer
        ::snowscape::inventory::submit! {
            ::snowscape::preview::Descriptor {
                metadata: ::snowscape::Metadata::new(#preview_label),
                create: #preview_fn_name,
            }
        }
    };

    TokenStream::from(expanded)
}
