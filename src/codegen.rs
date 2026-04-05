use crate::*;

/// Used to safely embed external strings into generated Rust code without risking injection attacks.
pub(crate) fn escaped_str_code(t: &str) -> String {
    let escaped = t.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Turns a [TextPart] to Rust code for Yew
pub(crate) fn text_part_to_code(text_part: &TextPart, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    match text_part {
        TextPart::Literal(t) => {
            format!("{{{}}}", escaped_str_code(t))
        }
        TextPart::Expression(id) => {
            let mut value = args.get_val(id, opts, iters, args).to_string();
            // Extract the base identifier before any field access (dot notation)
            let base_id = id.split('.').next().unwrap_or(id);
            if base_id.starts_with("opt_") || base_id.ends_with("_opt") || base_id.starts_with("iter_") || base_id.ends_with("_iter") {
                // For iterator/optional variables, replace the base identifier with the macro-produced version
                if id.contains('.') {
                    // Handle field access: replace "iter_var.field" with "macro_produced_iter_var.field"
                    let field_part = &id[base_id.len()..]; // includes the dot and field name
                    value = format!("macro_produced_{base_id}{field_part}");
                } else {
                    // Simple variable access: replace "iter_var" with "macro_produced_iter_var"
                    value = format!("macro_produced_{base_id}");
                }
            };
            format!("{{{value}}}")
        },
    }
}

/// Process attribute expressions to handle iterator/optional variables and ensure string conversion
fn process_attribute_expression(id: &str, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    let mut value = args.get_val(id, opts, iters, args).to_string();
    // Extract the base identifier before any field access (dot notation)
    let base_id = id.split('.').next().unwrap_or(id);
    if base_id.starts_with("opt_") || base_id.ends_with("_opt") || base_id.starts_with("iter_") || base_id.ends_with("_iter") {
        // For iterator/optional variables, replace the base identifier with the macro-produced version
        if id.contains('.') {
            // Handle field access: replace "iter_var.field" with "macro_produced_iter_var.field"
            let field_part = &id[base_id.len()..]; // includes the dot and field name
            value = format!("macro_produced_{base_id}{field_part}");
        } else {
            // Simple variable access: replace "iter_var" with "macro_produced_iter_var"
            value = format!("macro_produced_{base_id}");
        }
    }
    value
}

/// Turns an HTML attribute to Rust code for Yew
pub(crate) fn attr_to_code((name, value): (String, String), opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> Option<String> {
    // Remove attributes used by yew-template
    if name == "opt" || name == "present-if" || name == "loop_var" || name.starts_with("iter.") {
        return None
    }

    // Split text into text parts
    let text_parts = TextPart::parse(&value, args);

    // Generate code
    match text_parts.len() {
        0 => None,
        1 => {
            if let TextPart::Literal(text) = &text_parts[0] {
                if text == "true" || text == "false" {
                    return Some(format!("{name}={{{text}}}"))
                }
            }
            let text_part_code = match &text_parts[0] {
                TextPart::Expression(id) => {
                    let value = process_attribute_expression(id, opts, iters, args);
                    // Event-handler attributes (onclick, onchange, oninput, …) carry callbacks,
                    // not strings – wrapping them in `.to_string()` would cause a compile error.
                    if name.starts_with("on") {
                        format!("{{{value}}}")
                    } else {
                        format!("{{{value}.to_string()}}")
                    }
                }
                _ => text_parts[0].to_code(opts, iters, args)
            };
            Some(format!("{name}={text_part_code}"))
        }
        _ => {
            let mut format_literal = String::new();
            let mut format_args = Vec::new();
            for text_part in text_parts {
                match text_part {
                    TextPart::Literal(t) => format_literal.push_str(&t),
                    TextPart::Expression(ref id) => {
                        let mut value = args.get_val(id, opts, iters, args).to_string();
                        if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
                            value = value[1..value.len() - 1].to_string();
                            value = value.replace('{', "{{").replace('}', "}}");
                            format_literal.push_str(&value);
                        } else {
                            format_literal.push_str("{}");
                            // For attributes, ensure the expression is converted to string
                            let value = process_attribute_expression(id, opts, iters, args);
                            format_args.push(format!("{{{value}.to_string()}}"));
                        }
                    }
                }
            }
            let format_literal = escaped_str_code(&format_literal);
            let format_args = format_args.join(", ");
            Some(format!("{name}={{format!({format_literal}, {format_args})}}"))
        }
    }
}

/// Turns an HTML element and its children to Rust code for Yew
pub(crate) fn element_to_code(el: Element, depth: usize, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    let tabs = "    ".repeat(depth);

    // Make sure the element is valid
    if el.self_closing && !el.close_attrs.is_empty() {
        abort!(args.path_span, "Self-closing tags cannot have closing attributes");
    }
    if el.self_closing && !el.children.is_empty() {
        abort!(args.path_span, "Self-closing tags cannot have children");
    }

    // Get element properties for templating
    let opt = el.open_attrs.iter().any(|(n,_)| n=="opt");

    // Reject old bare `iter` attribute — use iter.varname={expr} instead
    if el.open_attrs.iter().any(|(n,_)| n=="iter") {
        abort!(args.path_span, "The bare `iter` attribute is no longer supported. Use `iter.varname={{expr}}` instead, e.g. <li iter.item={{items}}>…</li>");
    }

    // Check for new iter.variable={iterator} syntax
    let iter_attr = el.open_attrs.iter().find(|(n,_)| n.starts_with("iter."));
    let (iter_var_name, iter_source) = if let Some((attr_name, attr_value)) = iter_attr {
        let var_name = &attr_name[5..]; // Remove "iter." prefix
        (Some(var_name.to_string()), Some(attr_value.clone()))
    } else {
        (None, None)
    };

    // Early return for new iteration style
    if let Some(var_name) = &iter_var_name {
        return handle_new_iteration_style(el, &var_name, iter_source.as_ref().unwrap(), depth, opts, iters, args);
    }

    let present_if = el.open_attrs.iter().find(|(n,_)| n=="present-if").map(|(_,v)| v.to_owned());

    // Scan and generate children
    let mut inner_opts = Vec::new();
    let mut inner_iters = Vec::new();
    let name = el.name;
    // Strip the "name" directive from <block> elements — it is a yew-template marker,
    // not an HTML attribute, and must not appear in the generated Yew code.
    let mut f_open_attrs = el.open_attrs.into_iter()
        .filter(|(attr_name, _)| !(name == "block" && attr_name == "name"))
        .filter_map(|a| attr_to_code(a, &mut inner_opts, &mut inner_iters, args))
        .collect::<Vec<_>>().join(" ");
    if !f_open_attrs.is_empty() {
        f_open_attrs.insert(0, ' ');
    }
    let mut f_close_attrs = el.close_attrs.into_iter().filter_map(|a| attr_to_code(a, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join(" ");
    if !f_close_attrs.is_empty() {
        f_close_attrs.insert(0, ' ');
    }
    let mut content = el.children.into_iter().map(|p| p.part.into_code(depth + 1, &mut inner_opts, &mut inner_iters, args)).collect::<Vec<_>>().join("");

    inner_opts.sort();
    inner_opts.dedup();
    inner_iters.sort();
    inner_iters.dedup();

    // Handle special virtual / block elements (block is the base-template fallback wrapper)
    content = match name == "virtual" || name == "block" {
        true => {
            if !f_open_attrs.is_empty() || !f_close_attrs.is_empty() {
                abort!(args.path_span, "Virtual elements cannot have attributes (found {:?} and {:?})", f_open_attrs, f_close_attrs);
            }
            content.replace("\n    ", "\n")
        },
        false => match el.self_closing {
            true if &name == "br" => format!("<{name} {f_open_attrs}/>"),
            true => format!("\n{tabs}<{name}{f_open_attrs}/>"),
            false => format!("\n{tabs}<{name}{f_open_attrs}>{content}\n{tabs}</{name}{f_close_attrs}>"),
        }
    };

    // Handle optional elements
    match opt {
        true => {
            let left = inner_opts.iter().map(|id| format!("Some(macro_produced_{id})")).collect::<Vec<_>>().join(", ");
            let right = inner_opts.iter().map(|id| args.get_val(id, &mut Vec::new(), &mut Vec::new(), args).to_string()).collect::<Vec<_>>().join(", ");
            content = content.replace('\n', "\n    ");
            content = format!("\n{tabs}if let ({left}) = ({right}) {{ {content}\n{tabs}}}");
        },
        false => opts.extend_from_slice(&inner_opts),
    }

    iters.extend_from_slice(&inner_iters);

    // Handle optionaly present elements
    if let Some(mut present_if) = present_if {
        let not = present_if.starts_with('!');
        if not {
            present_if = present_if[1..].to_string();
        }
        let negation = if not {"!"} else {""};
        if !present_if.starts_with(&args.config.variable_bounds.0) || !present_if.ends_with(&args.config.variable_bounds.1) {
            abort!(args.path_span, "present_if attribute must be a variable");
        }
        present_if = present_if[args.config.variable_bounds.0.len()..present_if.len()-args.config.variable_bounds.1.len()].to_string();
        let val = args.get_val(&present_if, &mut Vec::new(), &mut Vec::new(), args);
        content = content.replace('\n', "\n    ");
        content = format!("\n\
            {tabs}if {negation}{{{val}}} {{\
            {tabs}{content}\n\
            {tabs}}}"
        );
    }

    content
}

/// Turns HTML text to Rust code for Yew
pub(crate) fn text_to_code(text: String, depth: usize, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    let tabs = "    ".repeat(depth);

    // If it's only a single variable then no need to translate
    let text_parts = TextPart::parse(&text, args);
    if matches!(text_parts.as_slice(), &[TextPart::Expression(_)]) {
        return text_parts[0].to_code(opts, iters, args);
    }

    // Get localized texts
    #[cfg(feature = "i18n")]
    let translations = args.catalog.translate_text(&text, args);
    #[cfg(not(feature = "i18n"))]
    let translations = vec![(String::new(), text_parts)];

    // Translations are disabled
    if translations.len() == 1 {
        return format!("\n{tabs}{}", translations[0].1.to_code(opts, iters, args));
    }

    // It's a simple case with a static string
    let mut all_are_single_literal = true;
    for (_, translation) in &translations {
        if translation.len() != 1 || !matches!(translation[0], TextPart::Literal(_)) {
            all_are_single_literal = false;
            break;
        }
    }
    let locale_code = &args.config.locale_code;
    if all_are_single_literal {
        let mut result = String::new();
        result.push_str(&format!("\n{tabs}{{match {locale_code} {{\n"));
        for (i, (locale, translation)) in translations.iter().enumerate().rev() {
            let arm = match i == 0 {
                true => String::from("_"),
                false => escaped_str_code(locale),
            };
            let text = match &translation[0] {
                TextPart::Literal(l) => l,
                _ => unreachable!(),
            };
            let text = escaped_str_code(text);
            result.push_str(&format!("{tabs}    {arm} => {text},\n"));
        }
        result.push_str(&format!("{tabs}}}}}"));
        return result;
    }

    // It's a complex case
    let mut result = String::new();
    result.push_str(&format!("\n{tabs}{{match {locale_code} {{\n"));
    for (i, (locale, translation)) in translations.iter().enumerate().rev() {
        let arm = match i == 0 {
            true => String::from("_"),
            false => escaped_str_code(locale),
        };
        let code = translation.to_code(opts, iters, args);
        result.push_str(&format!("{tabs}    {arm} => yew::html! {{ <> {code} </> }},\n"));
    }
    result.push_str(&format!("{tabs}}}}}"));

    result
}

pub(crate) fn generate_code(root: Element, args: Args) -> String {
    let yew_html = HtmlPart::Element(root).into_code(0, &mut Vec::new(), &mut Vec::new(), &args);
    let yew_code = format!("yew::html! {{ {yew_html} }}");

    yew_code
}

/// Handle the new iteration style where children repeat inside the HTML container
fn handle_new_iteration_style(el: Element, var_name: &str, iter_source: &str, depth: usize, opts: &mut Vec<String>, iters: &mut Vec<String>, args: &Args) -> String {
    let tabs = "    ".repeat(depth);

    // Parse the iterator source to get the variable name
    let iter_var = if iter_source.starts_with('{') && iter_source.ends_with('}') {
        &iter_source[1..iter_source.len()-1]
    } else {
        iter_source
    };

    // Get the iterator value using the args system
    let iter_value = args.get_val(iter_var, &mut Vec::new(), &mut Vec::new(), args);

    // Extract optional loop_var alias (default: "loop").
    // This lets callers distinguish multiple loop levels: iter.outer={x} loop_var="outer_loop"
    let loop_alias = el.open_attrs.iter()
        .find(|(n, _)| n == "loop_var")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "loop".to_string());

    // Process attributes (filtering out iter.* and loop_var attributes — attr_to_code also
    // filters loop_var, but we pre-filter here for clarity)
    let mut inner_opts = Vec::new();
    let mut inner_iters = Vec::new();
    let mut f_open_attrs = el.open_attrs.into_iter()
        .filter(|(n,_)| !n.starts_with("iter.") && n != "loop_var")
        .filter_map(|a| attr_to_code(a, &mut inner_opts, &mut inner_iters, args))
        .collect::<Vec<_>>()
        .join(" ");
    if !f_open_attrs.is_empty() {
        f_open_attrs.insert(0, ' ');
    }
    let mut f_close_attrs = el.close_attrs.into_iter()
        .filter_map(|a| attr_to_code(a, &mut inner_opts, &mut inner_iters, args))
        .collect::<Vec<_>>()
        .join(" ");
    if !f_close_attrs.is_empty() {
        f_close_attrs.insert(0, ' ');
    }

    let name = el.name;

    // Compute this loop's depth and register it under the alias so that get_val can
    // resolve alias.depth / alias.index1 etc. even from inside deeply nested loops.
    // wrapping_add: initialised to usize::MAX so first increment yields 0.
    let prev_loop_depth = args.loop_depth.get();
    let new_depth = prev_loop_depth.wrapping_add(1);
    args.loop_depth.set(new_depth);
    args.loop_aliases.borrow_mut().insert(loop_alias.clone(), new_depth);

    // Generate prefixed Rust variable names for this loop level.
    // Using the alias as a prefix (e.g. "_yew_outer_loop_") means inner loops with a
    // different alias don't shadow these names, so outer vars remain accessible.
    let p = format!("_yew_{loop_alias}");

    let children_content = el.children.into_iter()
        .map(|p_inner| p_inner.part.into_code(depth + 1, &mut inner_opts, &mut inner_iters, args))
        .collect::<Vec<_>>()
        .join("");
    args.loop_depth.set(prev_loop_depth);
    // Note: alias entry intentionally NOT removed from loop_aliases after exit — this lets
    // a child template snippet reference an outer alias by name and still get the correct depth.

    let children_content = children_content.replace('\n', "\n            ");

    opts.extend_from_slice(&inner_opts);
    iters.extend_from_slice(&inner_iters);

    // Generate code that creates the container with repeated children inside.
    // Iterator is collected upfront so alias.length / alias.last / alias.previtem / alias.nextitem
    // are available without a second pass.
    format!("\n{tabs}<{name}{f_open_attrs}>{{\n\
             {tabs}    {{\n\
             {tabs}        let {p}_items: Vec<_> = ({iter_value}).collect();\n\
             {tabs}        let {p}_len = {p}_items.len();\n\
             {tabs}        let mut {p}_children = Vec::new();\n\
             {tabs}        for ({p}_idx, {p}_item) in {p}_items.iter().enumerate() {{\n\
             {tabs}            let {var_name} = *{p}_item;\n\
             {tabs}            let {p}_index0 = {p}_idx;\n\
             {tabs}            let {p}_index = {p}_idx + 1;\n\
             {tabs}            let {p}_first = {p}_idx == 0;\n\
             {tabs}            let {p}_last = {p}_idx + 1 == {p}_len;\n\
             {tabs}            let {p}_length = {p}_len;\n\
             {tabs}            let {p}_previtem = if {p}_idx > 0 {{ Some({p}_items[{p}_idx - 1]) }} else {{ None }};\n\
             {tabs}            let {p}_nextitem = {p}_items.get({p}_idx + 1).copied();\n\
             {tabs}            {p}_children.push(yew::html! {{ <>\
             {children_content}\n\
             {tabs}            </> }});\n\
             {tabs}        }}\n\
             {tabs}        yew::Html::from_iter({p}_children)\n\
             {tabs}    }}\n\
             {tabs}}}</{name}{f_close_attrs}>")
}
