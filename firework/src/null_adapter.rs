// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

#[allow(unused)]

use crate::{AdapterCommand, AdapterResult};

pub fn null_adapter(_cmd: AdapterCommand) -> AdapterResult {
    match &_cmd {
        AdapterCommand::RunLoop { title: _title, width: _width, height: _height, listener: _ } => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] RunLoop: title='{}', size={}x{}", _title, _width, _height);
            
            AdapterResult::Void
        },

        AdapterCommand::RemoveAll => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] RemoveAll");
            
            AdapterResult::Void
        },

        AdapterCommand::NewRect { layout: _layout } => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] NewRect: layout={:?}", _layout);
            
            AdapterResult::Handle(0)
        },

        AdapterCommand::SetPosition(_id, _pos) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetPosition: id={}, pos=({}, {})", _id, _pos.0, _pos.1);
            
            AdapterResult::Void
        },

        AdapterCommand::SetSize(_id, _size) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetSize: id={}, size=({}, {})", _id, _size.0, _size.1);
            
            AdapterResult::Void
        },

        AdapterCommand::SetColor(_id, _color) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetColor: id={}, color=({}, {}, {}, {})", 
                     _id, _color.0, _color.1, _color.2, _color.3);
            
            AdapterResult::Void
        },

        AdapterCommand::SetZ(_id, _z) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetZ: id={}, z={}", _id, _z);
            
            AdapterResult::Void
        },

        AdapterCommand::SetVisible(_id, _visible) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetVisible: id={}, visible={}", _id, _visible);
            
            AdapterResult::Void
        },

        AdapterCommand::Remove(_id) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] Remove: id={}", _id);
            
            AdapterResult::Void
        },

        AdapterCommand::SetHitGroup(_id, _group) => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] SetHitGroup: id={}, group={}", _id, _group);
            
            AdapterResult::Void
        },

        AdapterCommand::ResolveHit(_group, _rect) => { 
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] ResolveHit: group={}, rect=({}, {}, {}, {})", 
                     _group, _rect.0, _rect.1, _rect.2, _rect.3);
            
            AdapterResult::Fail
        },

        AdapterCommand::Render => {
            #[cfg(feature = "detail")]
            println!("[NULL_ADAPTER] Render");
            
            AdapterResult::Void
        }
    }
}
