use firework_ui::ui;

/*
#[ui]
fn test_screen() {
    let mut my_state = spark!(0);

    effect!(my_state, {
        println!("Component mount");
    });

    my_state += 1;
}
*/

#[ui]
fn test_combine_nested_structures_screen() {
    let mut filter_active = spark!(false);
    let mut items_count = spark!(0);

    for i in 0..items_count {
        if !filter_active || i % 2 == 0 {
            rect! {
                position: (10 * i, 10),
                color: (255, 255, 255),
                
                #[key_type(i32)]
                key: i,
            }
        }
    }

    if items_count == 0 {
        items_count = 3; 
    } else if !filter_active {
        filter_active = true; 
    } else if items_count == 3 {
        items_count = 1; 
    }
}


fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_combine_nested_structures_screen);
}
