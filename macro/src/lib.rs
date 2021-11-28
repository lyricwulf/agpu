extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(VertexLayout)]
pub fn vertex_layout(item: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: syn::DeriveInput = syn::parse(item).unwrap();

    let name = &ast.ident;

    let vertex_step_mode = quote! { ::wgpu::VertexStepMode::Vertex };

    // Get fields/types
    let vertex_attrs = match attrs_from_fields(&ast) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let gen = quote! {
       // TODO: impl TRAIT
        impl ::agpu::buffer::VertexLayout for #name {
            // Import trait
            fn vertex_buffer_layout<const L: u32>() -> wgpu::VertexBufferLayout<'static> {
                use ::agpu::VertexFormatType;
                wgpu::VertexBufferLayout {
                    array_stride: ::std::mem::size_of::<Self>() as ::wgpu::BufferAddress,
                    step_mode: #vertex_step_mode,
                    attributes: &[
                        #vertex_attrs
                    ],
                }
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(VertexLayoutInstance)]
pub fn vertex_layout_instance(item: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: syn::DeriveInput = syn::parse(item).unwrap();

    let name = &ast.ident;

    let vertex_step_mode = quote! { ::wgpu::VertexStepMode::Instance };

    // Get fields/types
    let vertex_attrs = match attrs_from_fields(&ast) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let gen = quote! {
       // TODO: impl TRAIT
        impl ::agpu::buffer::VertexLayout for #name {
            // Import trait
            fn vertex_buffer_layout<const L: u32>() -> wgpu::VertexBufferLayout<'static> {
                use ::agpu::VertexFormatType;
                wgpu::VertexBufferLayout {
                    array_stride: ::std::mem::size_of::<Self>() as ::wgpu::BufferAddress,
                    step_mode: #vertex_step_mode,
                    attributes: &[
                        #vertex_attrs
                    ],
                }
            }
        }
    };

    gen.into()
}

fn attrs_from_fields(ast: &syn::DeriveInput) -> Result<quote::__private::TokenStream, TokenStream> {
    let mut vertex_attrs = quote! {};
    let fields = match &ast.data {
        syn::Data::Struct(data) => &data.fields,
        // TODO: Maybe support enum? union?
        _ => return Err(quote! { compile_error!("Vertex buffer must be struct")}.into()),
    };
    let mut offset = quote! { 0 };
    for (i, field) in fields.iter().enumerate() {
        let ty = &field.ty;

        // TODO: Skip zero-sized fields
        // ZSF do not affect layout so do not need an entry
        // Somehow need to evaluate the size_of in this macro
        // If this is not possible, then we can use a type blacklist (unpreferred)
        //// if size_of<ty>() == 0 { continue; }

        // Add the vertex attribute entry
        vertex_attrs.extend(quote! {
            wgpu::VertexAttribute {
                offset: #offset,
                shader_location: L + #i as u32,
                format: <#ty>::VERTEX_FORMAT_TYPE,
            },
        });

        // Add the size of this field to the cumulative offset
        offset.extend(quote! {+ ::std::mem::size_of::<#ty>() as u64});
    }
    Ok(vertex_attrs)
}

// /// Determine step mode, if per_instance is set, then we will use instance, otherwise vertex
// fn get_vertex_step_mode(ast: &syn::DeriveInput) -> quote::__private::TokenStream {
//     let vertex_step_mode = {
//         let step_per_instance = ast
//             .attrs
//             .iter()
//             .any(|attr| attr.path.segments.last().unwrap().ident.to_string() == "per_instance");
//         if step_per_instance {
//             quote! {
//                 ::wgpu::VertexStepMode::Instance
//             }
//         } else {
//             quote! {

//             }
//         }
//     };
//     vertex_step_mode
// }
