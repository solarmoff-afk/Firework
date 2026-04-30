use firework_ui::ui;

/*
#[ui]
fn test_screen() {
    let mut my_spark = spark!(0);
    let mut spark2 = spark!(10);

    my_spark = spark2 * 2;

    if my_spark > 0 {
        println!("Больше 0, {}", my_spark);
    }

    spark2 = 20; 
}
*/

#[ui]
fn test_spark_dynamic_rect_screen() {
    let mut count = spark!(3);  

    for i in 0..count {
        rect! {
            position: (10, 10),
            color: (255, 255, 255),
            
            #[key_type(i32)]
            key: i,
        }
    }

    count += 1;
}

fn main() {
    /*
    firework_ui::run(test_screen);
    */
}
