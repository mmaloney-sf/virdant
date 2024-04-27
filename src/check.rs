use super::*;

impl Package {
    pub fn check(&self) {
        self.check_moddefs_acyclic();
        for moddef in &self.moddefs() {
            moddef.check_names_unique();
            moddef.check_connections_unique();
        }
    }

    // asserts that no module contains itself as a transitive submodule.
    pub fn check_moddefs_acyclic(&self) {
        todo!()
    }

}

impl ModDef {
    // asserts that every name in a moddef uniquely identifies either a component or a submodule.
    pub fn check_names_unique(&self) {
        todo!()
    }

    // asserts that no component has two connections.
    pub fn check_connections_unique(&self) {
        todo!()
    }

    // asserts that all connections have the correct connect type
    // eg, if r is a reg, then you have to use <=
    // but if w is a wire, you have to use :=
    pub fn check_connectiontypes_correct(&self) {
        todo!()
    }
}
