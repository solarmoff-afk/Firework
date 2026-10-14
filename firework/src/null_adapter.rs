// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[allow(unused)]
use crate::{AdapterCommand, AdapterResult, ShaderLanguage};

#[cfg(feature = "std-nulladapter")]
use crate::AdapterEvent;

#[cfg(feature = "std-nulladapter")]
use std::thread;

#[cfg(feature = "std-nulladapter")]
use std::time::{Duration, Instant};

pub fn null_adapter(_cmd: AdapterCommand) -> AdapterResult {
    match &_cmd {
        AdapterCommand::RunLoop {
            title: _title,
            width: _width,
            height: _height,
            listener: _listener,
        } => {
            #[cfg(feature = "detail")]
            println!(
                "[NULL_ADAPTER] RunLoop: title='{}', size={}x{}",
                _title, _width, _height
            );

            #[cfg(feature = "std-nulladapter")]
            {
                let mut last_tick = Instant::now();
                let target_frame_time = Duration::from_millis(16);

                loop {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_tick);

                    if elapsed >= target_frame_time {
                        let delta_ms = elapsed.as_millis() as u32;
                        last_tick = now;

                        _listener(AdapterEvent::Tick(delta_ms));
                    }

                    thread::sleep(Duration::from_millis(1));
                }
            }

            #[cfg(not(feature = "std-nulladapter"))]
            AdapterResult::Void
        }

        AdapterCommand::RemoveAll => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] RemoveAll");

            AdapterResult::Void
        }

        AdapterCommand::NewRect { layout: _layout } => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] NewRect: layout={:?}", _layout);

            AdapterResult::Handle(0)
        }

        AdapterCommand::NewText { layout: _layout } => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] NewText: layout={:?}", _layout);

            AdapterResult::Handle(0)
        }

        AdapterCommand::SetPosition(_id, _pos) => {
            #[cfg(feature = "detail")]
            println!(
                "[NULL_ADAPTER] SetPosition: id={}, pos=({}, {})",
                _id, _pos.0, _pos.1
            );

            AdapterResult::Void
        }

        AdapterCommand::SetSize(_id, _size) => {
            #[cfg(feature = "detail")]
            println!(
                "[NULL_ADAPTER] SetSize: id={}, size=({}, {})",
                _id, _size.0, _size.1
            );

            AdapterResult::Void
        }

        AdapterCommand::SetColor(_id, _color) => {
            #[cfg(feature = "detail")]
            println!(
                "[NULL_ADAPTER] SetColor: id={}, color=({}, {}, {}, {})",
                _id, _color.0, _color.1, _color.2, _color.3
            );

            AdapterResult::Void
        }

        AdapterCommand::SetZ(_id, _z) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetZ: id={}, z={}", _id, _z);

            AdapterResult::Void
        }

        AdapterCommand::SetVisible(_id, _visible) => {
            #[cfg(feature = "detail")]
            println!(
                "[NULL_ADAPTER] SetVisible: id={}, visible={}",
                _id, _visible
            );

            AdapterResult::Void
        }

        AdapterCommand::Remove(_id) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] Remove: id={}", _id);

            AdapterResult::Void
        }

        AdapterCommand::SetHitGroup(_id, _group) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetHitGroup: id={}, group={}", _id, _group);

            AdapterResult::Void
        }

        AdapterCommand::ResolveHit(_group, _rect) => {
            #[cfg(feature = "detail")]
            println!(
                "[NULL_ADAPTER] ResolveHit: group={}, rect=({}, {}, {}, {})",
                _group, _rect.0, _rect.1, _rect.2, _rect.3
            );

            AdapterResult::Fail
        }

        AdapterCommand::Render => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] Render");

            AdapterResult::Void
        }

        AdapterCommand::GetShaderLanguage => AdapterResult::ShaderLang(ShaderLanguage::Software),

        _ => AdapterResult::Void,
    }
}

pub fn test_null_adapter(_cmd: AdapterCommand) -> AdapterResult {
    match &_cmd {
        AdapterCommand::RunLoop {
            title: _title,
            width: _width,
            height: _height,
            listener: _listener,
        } => {
            #[cfg(feature = "detail")]
            println!(
                "[TEST_NULL_ADAPTER] RunLoop: title='{}', size={}x{}",
                _title, _width, _height
            );

            AdapterResult::Void
        }

        AdapterCommand::RemoveAll => {
            #[cfg(feature = "detail")]
            println!("[TEST_NULL_ADAPTER] RemoveAll");

            AdapterResult::Void
        }

        AdapterCommand::NewRect { layout: _layout } => {
            #[cfg(feature = "detail")]
            println!("[TEST_NULL_ADAPTER] NewRect: layout={:?}", _layout);

            AdapterResult::Handle(0)
        }

        AdapterCommand::NewText { layout: _layout } => {
            #[cfg(feature = "detail")]
            println!("[TEST_NULL_ADAPTER] NewText: layout={:?}", _layout);

            AdapterResult::Handle(0)
        }

        AdapterCommand::SetPosition(_id, _pos) => {
            #[cfg(feature = "detail")]
            println!(
                "[TEST_NULL_ADAPTER] SetPosition: id={}, pos=({}, {})",
                _id, _pos.0, _pos.1
            );

            AdapterResult::Void
        }

        AdapterCommand::SetSize(_id, _size) => {
            #[cfg(feature = "detail")]
            println!(
                "[TEST_NULL_ADAPTER] SetSize: id={}, size=({}, {})",
                _id, _size.0, _size.1
            );

            AdapterResult::Void
        }

        AdapterCommand::SetColor(_id, _color) => {
            #[cfg(feature = "detail")]
            println!(
                "[TEST_NULL_ADAPTER] SetColor: id={}, color=({}, {}, {}, {})",
                _id, _color.0, _color.1, _color.2, _color.3
            );

            AdapterResult::Void
        }

        AdapterCommand::SetZ(_id, _z) => {
            #[cfg(feature = "detail")]
            println!("[TEST_NULL_ADAPTER] SetZ: id={}, z={}", _id, _z);

            AdapterResult::Void
        }

        AdapterCommand::SetVisible(_id, _visible) => {
            #[cfg(feature = "detail")]
            println!(
                "[TEST_NULL_ADAPTER] SetVisible: id={}, visible={}",
                _id, _visible
            );

            AdapterResult::Void
        }

        AdapterCommand::Remove(_id) => {
            #[cfg(feature = "detail")]
            println!("[TEST_NULL_ADAPTER] Remove: id={}", _id);

            AdapterResult::Void
        }

        AdapterCommand::SetHitGroup(_id, _group) => {
            #[cfg(feature = "detail")]
            println!(
                "[TEST_NULL_ADAPTER] SetHitGroup: id={}, group={}",
                _id, _group
            );

            AdapterResult::Void
        }

        AdapterCommand::ResolveHit(_group, _rect) => {
            #[cfg(feature = "detail")]
            println!(
                "[TEST_NULL_ADAPTER] ResolveHit: group={}, rect=({}, {}, {}, {})",
                _group, _rect.0, _rect.1, _rect.2, _rect.3
            );

            AdapterResult::Fail
        }

        AdapterCommand::Render => {
            #[cfg(feature = "detail")]
            println!("[TEST_NULL_ADAPTER] Render");

            AdapterResult::Void
        }

        AdapterCommand::GetShaderLanguage => AdapterResult::ShaderLang(ShaderLanguage::Software),

        _ => AdapterResult::Void,
    }
}
