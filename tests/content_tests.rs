use yew::virtual_dom::VNode;
use yew_template::*;
use yew::prelude::*;
use yew::ServerRenderer;

/// Verifies that event-handler attributes (onclick, onchange, …) are emitted as-is
/// and NOT wrapped in `.to_string()`, which would make them fail to compile.
#[test]
fn test_event_handler_attribute_not_coerced_to_string() {
    let on_click = Callback::from(|_: MouseEvent| {});
    // This must compile: the generated code should be  onclick={on_click}
    // not  onclick={on_click.to_string()}  (Callback does not implement Display).
    let _html = template_html!("templates/button_click.html", on_click, ...);
    println!("✓ button_click.html event-handler attribute compiled successfully");
}

// ── template inheritance tests ───────────────────────────────────────────────

#[function_component]
fn InheritanceApp() -> Html {
    let title = "My Page";
    let name = "World";
    template_html!("templates/inheritance/child.html", title, name, ...)
}

#[function_component]
fn BaseDirectApp() -> Html {
    template_html!("templates/inheritance/base.html", ...)
}

#[tokio::test(flavor = "current_thread")]
async fn test_template_inheritance() {
    let child_rendered  = ServerRenderer::<InheritanceApp>::new().render().await;
    let base_rendered   = ServerRenderer::<BaseDirectApp>::new().render().await;
    println!("Inheritance child HTML: {}", child_rendered);
    println!("Inheritance base HTML:  {}", base_rendered);

    // Child overrides: header and content blocks replaced
    assert!(child_rendered.contains("My Page"),    "overridden header should contain title");
    assert!(child_rendered.contains("Hello, World!"), "overridden content block applied");

    // Footer not overridden → base default still present
    assert!(child_rendered.contains("Default Footer"), "default footer from base should be kept");

    // The base layout structure comes through
    assert!(child_rendered.contains(r#"class="page""#), "base wrapper class must be present");

    // Default content NOT present (was replaced by child override)
    assert!(!child_rendered.contains("No content provided"), "default content should be replaced");
    assert!(!child_rendered.contains("Default Title"),        "default title should be replaced");

    // Base used directly — all defaults render
    assert!(base_rendered.contains("Default Title"),     "direct base use renders default header");
    assert!(base_rendered.contains("No content provided"), "direct base use renders default content");
    assert!(base_rendered.contains("Default Footer"),    "direct base use renders default footer");
}

// ── nested loop tests ─────────────────────────────────────────────────────────

#[function_component]
fn NestedLoopApp() -> Html {
    let outers = vec!["A", "B"];
    let inners = vec![1i32, 2, 3];
    template_html!("templates/nested_loop.html", outers={outers.iter()}, inners={inners.iter()}, ...)
}

#[tokio::test(flavor = "current_thread")]
async fn test_nested_loops() {
    let rendered = ServerRenderer::<NestedLoopApp>::new().render().await;
    println!("Nested-loop rendered HTML: {}", rendered);

    // Outer loop: outer_loop alias, index1 and depth=0
    assert!(rendered.contains("outer=1 depth=0"), "outer first item, correct index1 and depth");
    assert!(rendered.contains("outer=2 depth=0"), "outer second item, correct index1 and depth");

    // Inner loop: inner_loop alias; index1 resets each outer iteration, depth=1
    // Two outer items × three inner items = 2 occurrences each
    assert_eq!(rendered.matches("inner=1 depth=1").count(), 2, "inner first item appears once per outer");
    assert_eq!(rendered.matches("inner=2 depth=1").count(), 2, "inner second item appears once per outer");
    assert_eq!(rendered.matches("inner=3 depth=1").count(), 2, "inner third item appears once per outer");

    // Key named-alias feature: outer_loop.index1 is accessible FROM INSIDE the inner loop
    // The template renders "outer=N" from outer_loop.index1 inside each inner <li>
    assert_eq!(rendered.matches("outer=1").count(), 1 + 3,   // 1 in outer <li> + 3 in inner <li>s
               "outer index1=1 referenced both in outer <li> and in each inner <li>");
    assert_eq!(rendered.matches("outer=2").count(), 1 + 3,
               "outer index1=2 referenced both in outer <li> and in each inner <li>");

    // Depth values are distinct
    assert!(!rendered.contains("outer=1 depth=1"), "outer items must NOT be at depth 1");
    assert!(!rendered.contains("inner=1 depth=0"), "inner items must NOT be at depth 0");
}

// ── loop variable tests ───────────────────────────────────────────────────────

#[function_component]
fn LoopVarsApp() -> Html {
    let items = vec!["Alpha", "Beta", "Gamma", "Delta", "Epsilon"];
    template_html!("templates/loop_vars.html", items={items.iter()}, ...)
}

#[tokio::test(flavor = "current_thread")]
async fn test_loop_variables() {
    let rendered = ServerRenderer::<LoopVarsApp>::new().render().await;
    println!("Loop-vars rendered HTML: {}", rendered);

    // ul 1 — index (0-based) / index1 (1-based) / length
    assert!(rendered.contains("0 / 1 of 5 \u{2014} Alpha"),   "should contain index row for Alpha");
    assert!(rendered.contains("1 / 2 of 5 \u{2014} Beta"),    "should contain index row for Beta");
    assert!(rendered.contains("2 / 3 of 5 \u{2014} Gamma"),   "should contain index row for Gamma");
    assert!(rendered.contains("3 / 4 of 5 \u{2014} Delta"),   "should contain index row for Delta");
    assert!(rendered.contains("4 / 5 of 5 \u{2014} Epsilon"), "should contain index row for Epsilon");

    // ul 2 — first/last markers via present-if
    assert!(rendered.contains("[first]"), "loop.first should render [first] span");
    assert!(rendered.contains("[last]"),  "loop.last should render [last] span");
    // Each marker appears exactly once (only the first / last item)
    assert_eq!(rendered.matches("[first]").count(), 1, "[first] should appear exactly once");
    assert_eq!(rendered.matches("[last]").count(),  1, "[last] should appear exactly once");
}


#[derive(Clone, Copy)]
struct Person {
    first_name: &'static str,
    last_name: &'static str,
}

#[derive(Clone, Copy)]
struct SimpleItem {
    id: i32,
    value: i32,
}


#[test]
fn test_main_set() {
    let boobool = false.to_string();
    let person = Person { first_name: "Edouard", last_name: "G" };
    let zebi = 42;
    let color = "red";
    let _locale = String::from("en");
    let simple_collection = vec![
        SimpleItem { id: 1, value: 100 },
        SimpleItem { id: 2, value: 200 },
    ];

    let _html = template_html!("templates/all_syntax.html", value="tes", value2={5.to_string()}, boobool, opt_value={Some("tes")}, opt_value2={Some("optvalue2")}, names_iter={["Edouart", "Foobar"].iter()}, background_color="#aaa", person, has_password = true, simple_collection = {simple_collection.iter()}, ...);
    let people = vec![
        Person { first_name: "Alice", last_name: "Liddell" },
        Person { first_name: "Bob", last_name: "Builder" },
    ];
    let html2: VNode = template_html!("templates/people_iter.html", people={people.iter()}, ...);

    // we need to validate that Alice and Bob are in the generated HTML
    let html_debug_str = format!("{:?}", html2);

    println!("People iteration HTML generated successfully:\n{:?}", html2);

    // The VNode structure should contain ul elements and the people's names
    // Even though this is debug output, we can validate the structure contains what we expect
    assert!(html_debug_str.contains(r#"tag: "ul""#), "HTML should contain ul tag");
    assert!(html_debug_str.contains(r#"tag: "li""#), "HTML should contain li tags");
    assert!(html_debug_str.contains(r#"text: "Alice""#), "HTML should contain Alice");
    assert!(html_debug_str.contains(r#"text: "Liddell""#), "HTML should contain Liddell");
    assert!(html_debug_str.contains(r#"text: "Bob""#), "HTML should contain Bob");
    assert!(html_debug_str.contains(r#"text: "Builder""#), "HTML should contain Builder");

    // Verify that there are at least 2 ul elements (one for each person due to iteration)
    let ul_count = html_debug_str.matches(r#"tag: "ul""#).count();
    assert!(ul_count >= 1, "Should have at least 1 ul element, found {}", ul_count);

    let simple_items = vec![
        SimpleItem { id: 1, value: 100 },
        SimpleItem { id: 2, value: 200 },
    ];
    let _html3 = template_html!("templates/simple_iter.html", simple_items={simple_items.iter()}, ...);

}

#[function_component]
fn App() -> Html {
    let name = "World";
    template_html!("templates/hello.html", name, ...)
}

// Test server-side rendering with yew template
#[tokio::test(flavor = "current_thread")]
async fn test_server_rendering() {
    let renderer: ServerRenderer<App> = ServerRenderer::new();

    let rendered = renderer.render().await;

    // Verify the rendered HTML contains our expected content
    assert!(rendered.contains("<div>"), "HTML should contain opening div tag");
    assert!(rendered.contains("<p>"), "HTML should contain opening p tag");
    assert!(rendered.contains("Hello World!"), "HTML should contain the text content");
    assert!(rendered.contains("</p>"), "HTML should contain closing p tag");
    assert!(rendered.contains("</div>"), "HTML should contain closing div tag");

    // Prints: <div>Hello, World!</div>
    println!("Server rendered HTML: {}", rendered);
}

#[function_component]
fn PeopleApp() -> Html {
    let people = vec![
        Person { first_name: "Alice", last_name: "Liddell" },
        Person { first_name: "Bob", last_name: "Builder" },
    ];
    template_html!("templates/people_iter.html", people={people.iter()}, ...)
}


// Test server-side rendering with yew-template
#[tokio::test(flavor = "current_thread")]
async fn test_content_people_template() {
    let renderer = ServerRenderer::<PeopleApp>::new();
    let rendered = renderer.render().await;
    println!("Template server rendered HTML: {}", rendered);

    // Verify the rendered HTML contains our template content
    assert!(rendered.contains("<h2>People:</h2>"), "HTML should contain the People header");
    assert!(rendered.contains(r#"<div><h2>People:</h2><ul class="someclass"><li><button id="btn_Alice"></button>Alice Liddell<span>A node that will also be duplicated</span></li><li><button id="btn_Bob"></button>Bob Builder<span>A node that will also be duplicated</span></li></ul>"#), "HTML Should be an iteration of LI's");
}
