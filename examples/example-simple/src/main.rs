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
    let mut show_board = spark!(false);
    let mut rows = spark!(0);

    if show_board {
        for r in 0..rows {
            for c in 0..2 {
                rect! {
                    position: (r * 10, c * 10),
                    color: (255, 0, 0),
                    
                    #[key_type((i32, i32))]
                    key: (r, c),
                }
            }
        }
    }

    // Машина состояний
    if rows == 0 { 
        rows = 2;
        show_board = true;
        println!("Hello world");
    }
}


fn main() {
    firework_ui::run_with_adapter(firework_ui::null_adapter, test_combine_nested_structures_screen);
}
