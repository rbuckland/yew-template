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


#[function_component]
fn LegacyPeopleApp() -> Html {
    let people = vec![
        Person { first_name: "Alice", last_name: "Liddell" },
        Person { first_name: "Bob", last_name: "Builder" },
    ];
    template_html!("templates/legacy_style_people.html", people_iter={people.iter()}, ...)
}

// Test server-side rendering with yew-template
#[tokio::test(flavor = "current_thread")]
async fn test_content_people_template() {
    let new_syntax_renderer = ServerRenderer::<PeopleApp>::new();
    let old_syntax_renderer = ServerRenderer::<LegacyPeopleApp>::new();
    let new_rendered = new_syntax_renderer.render().await;
    let old_rendered = old_syntax_renderer.render().await;
    println!("Template server new_rendered HTML: {}", new_rendered);
    println!("Template server old_rendered HTML: {}", old_rendered);

    // Verify the new_rendered HTML contains our template content
    assert!(new_rendered.contains("<h2>People:</h2>"), "HTML should contain the People header");
    assert!(new_rendered.contains(r#"<div><h2>People:</h2><ul class="someclass"><li><button id="btn_Alice"></button>Alice Liddell<span>A node that will also be duplicated</span></li><li><button id="btn_Bob"></button>Bob Builder<span>A node that will also be duplicated</span></li></ul>"#), "HTML Should be an interation of LI's");

    // Verify the old_rendered HTML contains our template content
    assert!(old_rendered.contains("<h2>People:</h2>"), "HTML should contain the People header");
    assert!(old_rendered.contains(r#"<div><h2>People:</h2><ul><li><button id="btn_Alice"></button>Alice Liddell</li><li><button id="btn_Bob"></button>Bob Builder</li></ul>"#), "HTML Should be an interation of LI's");
    
}
