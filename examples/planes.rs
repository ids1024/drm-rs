use std::{collections::HashMap, fs, io};
use drm::control::property::Value;

pub mod utils;
use crate::utils::*;

fn display_plane(card: &Card, plane_handle: drm::control::plane::Handle, plane: drm::control::plane::Info) -> io::Result<()> {
    let props = card.get_properties(plane_handle)?;
    let mut prop_map = HashMap::new();
    for (handle, value) in &props {
        let info = card.get_property(*handle)?;
        let name = info.name().to_str().unwrap().to_owned();
        //let value = info.value_type().convert_value(*value);
        prop_map.insert(name, (info, value));
    }
    //let prop_map = props.as_hashmap(card)?;
    let value_type = prop_map["type"].0.value_type();
    let Value::Enum(Some(type_)) = value_type.convert_value(*prop_map["type"].1) else {
        panic!();
    };
    println!("        Type: {}", type_.name().to_str().unwrap());
    if let Some(framebuffer_handle) = plane.framebuffer() {
        println!("        {:?}", framebuffer_handle);
        let framebuffer = card.get_planar_framebuffer(framebuffer_handle).unwrap();
        println!("          Format: {:?}", framebuffer.pixel_format());
        if let Some(modifier) = framebuffer.modifier() {
            println!("          Modifier: {:?}", modifier);
        }
    }
    Ok(())
}

fn display_card(card: &Card) -> io::Result<()> {
    card.set_client_capability(drm::ClientCapability::UniversalPlanes, true)?;

    let resources = card.resource_handles()?;

    for connector_handle in resources.connectors() {
        let connector = card.get_connector(*connector_handle, false)?;
        if connector.state() != drm::control::connector::State::Connected {
            continue;
        }

        println!("  {:?}", connector_handle);
        println!("    {:?}{}", connector.interface(), connector.interface_id());
        if let Some(encoder_handle) = connector.current_encoder() {
            let encoder = card.get_encoder(encoder_handle)?;
            println!("    {:?}", encoder_handle);
            println!("      Kind: {:?}", encoder.kind());
            if let Some(crtc_handle) = encoder.crtc() {
                let crtc = card.get_crtc(crtc_handle)?;
                if let Some(mode) = crtc.mode() {
                    println!("      {:?}", mode);
                }
                for plane_handle in card.plane_handles()? {
                    let plane = card.get_plane(plane_handle)?;
                    if plane.crtc() != Some(crtc_handle) {
                        continue;
                    }
                    println!("      {:?}", plane_handle);
                    display_plane(card, plane_handle, plane)?;
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    for i in fs::read_dir("/dev/dri")? {
        let i = i?;
        if i.file_name().to_str().unwrap().starts_with("card") {
            let card = Card::open(i.path().to_str().unwrap());
            println!("{}", i.path().display());
            display_card(&card)?;
        }
    }
    Ok(())
}
