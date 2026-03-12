use yew_template::*;

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

#[cfg(test)]
mod template_tests {
    use super::*;

    #[test]
    fn test_hello_template() {
        let _html = template_html!("templates/hello.html", name="World", ...);
        println!("✓ hello.html template compiled successfully");
    }

    #[test]
    fn test_fields_template() {
        let person = Person { first_name: "John", last_name: "Doe" };
        let _html = template_html!("templates/fields.html", person, ...);
        println!("✓ fields.html template compiled successfully");
    }

    #[test]
    fn test_opt_template() {
        let _html = template_html!(
            "templates/opt.html", 
            name="Alice",
            opt_age={Some(25)},
            opt_birth_city={Some("Paris")},
            ...
        );
        println!("✓ opt.html template compiled successfully");
    }

    #[test]
    fn test_present_if_template() {
        let _html = template_html!("templates/present_if.html", condition={true}, ...);
        println!("✓ present_if.html template compiled successfully");
    }

    #[test]
    fn test_virtual_template() {
        let _html = template_html!("templates/virtual.html", opt_name={Some("Test")}, ...);
        println!("✓ virtual.html template compiled successfully");
    }

    #[test]
    fn test_iter_template_old_syntax() {
        let contributors = vec!["Alice", "Bob", "Charlie"];
        let _html = template_html!(
            "templates/legacy_iter.html", 
            contributors_iter={contributors.iter()},
            ...
        );
        println!("✓ iter.html template (new syntax) compiled successfully");
    }

    #[test]
    fn test_people_iter_template() {
        let people = vec![
            Person { first_name: "Alice", last_name: "Smith" },
            Person { first_name: "Bob", last_name: "Jones" },
        ];
        let _html = template_html!("templates/people_iter.html", people={people.iter()}, ...);
        println!("✓ people_iter.html template compiled successfully");


    }

    #[test]
    fn test_simple_iter_template() {
        let simple_items = vec![
            SimpleItem { id: 1, value: 100 },
            SimpleItem { id: 2, value: 200 },
        ];
        let _html = template_html!("templates/simple_iter.html", simple_items={simple_items.iter()}, ...);
        println!("✓ simple_iter.html template compiled successfully");
    }

    #[test]
    fn test_all_templates_integration() {
        // Test that all templates can be used together in one test
        let person = Person { first_name: "Integration", last_name: "Test" };
        let people = vec![
            Person { first_name: "User1", last_name: "Last1" },
            Person { first_name: "User2", last_name: "Last2" },
        ];
        let items = vec![
            SimpleItem { id: 1, value: 42 },
            SimpleItem { id: 2, value: 84 },
        ];
        let contributors = vec!["Dev1", "Dev2"];

        let _hello = template_html!("templates/hello.html", name="Integration", ...);
        let _fields = template_html!("templates/fields.html", person, ...);
        let _opt = template_html!("templates/opt.html", name="Test", opt_age={Some(30)}, opt_birth_city={None::<&str>}, ...);
        let _present = template_html!("templates/present_if.html", condition={false}, ...);
        let _virtual = template_html!("templates/virtual.html", opt_name={Some("Virtual Test")}, ...);
        let _iter = template_html!("templates/legacy_iter.html", contributors_iter={contributors.iter()}, ...);
        let _people = template_html!("templates/people_iter.html", people={people.iter()}, ...);
        let _simple = template_html!("templates/simple_iter.html", simple_items={items.iter()}, ...);

        println!(":: All templates compiled");
    }
}