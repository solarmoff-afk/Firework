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
fn test_spark_derived_rect_screen() {
    let mut x = spark!(10);
    let mut pos = spark!((10, 10)); // (i32, i32)
 
    pos.0 = x;

    rect! {
        position: pos, // Должна быть инициализация с (10, 10)
        color: (255, 255, 255),
    }

    x = 20; // Должно вызвать реакцию с Pos и будет (20, 10)
}

fn main() {
    /*
    firework_ui::run(test_screen);
    */
}
