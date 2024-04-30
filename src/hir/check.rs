use std::collections::HashSet;
use crate::common::*;
use super::*;

impl Package {
    pub fn check(&self) -> Result<(), VirdantError> {
        self.no_duplicate_moddefs()?;
        //self.check_moddefs_acyclic()?;
        for moddef in &self.moddefs() {
            moddef.check_names_unique()?;
            //moddef.check_connections_unique()?;
        }
        Ok(())
    }

    fn no_duplicate_moddefs(&self) -> Result<(), VirdantError> {
        let mut errors = ErrorReport::new();

        let mut items = HashSet::new();
        for item in &self.items {
            let item_name = item.name();
            if !items.insert(item_name.clone()) {
                errors.add(VirdantError::Unknown(format!("Duplicate module definition: {item_name}")));
            }
        }

        errors.check()?;
        Ok(())
    }

    // asserts that no module contains itself as a transitive submodule.
    pub fn check_moddefs_acyclic(&self) -> Result<(), VirdantError> {
        todo!()
    }

}

impl ModDef {
    // asserts that every name in a moddef uniquely identifies either a component or a submodule.
    pub fn check_names_unique(&self) -> Result<(), VirdantError> {
        let mut errors = ErrorReport::new();

        let mut names = HashSet::new();
        for component in &self.components {
            let component_name = component.name();
            if !names.insert(component_name.clone()) {
                errors.add(VirdantError::Unknown(format!("Duplicate component or submodule name: {component_name}")));
            }
        }
        for submodule in &self.submodules {
            let submodule_name = &submodule.name;
            if !names.insert(submodule_name.clone()) {
                errors.add(VirdantError::Unknown(format!("Duplicate component or submodule name: {submodule_name}")));
            }
        }
        errors.check()?;
        Ok(())
    }

    // asserts that no component has two connections.
    pub fn check_connections_unique(&self) -> Result<(), VirdantError> {
        todo!()
    }

    // asserts that all connections have the correct connect type
    // eg, if r is a reg, then you have to use <=
    // but if w is a wire, you have to use :=
    pub fn check_connectiontypes_correct(&self) -> Result<(), VirdantError> {
        todo!()
    }
}
