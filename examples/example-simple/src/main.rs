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
fn test_spark_conditional_rect_screen() {
    // Изначально его нет
    let mut condition = spark!(0); // i32

    // Условный рендеринг
    if condition == 1 {
        rect! {
            position: (10, 10),
            color: (255, 255, 255),
        }

        // Теперь условие снова не выполняется, это перезапустит цикл реактивности
        // и прямоугольник снова исчезнет
        condition += 1;
    }

    // Теперь он должен появится так как условие перезапустится
    condition = 1;
}

fn main() {
    firework_ui::run(test_spark_conditional_rect_screen);
}
