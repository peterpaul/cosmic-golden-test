use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::Error;
use syn::ExprArray;
use syn::ItemFn;
use syn::LitInt;
use syn::ReturnType;
use syn::Stmt;
use syn::Token;
use syn::parse::ParseStream;

/// Turns a function that returns a `cosmic::Element` into a golden image test.
///
/// The snapshot name is derived from the function name. Width and height (in pixels)
/// are required arguments. Two optional named arguments follow:
///
/// - A theme keyword — `light` (default) or `dark`.
/// - `events = [expr, ...]` — a list of `(Event, mouse::Cursor)` pairs passed to
///   `render_with_events`, used to drive the widget into an interactive state
///   (e.g. open a `combo_box` dropdown or reveal a `tooltip`) before capturing the
///   snapshot.
///
/// # Examples
///
/// ```rust,ignore
/// #[golden_test(320, 60)]
/// fn my_widget_light() -> cosmic::Element<'_, ()> {
///     cosmic::widget::text("Hello").into()
/// }
///
/// #[golden_test(320, 60, dark)]
/// fn my_widget_dark() -> cosmic::Element<'_, ()> {
///     cosmic::widget::text("Hello").into()
/// }
///
/// #[golden_test(320, 200, events = [cosmic_golden::left_click(160.0, 20.0)])]
/// fn combo_box_open() -> cosmic::Element<'_, &'static str> {
///     let state = cosmic::widget::combo_box::State::new(vec!["Alpha", "Beta", "Gamma"]);
///     cosmic::widget::combo_box(&state, "Pick an option", None, |s| s).into()
/// }
///
/// #[golden_test(320, 200, dark, events = [cosmic_golden::left_click(160.0, 20.0)])]
/// fn combo_box_open_dark() -> cosmic::Element<'_, &'static str> {
///     let state = cosmic::widget::combo_box::State::new(vec!["Alpha", "Beta", "Gamma"]);
///     cosmic::widget::combo_box(&state, "Pick an option", None, |s| s).into()
/// }
/// ```
#[proc_macro_attribute]
pub fn golden_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    match golden_test_impl(attr, item) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

struct Args {
    width: LitInt,
    height: LitInt,
    /// `None` means default (light).
    theme: Option<Ident>,
    /// `None` means no events — use `render` instead of `render_with_events`.
    events: Option<ExprArray>,
}

fn parse_args(input: ParseStream) -> syn::Result<Args> {
    let width: LitInt = input.parse()?;
    let _: Token![,] = input.parse()?;
    let height: LitInt = input.parse()?;

    let mut theme = None;
    let mut events = None;

    while input.peek(Token![,]) {
        let _: Token![,] = input.parse()?;
        if input.is_empty() {
            break;
        }
        let key: Ident = input.parse()?;
        if key == "events" {
            let _: Token![=] = input.parse()?;
            events = Some(input.parse::<ExprArray>()?);
        } else {
            theme = Some(key);
        }
    }

    Ok(Args {
        width,
        height,
        theme,
        events,
    })
}

fn golden_test_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let Args {
        width,
        height,
        theme,
        events,
    } = syn::parse::Parser::parse(parse_args, attr)?;

    let theme_expr = match theme.as_ref().map(|id| id.to_string()).as_deref() {
        Some("dark") => quote! { cosmic::Theme::dark() },
        Some("light") | None => quote! { cosmic::Theme::light() },
        Some(other) => {
            return Err(Error::new_spanned(
                theme.as_ref().unwrap(),
                format!("unknown theme '{other}': expected `dark` or `light`"),
            ));
        }
    };

    let func = syn::parse::<ItemFn>(item)?;

    if !func.sig.inputs.is_empty() {
        return Err(Error::new_spanned(
            &func.sig.inputs,
            "golden_test functions must take no parameters",
        ));
    }

    let func_name = &func.sig.ident;
    let name_str = func_name.to_string();

    let return_type = match &func.sig.output {
        ReturnType::Type(_, ty) => ty.as_ref(),
        ReturnType::Default => {
            return Err(Error::new_spanned(
                &func.sig,
                "golden_test functions must have an explicit return type",
            ));
        }
    };

    // Flatten the function body into the test body: emit all setup statements
    // as-is, then bind the tail expression to `element`.  This keeps any
    // locals (e.g. `let component = MyComponent::new()`) alive for the entire
    // test function, so elements that borrow them (via `component.view()`) do
    // not dangle.
    let stmts = &func.block.stmts;
    let (setup_stmts, element_expr) = match stmts.split_last() {
        Some((Stmt::Expr(expr, None), rest)) => (rest, expr),
        _ => {
            return Err(Error::new_spanned(
                &func.block,
                "golden_test function body must end with an expression (not a `;`-terminated statement)",
            ));
        }
    };

    let render_call = if let Some(events) = &events {
        quote! { renderer.render_with_events(element, #width, #height, &#events) }
    } else {
        quote! { renderer.render(element, #width, #height) }
    };

    Ok(quote! {
        #[test]
        fn #func_name() {
            cosmic_golden::init();
            #(#setup_stmts)*
            let element: #return_type = #element_expr;
            let mut renderer = cosmic_golden::HeadlessRenderer::with_theme(#theme_expr);
            let rgba = #render_call;
            cosmic_golden::assert_snapshot_rgba!(#name_str, rgba, #width, #height);
        }
    }
    .into())
}
