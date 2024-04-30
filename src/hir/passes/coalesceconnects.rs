use std::collections::HashMap;
use super::*;

pub struct CoalesceConnects;

impl Pass for CoalesceConnects {
    fn run(&self, mut package: Package) -> Result<Package, VirdantError> {
        for moddef in package.moddefs() {
            //moddef = on_moddef(moddef)?;
            // TODO
        }
        Ok(package)
    }
}

fn on_moddef(moddef: Arc<ModDef>) -> Result<Arc<ModDef>, VirdantError> {
    /*
    let new_moddef = ModDef::clone(moddef);

    let mut inline_connects: HashMap<Ident, &mut InlineConnect> = HashMap::new();
    for component in &moddef.components {
        if let Some(connect) = component.connect_mut() {
            inline_connects.insert(component.name(), connect);
        }
    }

    for connect in &moddef.connects {
        eprintln!("************************************************************************************");
    }
    Ok(Arc::new(new_moddef))
    */
    Ok(moddef.clone())
}
