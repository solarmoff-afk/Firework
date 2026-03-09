// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use firework::ui;

fn main() {
    ui! {
        if a == 5 || a == 10 {
            let signal = signal!(10);
            signal += 1;
        }
    }
}
