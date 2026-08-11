// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use firework_adapter::{AdapterClickPhase, AdapterCommand, AdapterEvent, AdapterResult};

/// Структура для хранения информации о каждом созданном виджете который будет проксирован
/// в настоящий адаптер
struct ObjectMeta {
    label_id: usize,
    z: i32,
    hit_group: u16,
}

struct DevtoolState {
    inner_adapter: Option<fn(AdapterCommand) -> AdapterResult>,
    original_listener: Option<fn(AdapterEvent)>,
    objects: HashMap<usize, ObjectMeta>,
    info_label_id: Option<usize>,
    last_touch: Option<(u32, u32, AdapterClickPhase)>,
    screen_width: u32,
}

static DEVTOOL_STATE: OnceLock<Mutex<DevtoolState>> = OnceLock::new();

fn get_state() -> std::sync::MutexGuard<'static, DevtoolState> {
    DEVTOOL_STATE
        .get_or_init(|| {
            Mutex::new(DevtoolState {
                inner_adapter: None,
                original_listener: None,
                objects: HashMap::new(),
                info_label_id: None,
                last_touch: None,
                screen_width: 800,
            })
        })
        .lock()
        .unwrap()
}

/// Функция которая устанавливает адаптер к которому идут команды через прокси
pub fn init_visual_devtool(real_adapter: fn(AdapterCommand) -> AdapterResult) {
    let mut state = get_state();
    state.inner_adapter = Some(real_adapter);
}

fn devtool_listener(event: AdapterEvent) {
    let orig = {
        let mut state = get_state();

        if let AdapterEvent::Touch(x, y, phase) = event {
            state.last_touch = Some((x, y, phase));
        }

        state.original_listener
    };

    if let Some(orig) = orig {
        orig(event);
    }
}

/// Сам адаптер который пропускает через себя команды и проксирует в реальный добавляя
/// мета информацию на экран
pub fn visual_devtool_adapter(cmd: AdapterCommand) -> AdapterResult {
    let mut state_lock = get_state();
    let inner = state_lock
        .inner_adapter
        .expect("Proxy not inited, call init_visual_devtool");

    match cmd {
        AdapterCommand::RunLoop {
            title,
            width,
            height,
            listener,
        } => {
            state_lock.screen_width = width;
            state_lock.original_listener = Some(listener);

            drop(state_lock);

            inner(AdapterCommand::RunLoop {
                title,
                width,
                height,
                listener: devtool_listener,
            })
        }

        AdapterCommand::RemoveAll => {
            state_lock.objects.clear();
            let res = inner(cmd);

            if let AdapterResult::Handle(info_id) = inner(AdapterCommand::NewText { layout: 1 }) {
                state_lock.info_label_id = Some(info_id);

                let x = state_lock.screen_width.saturating_sub(200) as i32;
                inner(AdapterCommand::SetPosition(info_id, (x, 10)));
                inner(AdapterCommand::SetZ(info_id, 999999));
                inner(AdapterCommand::SetColor(info_id, (255, 0, 0, 255)));
                inner(AdapterCommand::SetFontSize(info_id, 14));

                inner(AdapterCommand::PushText {
                    handle: info_id,
                    text: "DevTool Ready",
                    mode: 0,
                });
            }
            res
        }

        AdapterCommand::NewRect { .. } | AdapterCommand::NewText { .. } => {
            let res = inner(cmd);
            if let AdapterResult::Handle(obj_id) = res {
                if let AdapterResult::Handle(label_id) =
                    inner(AdapterCommand::NewText { layout: 1 })
                {
                    inner(AdapterCommand::SetZ(label_id, 999998));
                    inner(AdapterCommand::SetColor(label_id, (0, 0, 255, 255)));
                    inner(AdapterCommand::SetFontSize(label_id, 12));

                    let meta = ObjectMeta {
                        label_id,
                        z: 0,
                        hit_group: 0,
                    };

                    update_obj_label(inner, obj_id, &meta);
                    state_lock.objects.insert(obj_id, meta);
                }
            }
            res
        }

        AdapterCommand::SetPosition(id, pos) => {
            let res = inner(cmd);
            if let Some(meta) = state_lock.objects.get(&id) {
                inner(AdapterCommand::SetPosition(
                    meta.label_id,
                    (pos.0, pos.1 - 15),
                ));
            }
            res
        }

        AdapterCommand::SetZ(id, z) => {
            let res = inner(cmd);
            if let Some(meta) = state_lock.objects.get_mut(&id) {
                meta.z = z;
                let label_id = meta.label_id;
                let meta_clone = ObjectMeta {
                    label_id,
                    z,
                    hit_group: meta.hit_group,
                };

                update_obj_label(inner, id, &meta_clone);
            }

            res
        }

        AdapterCommand::SetHitGroup(id, hg) => {
            let res = inner(cmd);
            if let Some(meta) = state_lock.objects.get_mut(&id) {
                meta.hit_group = hg;
                let meta_clone = ObjectMeta {
                    label_id: meta.label_id,
                    z: meta.z,
                    hit_group: hg,
                };

                update_obj_label(inner, id, &meta_clone);
            }

            res
        }

        AdapterCommand::Remove(id) => {
            if let Some(meta) = state_lock.objects.remove(&id) {
                inner(AdapterCommand::Remove(meta.label_id));
            }

            inner(cmd)
        }

        AdapterCommand::ResolveHit(group, _rect) => {
            let res = inner(cmd);
            let hit_id_str = match res {
                AdapterResult::Handle(id) => id.to_string(),
                _ => "None".to_string(),
            };

            if let Some(info_id) = state_lock.info_label_id {
                if let Some((x, y, phase)) = state_lock.last_touch {
                    inner(AdapterCommand::ClearText(info_id));
                    let info_text = format!(
                        "Phase: {:?}\nPos: {},{}\nHit ID: {}\nHit Grp: {}",
                        phase, x, y, hit_id_str, group
                    );

                    inner(AdapterCommand::PushText {
                        handle: info_id,
                        text: &info_text,
                        mode: 0,
                    });
                }
            }
            res
        }

        _ => inner(cmd),
    }
}

fn update_obj_label(inner: fn(AdapterCommand) -> AdapterResult, id: usize, meta: &ObjectMeta) {
    inner(AdapterCommand::ClearText(meta.label_id));
    let text = format!("id:{} z:{} hg:{}", id, meta.z, meta.hit_group);
    inner(AdapterCommand::PushText {
        handle: meta.label_id,
        text: &text,
        mode: 0,
    });
}
